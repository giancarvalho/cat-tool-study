/// Nó inline do núcleo: texto (k="t") ou tag (k="g"). Mesmo JSON dos outros candidatos.
class Inline {
  final String k;
  final String? v;
  final int? id;
  final String? tipo;
  final String? rotulo;
  const Inline.texto(String s)
      : k = 't',
        v = s,
        id = null,
        tipo = null,
        rotulo = null;
  const Inline.tag({required int this.id, required String this.tipo, required String this.rotulo})
      : k = 'g',
        v = null;

  bool get isTag => k == 'g';
  String get chipLabel => tipo == 'abre' ? '$id' : (tipo == 'fecha' ? '/$id' : rotulo ?? '');

  factory Inline.fromJson(Map<String, dynamic> j) =>
      j['k'] == 'g' ? Inline.tag(id: j['id'] as int, tipo: j['tipo'] as String, rotulo: j['rotulo'] as String) : Inline.texto(j['v'] as String);

  Map<String, dynamic> toJson() => isTag ? {'k': 'g', 'id': id, 'tipo': tipo, 'rotulo': rotulo} : {'k': 't', 'v': v};
}

List<Inline> inlinesFromJson(List<dynamic> l) => l.map((e) => Inline.fromJson(e as Map<String, dynamic>)).toList();
List<Map<String, dynamic>> inlinesToJson(List<Inline> l) => l.map((e) => e.toJson()).toList();
bool inlinesIguais(List<Inline> a, List<Inline> b) {
  if (a.length != b.length) return false;
  for (var i = 0; i < a.length; i++) {
    if (a[i].k != b[i].k || a[i].v != b[i].v || a[i].id != b[i].id || a[i].tipo != b[i].tipo || a[i].rotulo != b[i].rotulo) return false;
  }
  return true;
}

class Seg {
  final int id;
  final int blocoId;
  List<Inline> origem;
  List<Inline> destino;
  List<Inline> base;
  int versao;
  String estado;
  int indice = 0;
  Seg(this.id, this.blocoId, this.origem, this.destino, this.versao, this.estado) : base = List.of(destino);
  factory Seg.fromJson(Map<String, dynamic> j) => Seg(
        j['id'] as int,
        j['bloco_id'] as int,
        inlinesFromJson(j['origem'] as List),
        inlinesFromJson(j['destino'] as List),
        (j['versao'] as num).toInt(),
        j['estado'] as String? ?? '',
      );
}

class Bloco {
  final int id;
  final String tipo;
  final int nivel;
  Bloco(this.id, this.tipo, this.nivel);
  factory Bloco.fromJson(Map<String, dynamic> j) => Bloco(j['id'] as int, j['tipo'] as String, (j['nivel'] as num).toInt());
}
