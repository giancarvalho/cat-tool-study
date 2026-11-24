import 'package:fleather/fleather.dart';
import 'package:flutter/material.dart';
import 'package:parchment/parchment.dart';
import 'package:parchment_delta/parchment_delta.dart';

import 'model.dart';

const double kFonte = 15;
final Color kCinzaChip = const Color(0xFFE5E7EB);

/// Chip de tag (retângulo arredondado, rótulo monoespaçado). Usado no editor (embed) e nos textos estáticos.
Widget chip(String rotulo, {double fonte = 12}) => Container(
      margin: const EdgeInsets.symmetric(horizontal: 1),
      padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1),
      decoration: BoxDecoration(color: kCinzaChip, borderRadius: BorderRadius.circular(4)),
      child: Text(rotulo, style: TextStyle(fontFamily: 'Consolas', fontSize: fonte, color: const Color(0xFF374151), height: 1.1)),
    );

String _rotuloDe(Map data) {
  final tipo = data['tipo'];
  return tipo == 'abre' ? '${data['id']}' : (tipo == 'fecha' ? '/${data['id']}' : '${data['rotulo']}');
}

Widget embedBuilder(BuildContext context, EmbedNode node) {
  final d = node.value.data;
  return chip(_rotuloDe(d));
}

/// Átomo do editor: 1 unidade de texto (UTF-16) ou 1 tag. A posição do cursor no Fleather é o índice do átomo.
class Atomo {
  final String? ch;
  final Inline? tag;
  const Atomo.ch(String this.ch) : tag = null;
  const Atomo.tag(Inline this.tag) : ch = null;
  bool get isTag => tag != null;
  bool get espaco => ch != null && ch!.trim().isEmpty;
}

ParchmentDocument documentoDe(List<Inline> v) {
  final d = Delta();
  for (final n in v) {
    if (n.isTag) {
      d.insert(SpanEmbed('tag', data: {'id': n.id, 'tipo': n.tipo, 'rotulo': n.rotulo}).toJson());
    } else if (n.v!.isNotEmpty) {
      d.insert(n.v!);
    }
  }
  d.insert('\n');
  return ParchmentDocument.fromDelta(d);
}

/// Editor de um segmento: controlador Fleather + foco; conversões de e para os nós do núcleo.
class EditorSeg {
  final FleatherController controller;
  final FocusNode foco = FocusNode();
  final GlobalKey<EditorState> chave = GlobalKey<EditorState>();
  EditorSeg(List<Inline> v) : controller = FleatherController(document: documentoDe(v), autoFormats: AutoFormats(autoFormats: const []));

  List<Atomo> atomos() {
    final out = <Atomo>[];
    for (final op in controller.document.toDelta().toList()) {
      if (!op.isInsert) continue;
      final data = op.data;
      if (data is String) {
        for (final u in data.codeUnits) {
          if (u == 10) continue; // fim de linha do documento
          out.add(Atomo.ch(String.fromCharCode(u)));
        }
      } else if (data is Map) {
        out.add(Atomo.tag(Inline.tag(id: data['id'] as int, tipo: data['tipo'] as String, rotulo: data['rotulo'] as String)));
      }
    }
    return out;
  }

  List<Inline> inlines() {
    final o = <Inline>[];
    final sb = StringBuffer();
    void despejar() {
      if (sb.isNotEmpty) {
        o.add(Inline.texto(sb.toString()));
        sb.clear();
      }
    }

    for (final a in atomos()) {
      if (a.isTag) {
        despejar();
        o.add(a.tag!);
      } else {
        sb.write(a.ch);
      }
    }
    despejar();
    return o;
  }

  int get cursor => controller.selection.extentOffset < 0 ? 0 : controller.selection.extentOffset;

  // limites de palavra: a tag é sempre um limite (para antes e depois dela)
  int palavraDir(List<Atomo> a, int i) {
    final n = a.length;
    if (i >= n) return n;
    if (a[i].isTag) return i + 1;
    while (i < n && !a[i].isTag && a[i].espaco) {
      i++;
    }
    if (i < n && a[i].isTag) return i;
    while (i < n && !a[i].isTag && !a[i].espaco) {
      i++;
    }
    return i;
  }

  int palavraEsq(List<Atomo> a, int i) {
    if (i <= 0) return 0;
    if (a[i - 1].isTag) return i - 1;
    while (i > 0 && !a[i - 1].isTag && a[i - 1].espaco) {
      i--;
    }
    if (i > 0 && a[i - 1].isTag) return i;
    while (i > 0 && !a[i - 1].isTag && !a[i - 1].espaco) {
      i--;
    }
    return i;
  }

  void dispose() {
    controller.dispose();
    foco.dispose();
  }
}

/// Tema do editor: mesma fonte e entrelinha do texto estático, sem espaçamento entre parágrafos.
FleatherThemeData temaEditor(BuildContext context) => FleatherThemeData.fallback(context).copyWith(
      paragraph: TextBlockTheme(style: const TextStyle(fontSize: kFonte, height: 1.3, color: Color(0xFF111827)), spacing: const VerticalSpacing.zero()),
    );

/// O Fleather precisa de altura limitada para rolar e mover o cursor na vertical; estimamos a altura do texto
/// (tags contam como chips de 3 caracteres) para o editor crescer junto com o conteúdo.
double alturaEditor(List<Atomo> a, double largura) {
  final sb = StringBuffer();
  for (final x in a) {
    sb.write(x.isTag ? ' ${x.tag!.chipLabel} ' : x.ch);
  }
  final tp = TextPainter(
    text: TextSpan(text: sb.isEmpty ? ' ' : sb.toString(), style: const TextStyle(fontSize: kFonte, height: 1.3)),
    textDirection: TextDirection.ltr,
  )..layout(maxWidth: (largura - 8 - 3).clamp(40.0, 4000.0));
  final h = tp.height + 8 + 3 + 4;
  tp.dispose();
  return h;
}
