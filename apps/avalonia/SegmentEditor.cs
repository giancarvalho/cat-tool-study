using System.Globalization;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Input.Platform;
using Avalonia.Media;

namespace CatStudy;

public sealed class Atom
{
    public char C;
    public Inline? Tag;
    public bool IsTag => Tag != null;
}

/// <summary>
/// Controle de edição próprio (Emenda 2): texto com tags atômicas desenhadas como chips, quebra de linha, cursor,
/// seleção, teclado (inclusive Ctrl+setas com a tag como limite de palavra), apagar tag com confirmação e mouse.
/// Sem RichTextEditor do fornecedor. Entrada composta (acento morto) chega pelo evento TextInput do sistema.
/// </summary>
public sealed class SegmentEditor : Control
{
    const double FontSize = 14, ChipFontSize = 12, LineH = 22, Pad = 4, ChipPadX = 5, ChipGap = 2;
    static readonly Typeface Face = new(FontFamily.Default);
    static readonly Typeface Mono = new("Consolas");
    static readonly IBrush Fg = new SolidColorBrush(Color.FromRgb(0x11, 0x18, 0x27));
    static readonly IBrush ChipBg = new SolidColorBrush(Color.FromRgb(0xE5, 0xE7, 0xEB));
    static readonly IBrush ChipFg = new SolidColorBrush(Color.FromRgb(0x37, 0x41, 0x51));
    static readonly IBrush SelBg = new SolidColorBrush(Color.FromRgb(0xBF, 0xDB, 0xFE));
    static readonly Pen Borda = new(new SolidColorBrush(Color.FromRgb(0x25, 0x63, 0xEB)), 1.5);
    static readonly Dictionary<char, FormattedText> CacheCh = new();
    static readonly Dictionary<string, FormattedText> CacheChip = new();

    static FormattedText Ft(string s, Typeface f, double size, IBrush b) => new(s, CultureInfo.InvariantCulture, FlowDirection.LeftToRight, f, size, b);
    static FormattedText FtCh(char c) { if (!CacheCh.TryGetValue(c, out var t)) CacheCh[c] = t = Ft(c.ToString(), Face, FontSize, Fg); return t; }
    static FormattedText FtChip(string s) { if (!CacheChip.TryGetValue(s, out var t)) CacheChip[s] = t = Ft(s, Mono, ChipFontSize, ChipFg); return t; }
    static double WidthCh(char c) => c == '\t' ? 4 * FtCh(' ').WidthIncludingTrailingWhitespace : FtCh(c).WidthIncludingTrailingWhitespace;

    List<Atom> _a = new();
    double[] _x = Array.Empty<double>(), _w = Array.Empty<double>();
    int[] _line = Array.Empty<int>();
    int _lines = 1;
    double _layoutW = -1;
    int _caret, _anchor;
    bool _drag;

    public bool ReadOnly { get; set; } = true;
    public event Action<List<Inline>>? Changed;
    public Func<string, Task<bool>>? ConfirmarTag;

    public SegmentEditor() { Focusable = true; ClipToBounds = true; }

    // ---------- modelo ----------
    public void Load(List<Inline> v)
    {
        _a = new List<Atom>();
        foreach (var n in v)
        {
            if (n.IsTag) _a.Add(new Atom { Tag = n.Clone() });
            else foreach (var c in n.V ?? "") _a.Add(new Atom { C = c });
        }
        _caret = _anchor = 0;
        _layoutW = -1;
        InvalidateMeasure();
        InvalidateVisual();
    }

    public List<Inline> GetInlines()
    {
        var o = new List<Inline>();
        var sb = new System.Text.StringBuilder();
        void Flush() { if (sb.Length > 0) { o.Add(Inline.Texto(sb.ToString())); sb.Clear(); } }
        foreach (var x in _a) { if (x.IsTag) { Flush(); o.Add(x.Tag!.Clone()); } else sb.Append(x.C); }
        Flush();
        return o;
    }

    public int AtomIndex => _caret;
    public int Count => _a.Count;
    public int TagCount => _a.Count(x => x.IsTag);
    public void SetCaret(int i) { _caret = _anchor = Math.Clamp(i, 0, _a.Count); InvalidateVisual(); }
    public string TextoPuro => new string(_a.Where(x => !x.IsTag).Select(x => x.C).ToArray());

    // ---------- layout ----------
    double AtomW(Atom a) => a.IsTag ? FtChip(a.Tag!.ChipLabel).Width + 2 * ChipPadX + ChipGap : WidthCh(a.C);

    void Layout(double maxW)
    {
        int n = _a.Count;
        _x = new double[n]; _w = new double[n]; _line = new int[n];
        for (int k = 0; k < n; k++) _w[k] = AtomW(_a[k]);
        double x = 0; int line = 0, i = 0;
        while (i < n)
        {
            int j = i;
            if (_a[i].IsTag) j = i + 1;
            else
            {
                while (j < n && !_a[j].IsTag && _a[j].C != ' ') j++;
                while (j < n && !_a[j].IsTag && _a[j].C == ' ') j++;
                if (j == i) j = i + 1;
            }
            double trimmed = 0;
            int last = j; while (last > i && !_a[last - 1].IsTag && _a[last - 1].C == ' ') last--;
            for (int k = i; k < Math.Max(last, i + 1) && k < j; k++) trimmed += _w[k];
            if (x > 0 && x + trimmed > maxW) { line++; x = 0; }
            for (int k = i; k < j; k++) { _x[k] = x; _line[k] = line; x += _w[k]; }
            i = j;
        }
        _lines = line + 1;
        _layoutW = maxW;
    }

    protected override Size MeasureOverride(Size avail)
    {
        double w = double.IsInfinity(avail.Width) ? 400 : avail.Width;
        Layout(Math.Max(20, w - 2 * Pad));
        return new Size(w, _lines * LineH + 2 * Pad);
    }

    void EnsureLayout() { if (_layoutW < 0) Layout(Math.Max(20, Bounds.Width - 2 * Pad)); }

    (double x, int line) CaretXY(int c)
    {
        EnsureLayout();
        if (_a.Count == 0) return (0, 0);
        if (c < _a.Count) return (_x[c], _line[c]);
        return (_x[^1] + _w[^1], _line[^1]);
    }

    int HitTest(Point p)
    {
        EnsureLayout();
        int line = Math.Clamp((int)Math.Floor((p.Y - Pad) / LineH), 0, _lines - 1);
        double x = p.X - Pad;
        int lastOnLine = -1;
        for (int k = 0; k < _a.Count; k++)
        {
            if (_line[k] != line) { if (_line[k] > line) break; continue; }
            lastOnLine = k;
            if (x < _x[k] + _w[k] / 2) return k;
        }
        return lastOnLine < 0 ? _a.Count : lastOnLine + 1;
    }

    // ---------- desenho ----------
    public override void Render(DrawingContext ctx)
    {
        EnsureLayout();
        if (!ReadOnly) ctx.DrawRectangle(null, Borda, new Rect(Bounds.Size).Deflate(0.75));
        int s = Math.Min(_caret, _anchor), e = Math.Max(_caret, _anchor);
        for (int k = 0; k < _a.Count; k++)
        {
            double px = Pad + _x[k], py = Pad + _line[k] * LineH;
            if (k >= s && k < e) ctx.DrawRectangle(SelBg, null, new Rect(px, py, _w[k], LineH));
            if (_a[k].IsTag)
            {
                var t = FtChip(_a[k].Tag!.ChipLabel);
                var r = new Rect(px + ChipGap / 2, py + 2, _w[k] - ChipGap, LineH - 4);
                ctx.DrawRectangle(ChipBg, null, r, 4, 4);
                ctx.DrawText(t, new Point(r.X + ChipPadX, r.Y + (r.Height - t.Height) / 2));
            }
            else if (_a[k].C != ' ' && _a[k].C != '\t')
            {
                var t = FtCh(_a[k].C);
                ctx.DrawText(t, new Point(px, py + (LineH - t.Height) / 2));
            }
        }
        if (!ReadOnly && IsFocused)
        {
            var (cx, cl) = CaretXY(_caret);
            ctx.DrawLine(new Pen(Fg, 1), new Point(Pad + cx, Pad + cl * LineH + 2), new Point(Pad + cx, Pad + (cl + 1) * LineH - 2));
        }
    }

    protected override void OnGotFocus(FocusChangedEventArgs e) { base.OnGotFocus(e); InvalidateVisual(); }
    protected override void OnLostFocus(FocusChangedEventArgs e) { base.OnLostFocus(e); InvalidateVisual(); }

    // ---------- edição ----------
    void Notify() { _layoutW = -1; InvalidateMeasure(); InvalidateVisual(); Changed?.Invoke(GetInlines()); }
    bool HasSel => _caret != _anchor;
    (int a, int b) Sel => (Math.Min(_caret, _anchor), Math.Max(_caret, _anchor));

    public void InserirTexto(string s)
    {
        if (ReadOnly || string.IsNullOrEmpty(s)) return;
        s = s.Replace("\r", "").Replace("\n", " ");
        if (HasSel)
        {
            var (a0, b0) = Sel;
            if (_a.Skip(a0).Take(b0 - a0).Any(x => x.IsTag)) return; // trocar seleção com tag exige apagar com confirmação antes
            _a.RemoveRange(a0, b0 - a0);
            _caret = a0;
        }
        _a.InsertRange(_caret, s.Select(c => new Atom { C = c }));
        _caret += s.Length;
        _anchor = _caret;
        Notify();
    }

    async Task ApagarIntervalo(int a, int b)
    {
        if (a >= b) return;
        var remover = new SortedSet<int>(Enumerable.Range(a, b - a));
        var tags = remover.Where(i => _a[i].IsTag).ToList();
        if (tags.Count > 0)
        {
            var rot = _a[tags[0]].Tag!.ChipLabel.TrimStart('/');
            if (ConfirmarTag != null && !await ConfirmarTag(rot)) return;
            foreach (var i in tags)
            {
                var t = _a[i].Tag!;
                if (t.Tipo is "abre" or "fecha" or "link")
                    for (int k = 0; k < _a.Count; k++)
                        if (k != i && _a[k].IsTag && _a[k].Tag!.Id == t.Id && _a[k].Tag!.Tipo is "abre" or "fecha" or "link") remover.Add(k);
            }
        }
        foreach (var i in remover.Reverse()) _a.RemoveAt(i);
        _caret = _anchor = Math.Min(a, _a.Count);
        Notify();
    }

    // limites de palavra: a tag é sempre um limite (para antes e depois dela)
    int PalavraDir(int i)
    {
        int n = _a.Count;
        if (i >= n) return n;
        if (_a[i].IsTag) return i + 1;
        while (i < n && !_a[i].IsTag && char.IsWhiteSpace(_a[i].C)) i++;
        if (i < n && _a[i].IsTag) return i;
        while (i < n && !_a[i].IsTag && !char.IsWhiteSpace(_a[i].C)) i++;
        return i;
    }
    int PalavraEsq(int i)
    {
        if (i <= 0) return 0;
        if (_a[i - 1].IsTag) return i - 1;
        while (i > 0 && !_a[i - 1].IsTag && char.IsWhiteSpace(_a[i - 1].C)) i--;
        if (i > 0 && _a[i - 1].IsTag) return i;
        while (i > 0 && !_a[i - 1].IsTag && !char.IsWhiteSpace(_a[i - 1].C)) i--;
        return i;
    }

    int LinhaInicio(int c) { var (_, l) = CaretXY(c); int k = 0; while (k < _a.Count && _line[k] < l) k++; return k; }
    int LinhaFim(int c) { var (_, l) = CaretXY(c); int k = _a.Count; while (k > 0 && _line[k - 1] > l) k--; return k; }

    void Mover(int novo, bool shift)
    {
        _caret = Math.Clamp(novo, 0, _a.Count);
        if (!shift) _anchor = _caret;
        InvalidateVisual();
    }

    protected override void OnTextInput(TextInputEventArgs e)
    {
        base.OnTextInput(e);
        if (ReadOnly || string.IsNullOrEmpty(e.Text) || e.Text.All(char.IsControl)) return;
        InserirTexto(e.Text);
        e.Handled = true;
    }

    protected override async void OnKeyDown(KeyEventArgs e)
    {
        base.OnKeyDown(e);
        if (ReadOnly) return;
        bool ctrl = e.KeyModifiers.HasFlag(KeyModifiers.Control), shift = e.KeyModifiers.HasFlag(KeyModifiers.Shift);
        if (e.KeyModifiers.HasFlag(KeyModifiers.Alt)) return;
        switch (e.Key)
        {
            case Key.Left:
                e.Handled = true;
                if (HasSel && !shift && !ctrl) Mover(Sel.a, false); else Mover(ctrl ? PalavraEsq(_caret) : _caret - 1, shift);
                break;
            case Key.Right:
                e.Handled = true;
                if (HasSel && !shift && !ctrl) Mover(Sel.b, false); else Mover(ctrl ? PalavraDir(_caret) : _caret + 1, shift);
                break;
            case Key.Home when !ctrl: e.Handled = true; Mover(LinhaInicio(_caret), shift); break;
            case Key.End when !ctrl: e.Handled = true; Mover(LinhaFim(_caret), shift); break;
            case Key.Up:
            case Key.Down:
                {
                    e.Handled = true;
                    var (x, l) = CaretXY(_caret);
                    int alvo = l + (e.Key == Key.Up ? -1 : 1);
                    if (alvo < 0) Mover(0, shift);
                    else if (alvo >= _lines) Mover(_a.Count, shift);
                    else Mover(HitTest(new Point(Pad + x, Pad + alvo * LineH + LineH / 2)), shift);
                    break;
                }
            case Key.Back:
                e.Handled = true;
                if (HasSel) await ApagarIntervalo(Sel.a, Sel.b); else if (_caret > 0) await ApagarIntervalo(_caret - 1, _caret);
                break;
            case Key.Delete:
                e.Handled = true;
                if (HasSel) await ApagarIntervalo(Sel.a, Sel.b); else if (_caret < _a.Count) await ApagarIntervalo(_caret, _caret + 1);
                break;
            case Key.A when ctrl: e.Handled = true; _anchor = 0; _caret = _a.Count; InvalidateVisual(); break;
            case Key.C when ctrl:
            case Key.X when ctrl:
                {
                    e.Handled = true;
                    var (a, b) = Sel;
                    var txt = new string(_a.Skip(a).Take(b - a).Where(x => !x.IsTag).Select(x => x.C).ToArray());
                    var cb = TopLevel.GetTopLevel(this)?.Clipboard;
                    if (cb != null && txt.Length > 0) await cb.SetTextAsync(txt);
                    if (e.Key == Key.X && HasSel) await ApagarIntervalo(a, b);
                    break;
                }
            case Key.V when ctrl:
                {
                    e.Handled = true;
                    var cb = TopLevel.GetTopLevel(this)?.Clipboard;
                    var txt = cb == null ? null : await cb.TryGetTextAsync();
                    if (!string.IsNullOrEmpty(txt)) InserirTexto(txt);
                    break;
                }
        }
    }

    protected override void OnPointerPressed(PointerPressedEventArgs e)
    {
        base.OnPointerPressed(e);
        if (ReadOnly) return;
        Focus();
        _caret = HitTest(e.GetPosition(this));
        if (!e.KeyModifiers.HasFlag(KeyModifiers.Shift)) _anchor = _caret;
        _drag = true;
        e.Pointer.Capture(this);
        InvalidateVisual();
    }
    protected override void OnPointerMoved(PointerEventArgs e)
    {
        base.OnPointerMoved(e);
        if (!_drag) return;
        _caret = HitTest(e.GetPosition(this));
        InvalidateVisual();
    }
    protected override void OnPointerReleased(PointerReleasedEventArgs e)
    {
        base.OnPointerReleased(e);
        _drag = false;
        e.Pointer.Capture(null);
    }
}
