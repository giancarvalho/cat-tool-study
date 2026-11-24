import 'dart:io';
import 'package:flutter/widgets.dart';

/// Marcações de tempo (spec/quadro.md). Relógio de parede UTC em µs, o mesmo do driver:
/// origem lida uma vez e somada ao Stopwatch (DateTime.now sozinho pode ter resolução baixa).
class Perf {
  // Inicializadas juntas: campos estáticos do Dart são preguiçosos e a origem do relógio de parede
  // precisa ser lida no mesmo instante em que o cronômetro começa.
  static final _Relogio _r = _Relogio();
  static double get nowMs => _r.sw.elapsedMicroseconds / 1000.0;
  static int us(double ms) => _r.origemUs + (ms * 1000).round();

  static const cols = ['run_id', 'app', 'metrica', 'passo', 't_recebe_us', 't_editor_us', 't_previa_us', 't_core_us', 'valor', 'unidade', 'erro'];
  static final List<Map<String, Object?>> _buf = [];
  static String? _out;

  static void iniciar(String? saida) {
    nowMs; // força a origem do relógio já na partida
    _out = saida;
    if (saida != null) {
      // não perder linhas quando o driver encerra o app
      Future.doWhile(() async {
        await Future.delayed(const Duration(milliseconds: 500));
        flush();
        return true;
      });
    }
  }

  /// Dois quadros seguidos: addPostFrameCallback no quadro que reconstruiu a árvore, e outro no quadro seguinte.
  static void aposQuadro(void Function(double tqMs) fn) {
    final b = WidgetsBinding.instance;
    b.scheduleFrame();
    b.addPostFrameCallback((_) {
      b.scheduleFrame();
      b.addPostFrameCallback((_) => fn(nowMs));
    });
  }

  static void registrar(Map<String, Object?> row) => _buf.add(row);

  static void linha(String metrica, String passo, double t0, double tq, {String? erro, int tCore = 0, String? campoFim}) {
    registrar({
      'metrica': metrica, 'passo': passo, 't_recebe_us': us(t0), 'valor': ((tq - t0) * 1000).round() / 1000, 'unidade': 'ms', 'erro': erro,
      't_core_us': tCore == 0 ? null : tCore,
      if (campoFim != null) campoFim: us(tq),
    });
  }

  static void flush() {
    final f = _out;
    if (f == null || _buf.isEmpty) return;
    final rows = List.of(_buf);
    _buf.clear();
    final novo = !File(f).existsSync();
    File(f).parent.createSync(recursive: true);
    final run = pid.toString();
    final sb = StringBuffer();
    if (novo) sb.writeln(cols.join(','));
    for (final r in rows) {
      sb.writeln(cols.map((c) => c == 'app' ? 'flutter' : c == 'run_id' ? run : (r[c]?.toString() ?? '')).join(','));
    }
    File(f).writeAsStringSync(sb.toString(), mode: FileMode.append);
  }
}

class _Relogio {
  late final Stopwatch sw;
  late final int origemUs;
  _Relogio() {
    // DateTime.now() tem resolução de ~15 ms no Windows: espera a virada do valor para que o instante lido seja
    // o instante real (erro menor que 1 ms) e só então liga o cronômetro.
    final t = DateTime.now().toUtc().microsecondsSinceEpoch;
    var n = t;
    while (n == t) {
      n = DateTime.now().toUtc().microsecondsSinceEpoch;
    }
    sw = Stopwatch()..start();
    origemUs = n;
  }
}
