using System.Diagnostics;
using System.Globalization;
using System.Text;
using Avalonia.Controls;
using Avalonia.Threading;

namespace CatStudy;

/// <summary>Marcações de tempo (spec/quadro.md). Relógio de parede UTC em µs, o mesmo do driver.</summary>
public static class Perf
{
    static readonly long OrigemTicks = Stopwatch.GetTimestamp();
    static readonly long OrigemUs = (DateTime.UtcNow.Ticks - DateTime.UnixEpoch.Ticks) / 10;
    public static double NowMs => (Stopwatch.GetTimestamp() - OrigemTicks) * 1000.0 / Stopwatch.Frequency;
    public static long Us(double ms) => OrigemUs + (long)(ms * 1000);

    static readonly string[] Cols = { "run_id", "app", "metrica", "passo", "t_recebe_us", "t_editor_us", "t_previa_us", "t_core_us", "valor", "unidade", "erro" };
    static readonly List<Dictionary<string, object?>> Buf = new();
    static string? _out;
    static Timer? _timer;
    public static TopLevel? Top;

    public static void Iniciar(string? saida)
    {
        _out = saida;
        if (saida != null) _timer = new Timer(_ => Flush(), null, 500, 500); // não perder linhas quando o driver encerra o app
    }

    /// <summary>Dois quadros animados seguidos: o equivalente do rAF duplo (executa depois do quadro seguinte à mudança).</summary>
    public static void AposQuadro(Action<double> fn)
    {
        var t = Top;
        if (t == null) { fn(NowMs); return; }
        t.RequestAnimationFrame(_ => t.RequestAnimationFrame(_ => fn(NowMs)));
    }

    public static void Registrar(Dictionary<string, object?> row)
    {
        lock (Buf) Buf.Add(row);
    }

    public static void Flush()
    {
        if (_out == null) return;
        List<Dictionary<string, object?>> rows;
        lock (Buf) { rows = Buf.ToList(); Buf.Clear(); }
        if (rows.Count == 0) return;
        var novo = !File.Exists(_out);
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(_out))!);
        var sb = new StringBuilder();
        if (novo) sb.AppendLine(string.Join(",", Cols));
        var run = Environment.ProcessId.ToString();
        foreach (var r in rows)
            sb.AppendLine(string.Join(",", Cols.Select(c => c == "app" ? "avalonia" : c == "run_id" ? run : r.TryGetValue(c, out var v) && v != null ? Convert.ToString(v, CultureInfo.InvariantCulture) : "")));
        lock (Buf) File.AppendAllText(_out, sb.ToString());
    }

    public static Dictionary<string, object?> Linha(string metrica, string passo, double t0Ms, double tqMs, string? erro = null, long tCore = 0, string campoFim = "") => new()
    {
        ["metrica"] = metrica, ["passo"] = passo, ["t_recebe_us"] = Us(t0Ms), ["valor"] = Math.Round(tqMs - t0Ms, 3), ["unidade"] = "ms", ["erro"] = erro,
        ["t_core_us"] = tCore == 0 ? null : tCore,
        [campoFim] = campoFim == "" ? null : Us(tqMs),
    };
}
