using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Themes.Fluent;

namespace CatStudy;

public sealed class App : Application
{
    public override void Initialize() => Styles.Add(new FluentTheme());

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime d)
            d.MainWindow = new MainWindow(Environment.GetCommandLineArgs());
        base.OnFrameworkInitializationCompleted();
    }
}

public static class Program
{
    [STAThread]
    public static int Main(string[] args) =>
        Construir().StartWithClassicDesktopLifetime(args);

    static AppBuilder Construir()
    {
        var b = AppBuilder.Configure<App>().UsePlatformDetect();
        // Emenda 6: swap chain DXGI própria (a composição padrão apresenta pelo dwm.exe e o PresentMon não atribui os quadros ao app).
        return b.With(new Win32PlatformOptions { CompositionMode = new[] { Win32CompositionMode.LowLatencyDxgiSwapChain } });
    }
}
