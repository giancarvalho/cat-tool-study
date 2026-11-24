import 'dart:async';
import 'dart:convert';
import 'dart:io';

/// Cliente do núcleo compartilhado (catcore): JSON-RPC por stdin/stdout, uma mensagem por linha (spec §5).
class CoreClient {
  final Process _p;
  final Map<int, Completer<Map<String, dynamic>>> _pend = {};
  int _next = 1;

  CoreClient._(this._p) {
    _p.stdout.transform(utf8.decoder).transform(const LineSplitter()).listen((l) {
      final r = jsonDecode(l) as Map<String, dynamic>;
      _pend.remove(r['id'])?.complete(r);
    });
    _p.stderr.drain();
  }

  static Future<CoreClient> iniciar(String exe, String banco) async => CoreClient._(await Process.start(exe, ['--banco', banco]));

  /// Devolve (resultado, t_core_us).
  Future<(dynamic, int)> rpc(String metodo, [Map<String, dynamic> params = const {}]) async {
    final id = _next++;
    final c = Completer<Map<String, dynamic>>();
    _pend[id] = c;
    _p.stdin.writeln(jsonEncode({'id': id, 'metodo': metodo, 'params': params}));
    final r = await c.future;
    if (r['ok'] != true) throw Exception(r['erro']);
    return (r['resultado'], (r['t_core_us'] as num?)?.toInt() ?? 0);
  }

  void encerrar() {
    try {
      _p.stdin.writeln('{"id":0,"metodo":"sair"}');
    } catch (_) {}
    Timer(const Duration(milliseconds: 800), _p.kill);
  }
}
