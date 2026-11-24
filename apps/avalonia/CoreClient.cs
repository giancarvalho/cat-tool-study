using System.Collections.Concurrent;
using System.Diagnostics;
using System.Text.Json;

namespace CatStudy;

/// <summary>Cliente do núcleo compartilhado (catcore): JSON-RPC por stdin/stdout, uma mensagem por linha (spec §5).</summary>
public sealed class CoreClient : IDisposable
{
    readonly Process _p;
    readonly ConcurrentDictionary<int, TaskCompletionSource<JsonElement>> _pend = new();
    int _next = 1;

    public CoreClient(string exe, string banco)
    {
        _p = Process.Start(new ProcessStartInfo(exe, new[] { "--banco", banco })
        {
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            UseShellExecute = false,
            CreateNoWindow = true,
            StandardOutputEncoding = new System.Text.UTF8Encoding(false),
            StandardInputEncoding = new System.Text.UTF8Encoding(false),
        })!;
        _p.StandardInput.AutoFlush = true;
        new Thread(Ler) { IsBackground = true }.Start();
    }

    void Ler()
    {
        string? l;
        while ((l = _p.StandardOutput.ReadLine()) != null)
        {
            var doc = JsonDocument.Parse(l);
            var id = doc.RootElement.GetProperty("id").GetInt32();
            if (_pend.TryRemove(id, out var tcs)) tcs.SetResult(doc.RootElement.Clone());
        }
    }

    public sealed record Resposta(JsonElement Resultado, long TCoreUs);

    public async Task<Resposta> Rpc(string metodo, object? parametros = null)
    {
        var id = Interlocked.Increment(ref _next);
        var tcs = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously);
        _pend[id] = tcs;
        var msg = JsonSerializer.Serialize(new { id, metodo, @params = parametros ?? new { } }, Json.Opt);
        lock (_p) _p.StandardInput.WriteLine(msg);
        var r = await tcs.Task;
        if (!r.GetProperty("ok").GetBoolean()) throw new Exception(r.GetProperty("erro").GetString());
        return new Resposta(r.GetProperty("resultado"), r.TryGetProperty("t_core_us", out var t) ? t.GetInt64() : 0);
    }

    public async Task<T> Rpc<T>(string metodo, object? parametros = null)
    {
        var r = await Rpc(metodo, parametros);
        return r.Resultado.Deserialize<T>(Json.Opt)!;
    }

    public void Dispose()
    {
        try { lock (_p) _p.StandardInput.WriteLine("{\"id\":0,\"metodo\":\"sair\"}"); } catch { }
        try { if (!_p.WaitForExit(1000)) _p.Kill(); } catch { }
    }
}
