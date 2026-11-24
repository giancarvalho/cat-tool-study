using System.Collections.ObjectModel;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Documents;
using Avalonia.Controls.Templates;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;

namespace CatStudy;

/// <summary>Linha da lista: nº, origem (somente leitura) e destino (editor próprio; editável só no segmento ativo).</summary>
public sealed class Row : Grid
{
    public readonly Seg Seg;
    public readonly TextBlock Num = new() { Foreground = Brushes.Gray, Margin = new Thickness(6, 7, 0, 0), FontSize = 12 };
    public readonly SegmentEditor Orig = new();
    public readonly SegmentEditor Dest = new();
    public Row(Seg s, MainWindow w)
    {
        Seg = s;
        ColumnDefinitions = new ColumnDefinitions("44,*,*");
        Num.Text = (s.Indice + 1).ToString();
        Orig.Load(s.Origem);
        Dest.Load(s.Destino);
        Dest.Changed += v => w.AoEditar(s, v);
        Dest.ConfirmarTag = r => w.Confirmar($"Apagar tag {r}? (Enter = Sim, Esc = Não)");
        SetColumn(Orig, 1);
        SetColumn(Dest, 2);
        Children.Add(Num); Children.Add(Orig); Children.Add(Dest);
        AttachedToVisualTree += (_, _) => w.LinhaCriada(this);
        DetachedFromVisualTree += (_, _) => w.LinhaRemovida(this);
    }
    public void Ativar(bool a)
    {
        Dest.ReadOnly = !a;
        Background = a ? new SolidColorBrush(Color.FromRgb(0xEF, 0xF6, 0xFF)) : null;
        Dest.InvalidateVisual();
    }
}

public sealed class MainWindow : Window
{
    // ---- estado (mesmo modelo do app do Electron) ----
    readonly CoreClient _core;
    readonly ObservableCollection<Seg> _segs = new();
    Dictionary<long, Bloco> _blocos = new();
    List<long> _blocoOrdem = new();
    Dictionary<long, List<Seg>> _blocoSegs = new();
    Dictionary<long, int> _blocoIdx = new();
    readonly Dictionary<Seg, Row> _rows = new();
    readonly Dictionary<long, TextBlock> _previaTb = new();
    long _docId;
    int _active = -1;
    bool _ignoraSel;
    double? _teclaT;
    TaskCompletionSource<bool>? _modal;

    // ---- controles ----
    readonly ListBox _lista = new();
    readonly ListBox _previa = new();
    readonly TextBlock _nome = new() { Text = "sem documento", VerticalAlignment = VerticalAlignment.Center, FontWeight = FontWeight.SemiBold };
    readonly TextBlock _contador = new() { Text = "0/0", VerticalAlignment = VerticalAlignment.Center };
    readonly TextBlock _estado = new() { Text = "pronto" };
    readonly TextBlock _ultimo = new() { Foreground = Brushes.Gray };
    readonly ComboBox _modo = new() { ItemsSource = new[] { "concordancia", "memoria" }, SelectedIndex = 0, Width = 140 };
    readonly TextBox _busca = new() { Width = 280, Watermark = "Buscar (Ctrl+F)" };
    readonly Border _resultados = new() { IsVisible = false, Background = Brushes.White, BorderBrush = Brushes.Gray, BorderThickness = new Thickness(1), HorizontalAlignment = HorizontalAlignment.Right, VerticalAlignment = VerticalAlignment.Top, Margin = new Thickness(0, 42, 8, 0), MaxHeight = 360, Width = 560, Padding = new Thickness(8) };
    readonly TextBlock _resultadosTxt = new() { TextWrapping = TextWrapping.Wrap };
    readonly Border _modalBox = new() { IsVisible = false, Background = new SolidColorBrush(Color.FromArgb(0x66, 0, 0, 0)) };
    readonly TextBlock _modalTxt = new() { FontSize = 16 };

    public MainWindow(string[] args)
    {
        Title = "CAT Study (Avalonia)";
        Width = 1440; Height = 900; CanResize = false;
        WindowStartupLocation = WindowStartupLocation.CenterScreen;
        Background = Brushes.White;
        RequestedThemeVariant = Avalonia.Styling.ThemeVariant.Light;

        string? Val(string n) { var i = Array.IndexOf(args, n); return i >= 0 && i + 1 < args.Length ? args[i + 1] : null; }
        Perf.Iniciar(Val("--out"));
        Perf.Top = this;
        var dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "catstudy-avalonia");
        Directory.CreateDirectory(dir);
        var banco = Path.Combine(dir, "catstudy.db");
        if (args.Contains("--limpo") || args.Contains("--teste")) foreach (var suf in new[] { "", "-wal", "-shm" }) { try { File.Delete(banco + suf); } catch { } }
        _core = new CoreClient(AcharNucleo(), banco);

        MontarInterface();
        Closing += (_, _) => { Perf.Flush(); _core.Dispose(); };
        AddHandler(KeyDownEvent, AoTeclar, RoutingStrategies.Tunnel);
        Opened += async (_, _) =>
        {
            if (Val("--tmx") is { } pasta) { await _core.Rpc("tm_carregar_tmx", new { pasta, origem = "EN-GB", destino = "PT-PT" }); await _core.Rpc("tm_aquecer"); }
            if (Val("--tm-sintetico") is { } n) { await _core.Rpc("tm_sintetico", new { n = int.Parse(n), semente = 42 }); await _core.Rpc("tm_aquecer"); }
            if (Val("--doc") is { } doc) await AbrirDoc(doc);
        };
    }

    static string AcharNucleo()
    {
        var env = Environment.GetEnvironmentVariable("CATCORE");
        if (!string.IsNullOrEmpty(env)) return env;
        for (var d = new DirectoryInfo(AppContext.BaseDirectory); d != null; d = d.Parent)
        {
            var p = Path.Combine(d.FullName, "core", "target", "release", "catcore.exe");
            if (File.Exists(p)) return p;
        }
        throw new FileNotFoundException("catcore.exe não encontrado (defina CATCORE)");
    }

    // ---------- interface ----------
    void MontarInterface()
    {
        Button Btn(string t, Action a) { var b = new Button { Content = t, Margin = new Thickness(4, 0), VerticalAlignment = VerticalAlignment.Center }; b.Click += (_, _) => a(); return b; }
        var barra = new Border
        {
            Background = new SolidColorBrush(Color.FromRgb(0xF3, 0xF4, 0xF6)), Height = 40, Padding = new Thickness(8, 0),
            Child = new DockPanel
            {
                LastChildFill = false,
                Children =
                {
                    Dock(new StackPanel { Orientation = Orientation.Horizontal, Spacing = 10, Children = { Btn("Abrir", () => { }), _nome, _contador, Btn("Mesclar", () => _ = Mesclar()), Btn("Dividir", () => _ = Dividir()) } }, Avalonia.Controls.Dock.Left),
                    Dock(new StackPanel { Orientation = Orientation.Horizontal, Spacing = 6, VerticalAlignment = VerticalAlignment.Center, Children = { _modo, _busca } }, Avalonia.Controls.Dock.Right),
                },
            },
        };
        var rodape = new Border { Background = new SolidColorBrush(Color.FromRgb(0xF3, 0xF4, 0xF6)), Height = 26, Padding = new Thickness(10, 3), Child = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 24, Children = { _estado, _ultimo } } };

        _lista.ItemsSource = _segs;
        _lista.ItemTemplate = new FuncDataTemplate<Seg>((s, _) => new Row(s, this), true);
        _lista.SelectionChanged += (_, _) => { if (!_ignoraSel && _lista.SelectedIndex >= 0 && _lista.SelectedIndex != _active) SetActive(_lista.SelectedIndex, false); };
        _previa.ItemsSource = _blocoOrdemObs;
        _previa.ItemTemplate = new FuncDataTemplate<long>((id, _) => PreviaItem(id), true);
        _previa.SelectionMode = SelectionMode.Single;

        var principal = new Grid { ColumnDefinitions = new ColumnDefinitions("3*,2*") };
        Grid.SetColumn(_lista, 0); Grid.SetColumn(_previa, 1);
        principal.Children.Add(_lista); principal.Children.Add(_previa);
        _previa.BorderBrush = Brushes.LightGray; _previa.BorderThickness = new Thickness(1, 0, 0, 0);

        var corpo = new DockPanel();
        DockPanel.SetDock(barra, Avalonia.Controls.Dock.Top); DockPanel.SetDock(rodape, Avalonia.Controls.Dock.Bottom);
        corpo.Children.Add(barra); corpo.Children.Add(rodape); corpo.Children.Add(principal);

        _resultados.Child = new ScrollViewer { Content = _resultadosTxt };
        _modalBox.Child = new Border { Background = Brushes.White, BorderBrush = Brushes.Gray, BorderThickness = new Thickness(1), Padding = new Thickness(24), HorizontalAlignment = HorizontalAlignment.Center, VerticalAlignment = VerticalAlignment.Center, Child = new StackPanel { Spacing = 8, Children = { _modalTxt, new TextBlock { Text = "Enter = Sim · Esc = Não", Foreground = Brushes.Gray } } } };
        _busca.KeyDown += (_, e) => { if (e.Key == Key.Enter) { e.Handled = true; _ = Buscar(); } };
        _modo.SelectionChanged += (_, _) => _resultados.IsVisible = false;

        Content = new Grid { Children = { corpo, _resultados, _modalBox } };
    }
    static T Dock<T>(T c, Dock d) where T : Control { DockPanel.SetDock(c, d); return c; }
    readonly ObservableCollection<long> _blocoOrdemObs = new();

    Control PreviaItem(long id)
    {
        var b = _blocos[id];
        var tb = new TextBlock { TextWrapping = TextWrapping.Wrap, FontSize = b.Tipo == "titulo" ? (b.Nivel <= 1 ? 22 : b.Nivel == 2 ? 18 : 16) : 14, FontWeight = b.Tipo == "titulo" ? FontWeight.Bold : FontWeight.Normal };
        _previaTb[id] = tb;
        tb.DetachedFromVisualTree += (_, _) => { if (_previaTb.TryGetValue(id, out var x) && x == tb) _previaTb.Remove(id); };
        PreencherPrevia(tb, id);
        var box = new Border { Child = tb, Padding = new Thickness(b.Tipo == "item_lista" ? 22 : 8, 4, 8, 4) };
        if (b.Tipo == "tabela") { box.BorderBrush = Brushes.LightGray; box.BorderThickness = new Thickness(1); box.Margin = new Thickness(6, 1); }
        return box;
    }

    void PreencherPrevia(TextBlock tb, long blocoId)
    {
        var b = _blocos[blocoId];
        var ink = tb.Inlines ??= new InlineCollection();
        ink.Clear();
        if (b.Tipo == "item_lista") ink.Add(new Run("• "));
        bool neg = false, it = false, link = false;
        var ativo = _active >= 0 ? _segs[_active] : null;
        bool primeiro = true;
        foreach (var s in _blocoSegs.GetValueOrDefault(blocoId) ?? new())
        {
            if (!primeiro) ink.Add(new Run(" "));
            primeiro = false;
            IBrush? fundo = s == ativo ? new SolidColorBrush(Color.FromRgb(0xFE, 0xF3, 0xC7)) : null;
            foreach (var n in s.Destino)
            {
                if (!n.IsTag) { ink.Add(new Run(n.V ?? "") { FontWeight = neg ? FontWeight.Bold : FontWeight.Normal, FontStyle = it ? FontStyle.Italic : FontStyle.Normal, TextDecorations = link ? TextDecorations.Underline : null, Background = fundo }); continue; }
                switch (n.Rotulo)
                {
                    case "b": neg = true; break;
                    case "/b": neg = false; break;
                    case "i": it = true; break;
                    case "/i": it = false; break;
                    case "link": link = true; break;
                    case "/link": link = false; break;
                    default: ink.Add(new Run($"[{n.Rotulo}]") { Foreground = Brushes.Gray, Background = new SolidColorBrush(Color.FromRgb(0xE5, 0xE7, 0xEB)), FontFamily = new FontFamily("Consolas"), FontSize = 11 }); break;
                }
            }
        }
    }

    void AtualizarBloco(long blocoId) { if (_previaTb.TryGetValue(blocoId, out var tb)) PreencherPrevia(tb, blocoId); }
    void AtualizarTodosBlocos() { foreach (var (id, tb) in _previaTb.ToList()) PreencherPrevia(tb, id); }

    // ---------- linhas e ativação ----------
    public void LinhaCriada(Row r)
    {
        _rows[r.Seg] = r;
        bool ativa = _active >= 0 && _active < _segs.Count && _segs[_active] == r.Seg;
        r.Ativar(ativa);
        if (ativa) Dispatcher.UIThread.Post(() => r.Dest.Focus(), DispatcherPriority.Input);
    }
    public void LinhaRemovida(Row r) { if (_rows.TryGetValue(r.Seg, out var x) && x == r) _rows.Remove(r.Seg); }

    SegmentEditor? Ed => _active >= 0 && _active < _segs.Count && _rows.TryGetValue(_segs[_active], out var r) ? r.Dest : null;

    void SetActive(int i, bool scroll = true)
    {
        if (i < 0 || i >= _segs.Count) return;
        if (_active >= 0 && _active < _segs.Count && _rows.TryGetValue(_segs[_active], out var velha))
        {
            velha.Ativar(false);
            velha.Dest.Load(velha.Seg.Destino);
            AtualizarBloco(velha.Seg.BlocoId);
        }
        _active = i;
        var s = _segs[i];
        _ignoraSel = true; _lista.SelectedIndex = i; _ignoraSel = false;
        if (scroll) _lista.ScrollIntoView(i);
        if (_rows.TryGetValue(s, out var nova))
        {
            nova.Dest.Load(s.Destino);
            nova.Ativar(true);
            nova.Dest.Focus();
        }
        if (_blocoIdx.TryGetValue(s.BlocoId, out var bi)) _previa.ScrollIntoView(bi);
        AtualizarBloco(s.BlocoId);
        _contador.Text = $"{i + 1}/{_segs.Count}";
    }

    public void AoEditar(Seg s, List<Inline> v)
    {
        s.Destino = v;
        AtualizarBloco(s.BlocoId); // só o bloco do segmento muda
        var k = _teclaT; _teclaT = null;
        if (k == null) return;
        Perf.AposQuadro(tq =>
        {
            var dt = Math.Round(tq - k.Value, 3);
            Perf.Registrar(new() { ["metrica"] = "M1", ["passo"] = "digitar", ["t_recebe_us"] = Perf.Us(k.Value), ["t_editor_us"] = Perf.Us(tq), ["valor"] = dt, ["unidade"] = "ms" });
            Perf.Registrar(new() { ["metrica"] = "M2", ["passo"] = "digitar", ["t_recebe_us"] = Perf.Us(k.Value), ["t_previa_us"] = Perf.Us(tq), ["valor"] = dt, ["unidade"] = "ms" });
        });
    }

    // ---------- modal ----------
    public Task<bool> Confirmar(string msg)
    {
        _modalTxt.Text = msg;
        _modalBox.IsVisible = true;
        _modal = new TaskCompletionSource<bool>();
        return _modal.Task;
    }

    // ---------- operações ----------
    async Task SalvarRascunho()
    {
        var s = _segs[_active];
        if (Json.Igual(s.Destino, s.Base)) return;
        var r = await _core.Rpc("salvar_segmento", new { id = s.Id, destino = s.Destino });
        s.Versao = r.Resultado.GetProperty("versao").GetInt64();
        s.Estado = "confirmado";
        s.Base = Json.CloneList(s.Destino);
    }

    async Task ConfirmarSeg(bool avancar)
    {
        if (_active < 0) return;
        var t0 = Perf.NowMs;
        var s = _segs[_active];
        var r = await _core.Rpc("salvar_segmento", new { id = s.Id, destino = s.Destino });
        s.Versao = r.Resultado.GetProperty("versao").GetInt64();
        s.Estado = "confirmado";
        s.Base = Json.CloneList(s.Destino);
        var t1 = Perf.NowMs;
        if (avancar && _active + 1 < _segs.Count) SetActive(_active + 1);
        Perf.AposQuadro(tq =>
        {
            Perf.Registrar(Perf.Linha("M3", "confirmar", t0, tq, null, r.TCoreUs));
            _ultimo.Text = $"confirmar: {tq - t0:F1} ms (núcleo+SQLite {t1 - t0:F1} ms)";
        });
    }

    void Descartar()
    {
        if (_active < 0) return;
        var s = _segs[_active];
        s.Destino = Json.CloneList(s.Base);
        Ed?.Load(s.Destino);
        AtualizarBloco(s.BlocoId);
    }

    void Reindexar()
    {
        for (int i = 0; i < _segs.Count; i++) _segs[i].Indice = i;
        _blocoSegs = new();
        foreach (var s in _segs) { if (!_blocoSegs.TryGetValue(s.BlocoId, out var l)) _blocoSegs[s.BlocoId] = l = new(); l.Add(s); }
        _blocoIdx = _blocoOrdem.Select((id, i) => (id, i)).ToDictionary(x => x.id, x => x.i);
        foreach (var r in _rows.Values) r.Num.Text = (r.Seg.Indice + 1).ToString();
    }

    async Task Mesclar()
    {
        if (_active < 0 || _active + 1 >= _segs.Count || _segs[_active].BlocoId != _segs[_active + 1].BlocoId) { _estado.Text = "mesclar: sem segmento seguinte no bloco"; return; }
        var t0 = Perf.NowMs;
        await SalvarRascunho();
        var r = await _core.Rpc("mesclar", new { id = _segs[_active].Id });
        var m = r.Resultado.Deserialize<Seg>(Json.Opt)!;
        m.Base = Json.CloneList(m.Destino);
        var i = _active;
        _active = -1;
        _segs.RemoveAt(i + 1);
        _segs[i] = m;
        Reindexar();
        SetActive(i, true);
        AtualizarTodosBlocos();
        Perf.AposQuadro(tq => Perf.Registrar(Perf.Linha("M6", "mesclar", t0, tq, null, r.TCoreUs)));
    }

    async Task Dividir()
    {
        if (_active < 0 || Ed == null) return;
        var t0 = Perf.NowMs;
        var pos = Ed.AtomIndex;
        await SalvarRascunho();
        var r = await _core.Rpc("dividir", new { id = _segs[_active].Id, pos });
        var par = r.Resultado.Deserialize<List<Seg>>(Json.Opt)!;
        foreach (var x in par) x.Base = Json.CloneList(x.Destino);
        var i = _active;
        _active = -1;
        _segs[i] = par[0];
        _segs.Insert(i + 1, par[1]);
        Reindexar();
        SetActive(i, true);
        AtualizarTodosBlocos();
        Perf.AposQuadro(tq => Perf.Registrar(Perf.Linha("M6", "dividir", t0, tq, null, r.TCoreUs)));
    }

    async Task AbrirDoc(string caminho)
    {
        var t0 = Perf.NowMs;
        _estado.Text = "importando…";
        var r = await _core.Rpc("importar", new { caminho });
        _docId = r.Resultado.GetProperty("doc_id").GetInt64();
        int total = r.Resultado.GetProperty("n_segmentos").GetInt32();
        var todos = new List<Seg>(total);
        for (int de = 0; de < total; de += 2000) todos.AddRange(await _core.Rpc<List<Seg>>("segmentos", new { doc_id = _docId, de, n = 2000 }));
        foreach (var s in todos) s.Base = Json.CloneList(s.Destino);
        var bm = await _core.Rpc<List<Bloco>>("blocos_meta", new { doc_id = _docId });
        _blocos = bm.ToDictionary(b => b.Id);
        _blocoOrdem = bm.Select(b => b.Id).ToList();
        _nome.Text = Path.GetFileName(caminho);
        _active = -1;
        _segs.Clear();
        foreach (var s in todos) _segs.Add(s);
        Reindexar();
        _blocoOrdemObs.Clear();
        foreach (var id in _blocoOrdem) _blocoOrdemObs.Add(id);
        SetActive(0, true);
        Perf.AposQuadro(tq =>
        {
            Perf.Registrar(Perf.Linha("M5", "abrir", t0, tq, _nome.Text));
            _estado.Text = $"pronto · {_segs.Count} segmentos · aberto em {tq - t0:F0} ms";
            Perf.Flush();
        });
    }

    async Task Buscar()
    {
        var modo = (string)_modo.SelectedItem!;
        var texto = modo == "memoria" ? (_active >= 0 ? string.Concat(_segs[_active].Origem.Where(x => !x.IsTag).Select(x => x.V)) : "") : _busca.Text ?? "";
        var t0 = Perf.NowMs;
        var r = await _core.Rpc("buscar", new { modo, texto });
        var lista = r.Resultado.Deserialize<List<ResultadoBusca>>(Json.Opt)!;
        _resultadosTxt.Text = lista.Count == 0 ? "sem resultados" : string.Join("\n\n", lista.Select(x => $"{x.Pontuacao:F2}  {x.Origem}\n    {x.Destino}"));
        _resultados.IsVisible = true;
        Perf.AposQuadro(tq => Perf.Registrar(Perf.Linha("M8", "buscar", t0, tq, modo, r.TCoreUs)));
    }

    // ---------- teclado (tunelamento: antes do controle focado) ----------
    // ---------- instantâneo de estado para os testes funcionais (Ctrl+Alt+F9) ----------
    int _seqEstado;
    static string Marcado(IEnumerable<Inline> v) => string.Concat(v.Select(x => x.IsTag ? "<" + (x.Tipo == "fecha" ? "/" : "") + x.Id + ">" : x.V));
    static int Hash(string t) { uint h = 7; foreach (var c in t) h = unchecked(h * 31 + c); return (int)h; }

    async Task Instantaneo()
    {
        var ativo = _active >= 0 ? _segs[_active] : null;
        var ed = Ed;
        bool visivel = false;
        if (ativo != null && _rows.TryGetValue(ativo, out var row) && row.TranslatePoint(new Point(0, 0), _lista) is { } pt)
            visivel = pt.Y >= -1 && pt.Y + row.Bounds.Height <= _lista.Bounds.Height + 1;
        string Texto(TextBlock tb) => string.Concat((tb.Inlines ?? new InlineCollection()).OfType<Run>().Select(r => r.Text));
        long idAtivo = ativo != null && _blocoSegs.Count > 0 ? ativo.BlocoId : -1;
        var d = new Dictionary<string, object?>
        {
            ["active"] = _active, ["total"] = _segs.Count,
            ["head"] = ed?.AtomIndex ?? -1, ["modal"] = _modal != null, ["ativoVisivel"] = visivel,
            ["editor"] = ed != null ? Marcado(ed.GetInlines()) : "",
            ["previaAtivo"] = _previaTb.TryGetValue(idAtivo, out var tbA) ? Texto(tbA) : null,
            ["previaOutros"] = Hash(string.Join("|", _previaTb.Where(kv => kv.Key != idAtivo).OrderBy(kv => kv.Key).Select(kv => Texto(kv.Value)))),
            ["resultados"] = _resultados.IsVisible ? _resultadosTxt.Text : "",
            ["fila"] = (await _core.Rpc("fila", new { estado = "pendente" })).Resultado.GetInt64(),
            ["contagem"] = (await _core.Rpc("contagem", new { doc_id = _docId })).Resultado.GetInt64(),
            ["versao"] = ativo?.Versao, ["destinoAtivo"] = ativo != null ? Marcado(ativo.Destino) : null,
        };
        Perf.Registrar(new() { ["metrica"] = "ESTADO", ["passo"] = (_seqEstado++).ToString(), ["erro"] = Uri.EscapeDataString(JsonSerializer.Serialize(d)) });
        Perf.Flush();
    }

    void AoTeclar(object? sender, KeyEventArgs e)
    {
        if (e.Key == Key.F9 && e.KeyModifiers.HasFlag(KeyModifiers.Control) && e.KeyModifiers.HasFlag(KeyModifiers.Alt)) { e.Handled = true; _ = Instantaneo(); return; }
        if (_modal != null)
        {
            var k0 = e.Key;
            if (k0 is Key.Enter or Key.Escape or Key.Y or Key.S or Key.N)
            {
                e.Handled = true;
                var sim = k0 is Key.Enter or Key.Y or Key.S;
                _modalBox.IsVisible = false;
                var t = _modal; _modal = null;
                t.SetResult(sim);
            }
            return;
        }
        bool mod = e.KeyModifiers.HasFlag(KeyModifiers.Control) || e.KeyModifiers.HasFlag(KeyModifiers.Meta);
        bool shift = e.KeyModifiers.HasFlag(KeyModifiers.Shift), alt = e.KeyModifiers.HasFlag(KeyModifiers.Alt);
        var foco = this.FocusManager?.GetFocusedElement();
        bool emBusca = foco == _busca || _busca.IsKeyboardFocusWithin;
        if (!mod && !alt && foco is SegmentEditor ed && !ed.ReadOnly && ImprimivelDe(e.Key)) _teclaT = Perf.NowMs;
        var k = e.Key;
        if (mod && k == Key.Enter) { e.Handled = true; _ = ConfirmarSeg(!shift); }
        else if (k == Key.Escape) { _resultados.IsVisible = false; if (!emBusca) { Descartar(); e.Handled = true; } }
        else if (mod && !shift && k == Key.M) { e.Handled = true; _ = Mesclar(); }
        else if (mod && shift && k == Key.D) { e.Handled = true; _ = Dividir(); }
        else if (mod && k == Key.F) { e.Handled = true; _busca.Focus(); }
        else if (k == Key.PageDown && !emBusca) { e.Handled = true; SetActive(Math.Min(_segs.Count - 1, _active + 10)); }
        else if (k == Key.PageUp && !emBusca) { e.Handled = true; SetActive(Math.Max(0, _active - 10)); }
        else if (mod && k == Key.Home) { e.Handled = true; SetActive(0); }
        else if (mod && k == Key.End) { e.Handled = true; SetActive(_segs.Count - 1); }
        else if (alt && k == Key.Down) { e.Handled = true; SetActive(_active + 1); }
        else if (alt && k == Key.Up) { e.Handled = true; SetActive(_active - 1); }
    }

    static bool ImprimivelDe(Key k) => k is >= Key.A and <= Key.Z || k is >= Key.D0 and <= Key.D9 || k is >= Key.NumPad0 and <= Key.NumPad9 || k == Key.Space || (k >= Key.Oem1 && k <= Key.Oem102);
}
