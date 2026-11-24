//! Memória da árvore de processos (M7, M9), perfil Restrito (Job Object) e autoteste do SendInput.
use super::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};

// ---------- memória ----------
#[repr(C)]
struct ProcessEntry {
    size: u32,
    usage: u32,
    pid: u32,
    heap: usize,
    module: u32,
    threads: u32,
    parent: u32,
    prio: i32,
    flags: u32,
    exe: [u16; 260],
}
#[repr(C)]
struct MemCounters {
    cb: u32,
    page_faults: u32,
    peak_ws: usize,
    ws: usize,
    q1: usize,
    q2: usize,
    q3: usize,
    q4: usize,
    pagefile: usize,
    peak_pagefile: usize,
    private: usize,
}
#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
    fn Process32FirstW(s: isize, e: *mut ProcessEntry) -> i32;
    fn Process32NextW(s: isize, e: *mut ProcessEntry) -> i32;
    fn OpenProcess(access: u32, inherit: i32, pid: u32) -> isize;
    fn CloseHandle(h: isize) -> i32;
    fn K32GetProcessMemoryInfo(h: isize, c: *mut MemCounters, cb: u32) -> i32;
    fn CreateJobObjectW(attr: usize, name: usize) -> isize;
    fn SetInformationJobObject(job: isize, class: i32, info: *const JobExtended, cb: u32) -> i32;
    fn AssignProcessToJobObject(job: isize, proc_: isize) -> i32;
}

pub static CHILD_PID: AtomicU32 = AtomicU32::new(0);
pub static MEM_LOG: OnceLock<Mutex<std::fs::File>> = OnceLock::new();

/// Soma, por toda a árvore de processos do app: PrivateUsage (definição do pré-registro, seção 6) e WorkingSet.
pub fn memoria_arvore(raiz: u32) -> (f64, f64, usize) {
    unsafe {
        let snap = CreateToolhelp32Snapshot(2, 0);
        if snap == -1 {
            return (0.0, 0.0, 0);
        }
        let mut e: ProcessEntry = std::mem::zeroed();
        e.size = std::mem::size_of::<ProcessEntry>() as u32;
        let mut todos: Vec<(u32, u32)> = vec![];
        let mut ok = Process32FirstW(snap, &mut e);
        while ok != 0 {
            todos.push((e.pid, e.parent));
            ok = Process32NextW(snap, &mut e);
        }
        CloseHandle(snap);
        let mut arvore = vec![raiz];
        let mut i = 0;
        while i < arvore.len() {
            let pai = arvore[i];
            for (pid, par) in &todos {
                if *par == pai && !arvore.contains(pid) {
                    arvore.push(*pid);
                }
            }
            i += 1;
        }
        let (mut priv_b, mut ws_b, mut n) = (0usize, 0usize, 0usize);
        for pid in arvore {
            let h = OpenProcess(0x1000, 0, pid);
            if h == 0 {
                continue;
            }
            let mut c: MemCounters = std::mem::zeroed();
            c.cb = std::mem::size_of::<MemCounters>() as u32;
            if K32GetProcessMemoryInfo(h, &mut c, c.cb) != 0 {
                priv_b += c.private;
                ws_b += c.ws;
                n += 1;
            }
            CloseHandle(h);
        }
        (priv_b as f64 / 1048576.0, ws_b as f64 / 1048576.0, n)
    }
}

pub fn registrar_memoria(rotulo: &str) {
    let pid = CHILD_PID.load(Ordering::SeqCst);
    let (p, w, n) = memoria_arvore(pid);
    if let Some(f) = MEM_LOG.get() {
        let _ = writeln!(f.lock().unwrap(), "{},{},{:.2},{:.2},{}", agora_us(), rotulo, p, w, n);
    }
}

// ---------- perfil Restrito ----------
#[repr(C)]
struct JobBasic {
    per_process_time: i64,
    per_job_time: i64,
    flags: u32,
    min_ws: usize,
    max_ws: usize,
    active_limit: u32,
    affinity: usize,
    priority: u32,
    scheduling: u32,
}
#[repr(C)]
struct JobExtended {
    basic: JobBasic,
    io: [u64; 6],
    process_mem: usize,
    job_mem: usize,
    peak_process: usize,
    peak_job: usize,
}

/// Afinidade de CPU (máscara) e limite de memória do job (bytes) para o app e todos os seus filhos.
pub fn limitar(pid: u32, mascara: usize, bytes: usize) -> Result<()> {
    unsafe {
        let job = CreateJobObjectW(0, 0);
        let mut i: JobExtended = std::mem::zeroed();
        i.basic.flags = 0x10 | 0x200; // AFFINITY | JOB_MEMORY
        i.basic.affinity = mascara;
        i.job_mem = bytes;
        if SetInformationJobObject(job, 9, &i, std::mem::size_of::<JobExtended>() as u32) == 0 {
            bail!("SetInformationJobObject falhou");
        }
        let h = OpenProcess(0x0100 | 0x0001, 0, pid); // SET_QUOTA | TERMINATE
        if h == 0 || AssignProcessToJobObject(job, h) == 0 {
            bail!("não consegui colocar o app no Job Object");
        }
    }
    Ok(())
}

// ---------- autoteste: piso do SendInput numa janela própria, sem casca ----------
#[repr(C)]
struct WndClass {
    style: u32,
    proc_: extern "system" fn(isize, u32, usize, isize) -> isize,
    cls_extra: i32,
    wnd_extra: i32,
    instance: isize,
    icon: isize,
    cursor: isize,
    brush: isize,
    menu: usize,
    class: *const u16,
}
#[repr(C)]
struct Msg {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: [i32; 2],
    private: u32,
}
#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(c: *const WndClass) -> u16;
    fn CreateWindowExW(ex: u32, class: *const u16, title: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: isize, menu: isize, inst: isize, p: usize) -> isize;
    fn ShowWindow(h: isize, cmd: i32) -> i32;
    fn GetMessageW(m: *mut Msg, h: isize, a: u32, b: u32) -> i32;
    fn DispatchMessageW(m: *const Msg) -> isize;
    fn DefWindowProcW(h: isize, m: u32, w: usize, l: isize) -> isize;
    fn PostQuitMessage(c: i32);
    fn PostMessageW(h: isize, m: u32, w: usize, l: isize) -> i32;
}
static RECEBIDOS: Mutex<Vec<u64>> = Mutex::new(Vec::new());

extern "system" fn proc_autoteste(h: isize, m: u32, w: usize, l: isize) -> isize {
    match m {
        0x100 if w == 0x41 => {
            RECEBIDOS.lock().unwrap().push(agora_us());
            0
        }
        0x2 => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(h, m, w, l) },
    }
}

fn percentil(v: &[f64], q: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() - 1) as f64 * q).round() as usize]
}

pub fn autoteste(n: usize, saida: &str) -> Result<()> {
    unsafe { timeBeginPeriod(1) };
    let classe: Vec<u16> = "CatAutoteste\0".encode_utf16().collect();
    let titulo: Vec<u16> = "autoteste\0".encode_utf16().collect();
    let hwnd = unsafe {
        let c = WndClass { style: 0, proc_: proc_autoteste, cls_extra: 0, wnd_extra: 0, instance: 0, icon: 0, cursor: 0, brush: 6, menu: 0, class: classe.as_ptr() };
        RegisterClassW(&c);
        CreateWindowExW(0, classe.as_ptr(), titulo.as_ptr(), 0x10CF0000, 100, 100, 400, 200, 0, 0, 0, 0)
    };
    if hwnd == 0 {
        bail!("não criei a janela do autoteste");
    }
    unsafe { ShowWindow(hwnd, 5) };
    let envios = std::sync::Arc::new(Mutex::new(Vec::<u64>::new()));
    let e2 = envios.clone();
    let th = std::thread::spawn(move || {
        esperar(500);
        for _ in 0..50 {
            unsafe {
                keybd_event(0x12, 0, 0, 0);
                keybd_event(0x12, 0, 2, 0);
                SetForegroundWindow(hwnd);
                if GetForegroundWindow() == hwnd {
                    break;
                }
            }
            esperar(100);
        }
        esperar(500);
        for _ in 0..n {
            let t = caractere('a');
            e2.lock().unwrap().push(t);
            esperar(50);
        }
        esperar(300);
        unsafe { PostMessageW(hwnd, 0x10, 0, 0) }; // WM_CLOSE
    });
    let mut m: Msg = unsafe { std::mem::zeroed() };
    unsafe {
        while GetMessageW(&mut m, 0, 0, 0) > 0 {
            DispatchMessageW(&m);
        }
    }
    th.join().ok();
    let env = envios.lock().unwrap().clone();
    let rec = RECEBIDOS.lock().unwrap().clone();
    // pareia cada mensagem recebida com o último envio anterior a ela
    let mut pares: Vec<(u64, u64)> = vec![];
    for r in &rec {
        if let Some(e) = env.iter().rev().find(|e| **e <= *r) {
            pares.push((*e, *r));
        }
    }
    let d: Vec<f64> = pares.iter().map(|(e, r)| (*r as f64 - *e as f64) / 1000.0).collect();
    if d.is_empty() {
        bail!("nenhuma tecla chegou à janela (foco?)");
    }
    let mut f = std::fs::File::create(saida)?;
    writeln!(f, "indice,t_envio_us,t_recebe_us,atraso_ms")?;
    for (i, ((e, r), x)) in pares.iter().map(|(e, r)| (e, r)).zip(d.iter()).enumerate().map(|(i, (er, x))| (i, (er, x))) {
        writeln!(f, "{},{},{},{:.3}", i, e, r, x)?;
    }
    println!(
        "autoteste: enviadas {}, recebidas {}; atraso SendInput->mensagem: min {:.3} p50 {:.3} p95 {:.3} max {:.3} ms",
        env.len(),
        rec.len(),
        percentil(&d, 0.0),
        percentil(&d, 0.5),
        percentil(&d, 0.95),
        percentil(&d, 1.0)
    );
    Ok(())
}
