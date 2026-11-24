import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:fleather/fleather.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:scrollable_positioned_list/scrollable_positioned_list.dart';

import 'core_client.dart';
import 'editor.dart';
import 'model.dart';
import 'perf.dart';

void main(List<String> args) {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(MaterialApp(
    debugShowCheckedModeBanner: false,
    title: 'CAT Study (Flutter)',
    theme: ThemeData(useMaterial3: true, colorSchemeSeed: const Color(0xFF2563EB), brightness: Brightness.light),
    home: CatApp(args: args),
  ));
}

class CatApp extends StatefulWidget {
  final List<String> args;
  const CatApp({super.key, required this.args});
  @override
  State<CatApp> createState() => _CatAppState();
}

class _CatAppState extends State<CatApp> {
  late final CoreClient core;
  List<Seg> segs = [];
  Map<int, Bloco> blocos = {};
  List<int> blocoOrdem = [];
  Map<int, List<Seg>> blocoSegs = {};
  Map<int, int> blocoIdx = {};
  final Map<int, ValueNotifier<int>> blocoVer = {};
  int docId = 0, active = -1;
  EditorSeg? ed;
  String nome = 'sem documento', estado = 'pronto', ultimo = '', resultados = '';
  bool mostraResultados = false;
  String modo = 'concordancia';
  double? teclaT;
  Completer<bool>? modal;
  String modalMsg = '';
  final listaCtl = ItemScrollController();
  final listaPos = ItemPositionsListener.create();
  final previaCtl = ItemScrollController();
  final previaPos = ItemPositionsListener.create();
  final buscaCtl = TextEditingController();
  final buscaFoco = FocusNode();

  String? arg(String n) {
    final i = widget.args.indexOf(n);
    return i >= 0 && i + 1 < widget.args.length ? widget.args[i + 1] : null;
  }

  @override
  void initState() {
    super.initState();
    Perf.iniciar(arg('--out'));
    HardwareKeyboard.instance.addHandler(aoTeclar);
    unawaited(iniciar());
  }

  Future<void> iniciar() async {
    final dir = Directory('${Platform.environment['LOCALAPPDATA']}\\catstudy-flutter')..createSync(recursive: true);
    final banco = '${dir.path}\\catstudy.db';
    if (widget.args.contains('--limpo') || widget.args.contains('--teste')) {
      for (final suf in ['', '-wal', '-shm']) {
        try {
          File('$banco$suf').deleteSync();
        } catch (_) {}
      }
    }
    core = await CoreClient.iniciar(acharNucleo(), banco);
    final tm = arg('--tm-sintetico');
    if (tm != null) {
      await core.rpc('tm_sintetico', {'n': int.parse(tm), 'semente': 42});
      await core.rpc('tm_aquecer');
    }
    final tmx = arg('--tmx');
    if (tmx != null) {
      await core.rpc('tm_carregar_tmx', {'pasta': tmx, 'origem': 'EN-GB', 'destino': 'PT-PT'});
      await core.rpc('tm_aquecer');
    }
    final doc = arg('--doc');
    if (doc != null) await abrirDoc(doc);
  }

  static String acharNucleo() {
    final env = Platform.environment['CATCORE'];
    if (env != null && env.isNotEmpty) return env;
    for (var d = File(Platform.resolvedExecutable).parent; d.parent.path != d.path; d = d.parent) {
      final p = '${d.path}\\core\\target\\release\\catcore.exe';
      if (File(p).existsSync()) return p;
    }
    throw Exception('catcore.exe não encontrado (defina CATCORE)');
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(aoTeclar);
    Perf.flush();
    core.encerrar();
    super.dispose();
  }

  // ---------- modal ----------
  Future<bool> confirmar(String msg) {
    modalMsg = msg;
    final c = Completer<bool>();
    setState(() => modal = c);
    return c.future;
  }

  // ---------- ativação ----------
  bool visivel(ItemPositionsListener l, int i) => l.itemPositions.value.any((p) => p.index == i && p.itemLeadingEdge >= 0 && p.itemTrailingEdge <= 1);

  void setActive(int i, {bool scroll = true}) {
    if (i < 0 || i >= segs.length) return;
    final velho = ed;
    final seg = segs[i];
    final novo = EditorSeg(seg.destino);
    novo.controller.addListener(() => aoMudar(novo));
    ed = novo;
    final antigo = active;
    active = i;
    if (antigo >= 0 && antigo < segs.length) bump(segs[antigo].blocoId);
    bump(seg.blocoId);
    setState(() {});
    if (velho != null) WidgetsBinding.instance.addPostFrameCallback((_) => velho.dispose());
    final bi = blocoIdx[seg.blocoId];
    WidgetsBinding.instance.addPostFrameCallback((_) {
      // as listas só existem depois do primeiro quadro com o documento aberto
      if (scroll && listaCtl.isAttached && !visivel(listaPos, i)) listaCtl.jumpTo(index: i, alignment: 0.1);
      if (bi != null && previaCtl.isAttached && !visivel(previaPos, bi)) previaCtl.jumpTo(index: bi, alignment: 0.05);
      garantirFoco(novo, 1);
      WidgetsBinding.instance.scheduleFrame();
      WidgetsBinding.instance.addPostFrameCallback((_) => garantirFoco(novo, 1));
    });
  }

  /// Depois da rolagem a linha ativa pode ser reconstruída com o nó de foco já "focado": nesse caso o Fleather
  /// não vê a mudança de foco e não abre a conexão de texto, e as teclas se perdem. Pedir o teclado de novo
  /// (`requestKeyboard`) no quadro seguinte força a conexão (achado do Flutter, ver README).
  void garantirFoco(EditorSeg e, int tentativas) {
    if (e != ed) return;
    e.foco.requestFocus();
    e.chave.currentState?.requestKeyboard(); // abre a conexão de texto mesmo que o nó já estivesse focado
  }

  void bump(int blocoId) {
    final n = blocoVer[blocoId];
    if (n != null) n.value++;
  }

  String ultimoTexto = '';
  void aoMudar(EditorSeg e) {
    if (e != ed || active < 0) return;
    final v = e.inlines();
    final s = segs[active];

    if (inlinesIguais(v, s.destino)) return; // só mudança de seleção
    s.destino = v;
    bump(s.blocoId); // só o bloco do segmento muda
    final k = teclaT;
    teclaT = null;
    if (k == null) return;
    Perf.aposQuadro((tq) {
      final dt = ((tq - k) * 1000).round() / 1000;
      Perf.registrar({'metrica': 'M1', 'passo': 'digitar', 't_recebe_us': Perf.us(k), 't_editor_us': Perf.us(tq), 'valor': dt, 'unidade': 'ms'});
      Perf.registrar({'metrica': 'M2', 'passo': 'digitar', 't_recebe_us': Perf.us(k), 't_previa_us': Perf.us(tq), 'valor': dt, 'unidade': 'ms'});
    });
  }

  // ---------- operações ----------
  Future<void> salvarRascunho() async {
    final s = segs[active];
    if (inlinesIguais(s.destino, s.base)) return;
    final (r, _) = await core.rpc('salvar_segmento', {'id': s.id, 'destino': inlinesToJson(s.destino)});
    s.versao = (r['versao'] as num).toInt();
    s.estado = 'confirmado';
    s.base = List.of(s.destino);
  }

  Future<void> confirmarSeg(bool avancar) async {
    if (active < 0) return;
    final t0 = Perf.nowMs;
    final s = segs[active];
    final (r, tc) = await core.rpc('salvar_segmento', {'id': s.id, 'destino': inlinesToJson(s.destino)});
    s.versao = (r['versao'] as num).toInt();
    s.estado = 'confirmado';
    s.base = List.of(s.destino);
    final t1 = Perf.nowMs;
    if (avancar && active + 1 < segs.length) setActive(active + 1);
    Perf.aposQuadro((tq) {
      Perf.linha('M3', 'confirmar', t0, tq, tCore: tc);
      setState(() => ultimo = 'confirmar: ${(tq - t0).toStringAsFixed(1)} ms (núcleo+SQLite ${(t1 - t0).toStringAsFixed(1)} ms)');
    });
  }

  void descartar() {
    if (active < 0) return;
    final s = segs[active];
    s.destino = List.of(s.base);
    setActive(active, scroll: false);
  }

  void reindexar() {
    for (var i = 0; i < segs.length; i++) {
      segs[i].indice = i;
    }
    blocoSegs = {};
    for (final s in segs) {
      (blocoSegs[s.blocoId] ??= []).add(s);
    }
    blocoIdx = {for (var i = 0; i < blocoOrdem.length; i++) blocoOrdem[i]: i};
  }

  Future<void> mesclar() async {
    if (active < 0 || active + 1 >= segs.length || segs[active].blocoId != segs[active + 1].blocoId) {
      setState(() => estado = 'mesclar: sem segmento seguinte no bloco');
      return;
    }
    final t0 = Perf.nowMs;
    await salvarRascunho();
    final (r, tc) = await core.rpc('mesclar', {'id': segs[active].id});
    final m = Seg.fromJson(r as Map<String, dynamic>);
    final i = active;
    segs.removeAt(i + 1);
    segs[i] = m;
    reindexar();
    blocoVer.values.forEach((n) => n.value++);
    setActive(i);
    Perf.aposQuadro((tq) => Perf.linha('M6', 'mesclar', t0, tq, tCore: tc));
  }

  Future<void> dividir() async {
    if (active < 0 || ed == null) return;
    final t0 = Perf.nowMs;
    final pos = ed!.cursor;
    await salvarRascunho();
    final (r, tc) = await core.rpc('dividir', {'id': segs[active].id, 'pos': pos});
    final par = (r as List).map((e) => Seg.fromJson(e as Map<String, dynamic>)).toList();
    final i = active;
    segs[i] = par[0];
    segs.insert(i + 1, par[1]);
    reindexar();
    blocoVer.values.forEach((n) => n.value++);
    setActive(i);
    Perf.aposQuadro((tq) => Perf.linha('M6', 'dividir', t0, tq, tCore: tc));
  }

  Future<void> abrirDoc(String caminho) async {
    final t0 = Perf.nowMs;
    setState(() => estado = 'importando…');
    final (r, _) = await core.rpc('importar', {'caminho': caminho});
    docId = (r['doc_id'] as num).toInt();
    final total = (r['n_segmentos'] as num).toInt();
    final todos = <Seg>[];
    for (var de = 0; de < total; de += 2000) {
      final (l, _) = await core.rpc('segmentos', {'doc_id': docId, 'de': de, 'n': 2000});
      todos.addAll((l as List).map((e) => Seg.fromJson(e as Map<String, dynamic>)));
    }
    final (bm, _) = await core.rpc('blocos_meta', {'doc_id': docId});
    final bl = (bm as List).map((e) => Bloco.fromJson(e as Map<String, dynamic>)).toList();
    blocos = {for (final b in bl) b.id: b};
    blocoOrdem = bl.map((b) => b.id).toList();
    blocoVer
      ..clear()
      ..addEntries(blocoOrdem.map((id) => MapEntry(id, ValueNotifier<int>(0))));
    segs = todos;
    nome = caminho.split(RegExp(r'[\\/]')).last;
    reindexar();
    active = -1;
    setActive(0);
    Perf.aposQuadro((tq) {
      Perf.linha('M5', 'abrir', t0, tq, erro: nome);
      setState(() => estado = 'pronto · ${segs.length} segmentos · aberto em ${(tq - t0).toStringAsFixed(0)} ms');
      Perf.flush();
    });
  }

  Future<void> buscar() async {
    final texto = modo == 'memoria'
        ? (active >= 0 ? segs[active].origem.where((x) => !x.isTag).map((x) => x.v).join() : '')
        : buscaCtl.text;
    final t0 = Perf.nowMs;
    final (r, tc) = await core.rpc('buscar', {'modo': modo, 'texto': texto});
    final l = r as List;
    setState(() {
      resultados = l.isEmpty
          ? 'sem resultados'
          : l.map((x) => '${(x['pontuacao'] as num).toStringAsFixed(2)}  ${x['origem']}\n    ${x['destino']}').join('\n\n');
      mostraResultados = true;
    });
    Perf.aposQuadro((tq) => Perf.linha('M8', 'buscar', t0, tq, erro: modo, tCore: tc));
  }

  // ---------- instantâneo de estado para os testes funcionais (Ctrl+Alt+F9) ----------
  // A prévia é reconstruída por bloco (ValueListenableBuilder): contamos as construções por bloco para provar
  // que uma tecla reconstrói só o bloco do segmento ativo.
  final Map<int, int> construcoes = {};
  int seqEstado = 0;
  static String marcado(List<Inline> v) => v.map((x) => x.isTag ? '<${x.tipo == 'fecha' ? '/' : ''}${x.id}>' : x.v).join();

  Future<void> instantaneo() async {
    final a = active >= 0 && active < segs.length ? segs[active] : null;
    final idAtivo = a?.blocoId ?? -1;
    var visivel = false;
    if (a != null) {
      for (final p in listaPos.itemPositions.value) {
        if (p.index == active && p.itemTrailingEdge > 0 && p.itemLeadingEdge < 1) visivel = true;
      }
    }
    final (fila, _) = await core.rpc('fila', {'estado': 'pendente'});
    final (cont, _) = await core.rpc('contagem', {'doc_id': docId});
    final d = <String, Object?>{
      'active': active, 'total': segs.length, 'head': ed?.cursor ?? -1, 'modal': modal != null, 'ativoVisivel': visivel,
      'editor': ed != null ? marcado(ed!.inlines()) : '',
      'previaAtivo': a == null ? null : (blocoSegs[idAtivo] ?? const <Seg>[]).map((s) => s.destino.where((x) => !x.isTag).map((x) => x.v).join()).join(' '),
      'previaOutros': construcoes.entries.where((e) => e.key != idAtivo).fold<int>(0, (h, e) => h * 31 + e.key * 7 + e.value),
      'resultados': mostraResultados ? resultados : '',
      'fila': (fila as num).toInt(), 'contagem': (cont as num).toInt(),
      'versao': a?.versao, 'destinoAtivo': a == null ? null : marcado(a.destino),
    };
    Perf.registrar({'metrica': 'ESTADO', 'passo': '${seqEstado++}', 'erro': Uri.encodeComponent(jsonEncode(d))});
    Perf.flush();
  }

  // ---------- teclado (global: antes do foco, como o tunelamento do Avalonia) ----------
  bool aoTeclar(KeyEvent e) {
    if (e is KeyUpEvent) return false;
    final k = e.logicalKey;
    final hk = HardwareKeyboard.instance;
    final ctrl = hk.isControlPressed || hk.isMetaPressed, shift = hk.isShiftPressed, alt = hk.isAltPressed;
    if (e is KeyDownEvent && ctrl && alt && k == LogicalKeyboardKey.f9) {
      instantaneo();
      return true;
    }
    if (modal != null) {
      if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.escape || k == LogicalKeyboardKey.keyY || k == LogicalKeyboardKey.keyS || k == LogicalKeyboardKey.keyN) {
        final sim = k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.keyY || k == LogicalKeyboardKey.keyS;
        final c = modal!;
        setState(() => modal = null);
        c.complete(sim);
      }
      return true;
    }
    final emBusca = buscaFoco.hasFocus;
    final emEd = ed != null && ed!.foco.hasFocus;
    if (e is KeyDownEvent && !ctrl && !alt && emEd && e.character != null && e.character!.isNotEmpty && e.character!.codeUnitAt(0) >= 32) teclaT = Perf.nowMs;

    if (ctrl && k == LogicalKeyboardKey.enter) {
      confirmarSeg(!shift);
      return true;
    }
    if (k == LogicalKeyboardKey.escape) {
      setState(() => mostraResultados = false);
      if (!emBusca) {
        descartar();
        return true;
      }
      return false;
    }
    if (ctrl && !shift && k == LogicalKeyboardKey.keyM) {
      mesclar();
      return true;
    }
    if (ctrl && shift && k == LogicalKeyboardKey.keyD) {
      dividir();
      return true;
    }
    if (ctrl && k == LogicalKeyboardKey.keyF) {
      buscaFoco.requestFocus();
      return true;
    }
    if (k == LogicalKeyboardKey.pageDown && !emBusca) {
      setActive(active + 10 < segs.length ? active + 10 : segs.length - 1);
      return true;
    }
    if (k == LogicalKeyboardKey.pageUp && !emBusca) {
      setActive(active - 10 > 0 ? active - 10 : 0);
      return true;
    }
    if (ctrl && k == LogicalKeyboardKey.home) {
      setActive(0);
      return true;
    }
    if (ctrl && k == LogicalKeyboardKey.end) {
      setActive(segs.length - 1);
      return true;
    }
    if (alt && k == LogicalKeyboardKey.arrowDown) {
      setActive(active + 1);
      return true;
    }
    if (alt && k == LogicalKeyboardKey.arrowUp) {
      setActive(active - 1);
      return true;
    }
    if (!emEd || ed == null) return false;
    final e0 = ed!;
    if (k == LogicalKeyboardKey.enter || k == LogicalKeyboardKey.tab) return true; // segmento de uma linha só
    return false;
  }

  /// Apagar (Backspace/Delete): o Fleather trata a tecla por intenção (DeleteCharacterIntent), que sobrepomos
  /// no editor; um evento de teclado global não impede o Fleather de agir.
  void aoApagar(EditorSeg e0, bool adiante) {
    final a = e0.atomos();
    final sel = e0.controller.selection;
    int ini, fim;
    if (!sel.isCollapsed) {
      ini = sel.start;
      fim = sel.end;
    } else if (adiante) {
      ini = sel.start;
      fim = sel.start + 1;
    } else {
      ini = sel.start - 1;
      fim = sel.start;
    }
    if (ini < 0 || fim > a.length || ini >= fim) return;
    if (a.sublist(ini, fim).any((x) => x.isTag)) {
      apagarComTag(e0, a, ini, fim);
    } else {
      e0.controller.replaceText(ini, fim - ini, '', selection: TextSelection.collapsed(offset: ini));
    }
  }

  /// Ctrl+←/→: a tag é limite de palavra (para antes e depois dela).
  void aoPalavra(EditorSeg e0, bool adiante, bool colapsar) {
    final a = e0.atomos();
    final sel = e0.controller.selection;
    final de = sel.extentOffset;
    final para = adiante ? e0.palavraDir(a, de) : e0.palavraEsq(a, de);
    e0.controller.updateSelection(TextSelection(baseOffset: colapsar ? para : sel.baseOffset, extentOffset: para));
  }

  Future<void> apagarComTag(EditorSeg e0, List<Atomo> a, int ini, int fim) async {
    final tags = [for (var i = ini; i < fim; i++) if (a[i].isTag) i];
    final rot = a[tags.first].tag!.chipLabel.replaceFirst('/', '');
    if (!await confirmar('Apagar tag $rot? (Enter = Sim, Esc = Não)')) return;
    final remover = <int>{for (var i = ini; i < fim; i++) i};
    for (final i in tags) {
      final t = a[i].tag!;
      if (t.tipo == 'abre' || t.tipo == 'fecha' || t.tipo == 'link') {
        for (var j = 0; j < a.length; j++) {
          if (j != i && a[j].isTag && a[j].tag!.id == t.id && (a[j].tag!.tipo == 'abre' || a[j].tag!.tipo == 'fecha' || a[j].tag!.tipo == 'link')) remover.add(j);
        }
      }
    }
    for (final i in (remover.toList()..sort()).reversed) {
      e0.controller.replaceText(i, 1, '', selection: TextSelection.collapsed(offset: i));
    }
    e0.controller.updateSelection(TextSelection.collapsed(offset: ini.clamp(0, e0.atomos().length)));
  }

  // ---------- interface ----------
  Widget textoEstatico(List<Inline> v, {double fonte = kFonte}) => RichText(
        text: TextSpan(
          style: TextStyle(fontSize: fonte, color: const Color(0xFF111827), height: 1.3),
          children: [
            for (final n in v)
              if (n.isTag) WidgetSpan(alignment: PlaceholderAlignment.middle, child: chip(n.chipLabel)) else TextSpan(text: n.v),
          ],
        ),
      );

  Widget linha(int i) {
    final s = segs[i];
    final ativo = i == active && ed != null;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapDown: (_) {
        if (i != active) setActive(i, scroll: false);
      },
      child: Container(
        color: ativo ? const Color(0xFFEFF6FF) : null,
        padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 4),
        child: Row(crossAxisAlignment: CrossAxisAlignment.start, children: [
          SizedBox(width: 44, child: Padding(padding: const EdgeInsets.only(left: 6, top: 4), child: Text('${i + 1}', style: const TextStyle(fontSize: 12, color: Colors.grey)))),
          Expanded(child: Padding(padding: const EdgeInsets.all(4), child: textoEstatico(s.origem))),
          Expanded(
            child: ativo
                ? ListenableBuilder(
                    listenable: ed!.controller,
                    builder: (ctx, _) => LayoutBuilder(
                      builder: (ctx, c) => Container(
                        height: alturaEditor(ed!.atomos(), c.maxWidth),
                        decoration: BoxDecoration(border: Border.all(color: const Color(0xFF2563EB), width: 1.5)),
                        child: FleatherTheme(
                          data: temaEditor(ctx),
                          child: Actions(
                            actions: <Type, Action<Intent>>{
                              DeleteCharacterIntent: CallbackAction<DeleteCharacterIntent>(onInvoke: (i) {
                                aoApagar(ed!, i.forward);
                                return null;
                              }),
                              ExtendSelectionToNextWordBoundaryIntent: CallbackAction<ExtendSelectionToNextWordBoundaryIntent>(onInvoke: (i) {
                                aoPalavra(ed!, i.forward, i.collapseSelection);
                                return null;
                              }),
                              ExtendSelectionToNextWordBoundaryOrCaretLocationIntent: CallbackAction<ExtendSelectionToNextWordBoundaryOrCaretLocationIntent>(onInvoke: (i) {
                                aoPalavra(ed!, i.forward, true);
                                return null;
                              }),
                            },
                            child: FleatherEditor(
                            controller: ed!.controller,
                            focusNode: ed!.foco,
                            editorKey: ed!.chave,
                            embedBuilder: embedBuilder,
                            scrollable: true,
                            autofocus: true,
                            padding: const EdgeInsets.all(4),
                          ),
                          ),
                        ),
                      ),
                    ),
                  )
                : Padding(padding: const EdgeInsets.all(4), child: textoEstatico(s.destino)),
          ),
        ]),
      ),
    );
  }

  Widget blocoPrevia(int idx) {
    final b = blocos[blocoOrdem[idx]]!;
    return ValueListenableBuilder<int>(
      valueListenable: blocoVer[b.id]!,
      builder: (_, __, ___) {
        construcoes[b.id] = (construcoes[b.id] ?? 0) + 1;
        final ativo = active >= 0 && active < segs.length ? segs[active] : null;
        final fonte = b.tipo == 'titulo' ? (b.nivel <= 1 ? 22.0 : b.nivel == 2 ? 18.0 : 16.0) : 14.0;
        final spans = <InlineSpan>[if (b.tipo == 'item_lista') const TextSpan(text: '• ')];
        var neg = false, it = false, link = false;
        var primeiro = true;
        for (final s in blocoSegs[b.id] ?? const <Seg>[]) {
          if (!primeiro) spans.add(const TextSpan(text: ' '));
          primeiro = false;
          final fundo = identical(s, ativo) ? const Color(0xFFFEF3C7) : null;
          for (final n in s.destino) {
            if (!n.isTag) {
              spans.add(TextSpan(
                text: n.v,
                style: TextStyle(
                  fontWeight: neg || b.tipo == 'titulo' ? FontWeight.bold : FontWeight.normal,
                  fontStyle: it ? FontStyle.italic : FontStyle.normal,
                  decoration: link ? TextDecoration.underline : null,
                  backgroundColor: fundo,
                ),
              ));
              continue;
            }
            switch (n.rotulo) {
              case 'b': neg = true;
              case '/b': neg = false;
              case 'i': it = true;
              case '/i': it = false;
              case 'link': link = true;
              case '/link': link = false;
              default: spans.add(WidgetSpan(alignment: PlaceholderAlignment.middle, child: chip('[${n.rotulo}]', fonte: 11)));
            }
          }
        }
        return Container(
          padding: EdgeInsets.fromLTRB(b.tipo == 'item_lista' ? 22 : 8, 4, 8, 4),
          margin: b.tipo == 'tabela' ? const EdgeInsets.symmetric(horizontal: 6, vertical: 1) : null,
          decoration: b.tipo == 'tabela' ? BoxDecoration(border: Border.all(color: const Color(0xFFD1D5DB))) : null,
          child: RichText(text: TextSpan(style: TextStyle(fontSize: fonte, color: const Color(0xFF111827), height: 1.3), children: spans)),
        );
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    const barraCor = Color(0xFFF3F4F6);
    return Scaffold(
      backgroundColor: Colors.white,
      body: Stack(children: [
        Column(children: [
          Container(
            color: barraCor,
            height: 40,
            padding: const EdgeInsets.symmetric(horizontal: 8),
            child: Row(children: [
              TextButton(onPressed: () {}, child: const Text('Abrir')),
              const SizedBox(width: 8),
              Text(nome, style: const TextStyle(fontWeight: FontWeight.w600)),
              const SizedBox(width: 12),
              Text('${active + 1}/${segs.length}'),
              const SizedBox(width: 8),
              TextButton(onPressed: mesclar, child: const Text('Mesclar')),
              TextButton(onPressed: dividir, child: const Text('Dividir')),
              const Spacer(),
              DropdownButton<String>(
                value: modo,
                underline: const SizedBox(),
                items: const [DropdownMenuItem(value: 'concordancia', child: Text('concordancia')), DropdownMenuItem(value: 'memoria', child: Text('memoria'))],
                onChanged: (v) => setState(() {
                  modo = v!;
                  mostraResultados = false;
                }),
              ),
              const SizedBox(width: 8),
              SizedBox(
                width: 280,
                child: TextField(
                  controller: buscaCtl,
                  focusNode: buscaFoco,
                  decoration: const InputDecoration(hintText: 'Buscar (Ctrl+F)', isDense: true, border: OutlineInputBorder()),
                  onSubmitted: (_) => buscar(),
                ),
              ),
            ]),
          ),
          Expanded(
            child: Row(children: [
              Expanded(
                flex: 3,
                child: segs.isEmpty
                    ? const SizedBox()
                    : ScrollablePositionedList.builder(itemCount: segs.length, itemScrollController: listaCtl, itemPositionsListener: listaPos, itemBuilder: (_, i) => linha(i)),
              ),
              Expanded(
                flex: 2,
                child: DecoratedBox(
                  decoration: const BoxDecoration(border: Border(left: BorderSide(color: Color(0xFFD1D5DB)))),
                  child: blocoOrdem.isEmpty
                      ? const SizedBox()
                      : ScrollablePositionedList.builder(itemCount: blocoOrdem.length, itemScrollController: previaCtl, itemPositionsListener: previaPos, itemBuilder: (_, i) => blocoPrevia(i)),
                ),
              ),
            ]),
          ),
          Container(
            color: barraCor,
            height: 26,
            padding: const EdgeInsets.symmetric(horizontal: 10),
            alignment: Alignment.centerLeft,
            child: Row(children: [Text(estado, style: const TextStyle(fontSize: 13)), const SizedBox(width: 24), Text(ultimo, style: const TextStyle(fontSize: 13, color: Colors.grey))]),
          ),
        ]),
        if (mostraResultados)
          Positioned(
            top: 42,
            right: 8,
            width: 560,
            child: Material(
              elevation: 4,
              child: Container(
                constraints: const BoxConstraints(maxHeight: 360),
                padding: const EdgeInsets.all(8),
                child: SingleChildScrollView(child: Text(resultados)),
              ),
            ),
          ),
        if (modal != null)
          Positioned.fill(
            child: Container(
              color: const Color(0x66000000),
              alignment: Alignment.center,
              child: Material(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(mainAxisSize: MainAxisSize.min, children: [
                    Text(modalMsg, style: const TextStyle(fontSize: 16)),
                    const SizedBox(height: 8),
                    const Text('Enter = Sim · Esc = Não', style: TextStyle(color: Colors.grey)),
                  ]),
                ),
              ),
            ),
          ),
      ]),
    );
  }
}
