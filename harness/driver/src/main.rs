//! Driver de teclas do sistema operacional (Windows, SendInput). Spec/SCRIPT.md §1.
//! Uso: catdriver --roteiro spec/roteiros/curto.json --saida results/x/driver.csv --espera-csv app.csv -- <app.exe> <args...>
//! Relógio comum com o app: tempo de parede UTC em microssegundos (GetSystemTimePreciseAsFileTime);
//! no app, performance.timeOrigin + performance.now().
use anyhow::{anyhow, bail, Result};
use serde_json::Value;
use std::io::Write;
use std::process::Command;
use std::time::{Duration, Instant};

mod extra;
use extra::{autoteste, limitar, registrar_memoria, CHILD_PID, MEM_LOG};
use std::sync::atomic::Ordering;
use std::sync::Mutex;

#[repr(C)]
struct KeybdInput {
    w_vk: u16,
    w_scan: u16,
    flags: u32,
    time: u32,
    extra: usize,
}
#[repr(C)]
struct Input {
    kind: u32,
    _pad: u32,
    ki: KeybdInput,
    _tail: [u8; 8],
}

#[link(name = "user32")]
extern "system" {
    fn SendInput(n: u32, p: *const Input, size: i32) -> u32;
    fn VkKeyScanW(c: u16) -> i16;
    fn MapVirtualKeyW(code: u32, map: u32) -> u32;
    fn EnumWindows(cb: extern "system" fn(isize, isize) -> i32, lp: isize) -> i32;
    fn GetWindowThreadProcessId(h: isize, pid: *mut u32) -> u32;
    fn IsWindowVisible(h: isize) -> i32;
    fn SetForegroundWindow(h: isize) -> i32;
    fn GetForegroundWindow() -> isize;
    fn keybd_event(vk: u8, scan: u8, flags: u32, extra: usize);
}
#[link(name = "kernel32")]
extern "system" {
    fn GetSystemTimePreciseAsFileTime(ft: *mut u64);
}
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(ms: u32) -> u32;
}

const KEYUP: u32 = 2;
const UNICODE: u32 = 4;
const VK_SHIFT: u16 = 0x10;
const VK_CTRL: u16 = 0x11;
const VK_ALT: u16 = 0x12;

fn agora_us() -> u64 {
    let mut ft = 0u64;
    unsafe { GetSystemTimePreciseAsFileTime(&mut ft) };
    (ft - 116_444_736_000_000_000) / 10
}

fn enviar(vk: u16, scan: u16, flags: u32) {
    let i = Input { kind: 1, _pad: 0, ki: KeybdInput { w_vk: vk, w_scan: scan, flags, time: 0, extra: 0 }, _tail: [0; 8] };
    unsafe { SendInput(1, &i, std::mem::size_of::<Input>() as i32) };
}

fn evento(vk: u16, soltar: bool) {
    let scan = unsafe { MapVirtualKeyW(vk as u32, 0) } as u16;
    // PgUp..Down, Insert e Delete são teclas estendidas (KEYEVENTF_EXTENDEDKEY = 1)
    let ext = matches!(vk, 0x21..=0x28 | 0x2d | 0x2e) as u32;
    enviar(vk, scan, ext | if soltar { KEYUP } else { 0 });
}

fn vk_nome(n: &str) -> Result<u16> {
    Ok(match n.to_lowercase().as_str() {
        "enter" => 0x0d,
        "esc" | "escape" => 0x1b,
        "home" => 0x24,
        "end" => 0x23,
        "pgup" => 0x21,
        "pgdn" => 0x22,
        "left" => 0x25,
        "up" => 0x26,
        "right" => 0x27,
        "down" => 0x28,
        "backspace" => 0x08,
        "delete" => 0x2e,
        "tab" => 0x09,
        s if s.len() == 1 => s.to_uppercase().as_bytes()[0] as u16,
        s => bail!("tecla desconhecida: {}", s),
    })
}

struct Log {
    f: std::fs::File,
    run: String,
}
impl Log {
    fn linha(&mut self, passo: &str, indice: usize, tecla: &str, t: u64) {
        let _ = writeln!(self.f, "{},{},{},{},{}", self.run, passo, indice, tecla, t);
    }
}

/// Envia "ctrl+shift+d" etc. Devolve o instante do keydown da tecla final.
fn combo(c: &str) -> Result<u64> {
    let partes: Vec<&str> = c.split('+').collect();
    let (mods, tecla) = partes.split_at(partes.len() - 1);
    let mv: Vec<u16> = mods
        .iter()
        .map(|m| match *m {
            "ctrl" => VK_CTRL,
            "shift" => VK_SHIFT,
            "alt" => VK_ALT,
            _ => 0,
        })
        .collect();
    for m in &mv {
        evento(*m, false);
        std::thread::sleep(Duration::from_millis(5));
    }
    let vk = vk_nome(tecla[0])?;
    let t = agora_us();
    evento(vk, false);
    std::thread::sleep(Duration::from_millis(15));
    evento(vk, true);
    for m in mv.iter().rev() {
        evento(*m, true);
    }
    Ok(t)
}

fn caractere(ch: char) -> u64 {
    let r = unsafe { VkKeyScanW(ch as u16) };
    if r == -1 {
        // sem tecla no layout: envia como Unicode
        let t = agora_us();
        enviar(0, ch as u16, UNICODE);
        enviar(0, ch as u16, UNICODE | KEYUP);
        return t;
    }
    let vk = (r & 0xff) as u16;
    let shift = (r >> 8) & 1 != 0;
    if shift {
        evento(VK_SHIFT, false);
    }
    let t = agora_us();
    evento(vk, false);
    std::thread::sleep(Duration::from_millis(10));
    evento(vk, true);
    if shift {
        evento(VK_SHIFT, true);
    }
    t
}

fn esperar(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

fn executar(passos: &[Value], log: &mut Log, cont: &mut usize) -> Result<()> {
    for p in passos {
        let tipo = p["tipo"].as_str().ok_or_else(|| anyhow!("passo sem tipo"))?;
        let atraso = p["atraso_ms"].as_u64().unwrap_or(80);
        let espera = |d: u64| p["espera_ms"].as_u64().unwrap_or(d);
        match tipo {
            "abrir" => {} // o app abre o documento ao iniciar (--doc); M5 vem do app
            "repetir" => {
                for _ in 0..p["vezes"].as_u64().unwrap_or(1) {
                    executar(p["passos"].as_array().ok_or_else(|| anyhow!("repetir sem passos"))?, log, cont)?;
                }
            }
            "digitar" => {
                for ch in p["texto"].as_str().unwrap_or("").chars() {
                    let t = caractere(ch);
                    log.linha("digitar", *cont, &ch.to_string(), t);
                    *cont += 1;
                    esperar(atraso);
                }
            }
            "confirmar" => {
                let t = combo("ctrl+enter")?;
                log.linha("confirmar", *cont, "ctrl+enter", t);
                *cont += 1;
                esperar(espera(400));
            }
            "mesclar" => {
                let t = combo("ctrl+m")?;
                log.linha("mesclar", *cont, "ctrl+m", t);
                *cont += 1;
                esperar(espera(500));
            }
            "dividir" => {
                combo("home")?;
                for _ in 0..p["posicao"].as_u64().unwrap_or(10) {
                    combo("right")?;
                    esperar(10);
                }
                let t = combo("ctrl+shift+d")?;
                log.linha("dividir", *cont, "ctrl+shift+d", t);
                *cont += 1;
                esperar(espera(500));
            }
            "rolar" => {
                let tecla = if p["direcao"].as_str() == Some("cima") { "pgup" } else { "pgdn" };
                for _ in 0..p["quadros"].as_u64().unwrap_or(10) {
                    let t = combo(tecla)?;
                    log.linha("rolar", *cont, tecla, t);
                    *cont += 1;
                    esperar(atraso);
                }
            }
            "buscar" => {
                combo("ctrl+f")?;
                esperar(60);
                combo("ctrl+a")?;
                esperar(30);
                for ch in p["texto"].as_str().unwrap_or("").chars() {
                    caractere(ch);
                    esperar(30);
                }
                let t = combo("enter")?;
                log.linha("buscar", *cont, "enter", t);
                *cont += 1;
                esperar(espera(500));
            }
            "tecla" => {
                let c = p["combo"].as_str().unwrap_or("");
                let t = combo(c)?;
                log.linha("tecla", *cont, c, t);
                *cont += 1;
                esperar(atraso);
            }
            "esperar" => esperar(p["ms"].as_u64().unwrap_or(100)),
            "amostrar_memoria" => registrar_memoria(p["rotulo"].as_str().unwrap_or("passo")),
            t => bail!("tipo de passo desconhecido: {}", t),
        }
    }
    Ok(())
}

static mut ALVO_PID: u32 = 0;
static mut ALVO_HWND: isize = 0;
extern "system" fn cb(h: isize, _: isize) -> i32 {
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(h, &mut pid);
        if pid == ALVO_PID && IsWindowVisible(h) != 0 {
            ALVO_HWND = h;
            return 0;
        }
    }
    1
}

fn focar(pid: u32) -> Result<()> {
    unsafe {
        ALVO_PID = pid;
        ALVO_HWND = 0;
        EnumWindows(cb, 0);
    }
    let h = unsafe { ALVO_HWND };
    if h == 0 {
        bail!("janela do app não encontrada");
    }
    unsafe {
        keybd_event(0x12, 0, 0, 0); // truque para liberar SetForegroundWindow
        keybd_event(0x12, 0, 2, 0);
        SetForegroundWindow(h);
    }
    esperar(300);
    if unsafe { GetForegroundWindow() } != h {
        bail!("não consegui colocar a janela do app em primeiro plano");
    }
    Ok(())
}

fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    let v = |n: &str| a.iter().position(|x| x == n).and_then(|i| a.get(i + 1)).cloned();
    if let Some(n) = v("--autoteste") {
        return autoteste(n.parse()?, &v("--saida").ok_or_else(|| anyhow!("--saida ausente"))?);
    }
    let roteiro = v("--roteiro").ok_or_else(|| anyhow!("--roteiro ausente"))?;
    let saida = v("--saida").ok_or_else(|| anyhow!("--saida ausente"))?;
    let esp_csv = v("--espera-csv"); // CSV do app: espera aparecer a linha M5 (abrir)
    let sep = a.iter().position(|x| x == "--").ok_or_else(|| anyhow!("faltou '-- <app> <args>'"))?;
    let cmd = &a[sep + 1..];
    unsafe { timeBeginPeriod(1) };
    let script: Value = serde_json::from_str(&std::fs::read_to_string(&roteiro)?)?;
    if let Some(d) = std::path::Path::new(&saida).parent() {
        std::fs::create_dir_all(d)?;
    }
    let mut f = std::fs::File::create(&saida)?;
    writeln!(f, "run_id,passo,indice,tecla,t_envio_us")?;
    let t_spawn = agora_us();
    let mut child = Command::new(&cmd[0]).args(&cmd[1..]).spawn()?;
    CHILD_PID.store(child.id(), Ordering::SeqCst);
    if v("--perfil").as_deref() == Some("restrito") {
        limitar(child.id(), 0x0F, 8usize << 30)?; // 4 primeiros núcleos lógicos (P-cores) e 8 GB
    }
    let mut mf = std::fs::File::create(std::path::Path::new(&saida).with_file_name("memoria.csv"))?;
    writeln!(mf, "t_us,rotulo,privada_mb,ws_mb,n_processos")?;
    let _ = MEM_LOG.set(Mutex::new(mf));
    if let Some(s) = v("--memoria-cada") {
        let seg: u64 = s.parse()?;
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(seg));
            registrar_memoria("auto");
        });
    }
    let mut log = Log { f, run: child.id().to_string() };
    let t0 = Instant::now();
    if let Some(csv) = esp_csv {
        loop {
            if std::fs::read_to_string(&csv).map(|s| s.contains(",M5,")).unwrap_or(false) {
                break;
            }
            if t0.elapsed() > Duration::from_secs(60) {
                let _ = child.kill();
                bail!("app não abriu o documento em 60 s");
            }
            esperar(100);
        }
    } else {
        esperar(5000);
    }
    esperar(800);
    registrar_memoria("aberto");
    focar(child.id())?;
    log.linha("spawn", 0, "", t_spawn);
    log.linha("inicio", 0, "", agora_us());
    let mut cont = 0usize;
    let r = executar(script["passos"].as_array().ok_or_else(|| anyhow!("roteiro sem passos"))?, &mut log, &mut cont);
    log.linha("fim", cont, "", agora_us());
    registrar_memoria("fim");
    esperar(1000);
    let _ = child.kill();
    r?;
    println!("driver: {} eventos de tecla registrados em {}", cont, saida);
    Ok(())
}
