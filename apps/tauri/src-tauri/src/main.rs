// Candidato Tauri 2: o núcleo compartilhado roda em processo (sem IPC por processo filho), exposto por comandos.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use catcore::rpc::responder;
use catcore::Core;
use serde_json::{json, Value};
use std::io::Write;
use std::sync::Mutex;
use tauri::{Manager, State};

struct Estado {
    core: Mutex<Core>,
    args: Value,
    out: Option<String>,
}

fn args_do_processo() -> (Value, Option<String>) {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let val = |n: &str| a.iter().position(|x| x == n).and_then(|i| a.get(i + 1)).cloned();
    let out = val("--out");
    (
        json!({
            "teste": a.iter().any(|x| x == "--teste"),
            "doc": val("--doc"),
            "script": val("--script"),
            "out": out,
            "tm": val("--tm-sintetico").and_then(|v| v.parse::<i64>().ok()),
            "tmx": val("--tmx"),
        }),
        out,
    )
}

// Comandos assíncronos: rodam fora da thread principal da interface.
#[tauri::command]
async fn rpc(estado: State<'_, Estado>, metodo: String, params: Value) -> Result<Value, String> {
    let mut core = estado.core.lock().map_err(|e| e.to_string())?;
    Ok(responder(&mut core, &metodo, &params))
}

#[tauri::command]
async fn args(estado: State<'_, Estado>) -> Result<Value, String> {
    Ok(estado.args.clone())
}

#[tauri::command]
async fn abrir() -> Result<Value, String> {
    Ok(Value::Null) // seletor de arquivo (plugin de diálogo) fica para quando for preciso; os roteiros usam --doc
}

const COLS: [&str; 11] = ["run_id", "app", "metrica", "passo", "t_recebe_us", "t_editor_us", "t_previa_us", "t_core_us", "valor", "unidade", "erro"];

#[tauri::command]
async fn log(estado: State<'_, Estado>, rows: Vec<Value>) -> Result<(), String> {
    let Some(caminho) = &estado.out else { return Ok(()) };
    let novo = !std::path::Path::new(caminho).exists();
    if let Some(d) = std::path::Path::new(caminho).parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(caminho).map_err(|e| e.to_string())?;
    if novo {
        writeln!(f, "{}", COLS.join(",")).map_err(|e| e.to_string())?;
    }
    let run = std::process::id().to_string();
    for r in rows {
        let cel = |c: &str| -> String {
            match c {
                "app" => "tauri".into(),
                "run_id" => run.clone(),
                _ => match r.get(c) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Null) | None => String::new(),
                    Some(v) => v.to_string(),
                },
            }
        };
        writeln!(f, "{}", COLS.iter().map(|c| cel(c)).collect::<Vec<_>>().join(",")).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn main() {
    let (args_v, out) = args_do_processo();
    let a: Vec<String> = std::env::args().collect();
    tauri::Builder::default()
        .setup(move |app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let banco = dir.join("catstudy.db");
            if a.iter().any(|x| x == "--teste" || x == "--limpo") {
                for suf in ["", "-wal", "-shm"] {
                    let _ = std::fs::remove_file(format!("{}{}", banco.display(), suf));
                }
            }
            let mut core = Core::abrir(banco.to_str().unwrap())?;
            if let Some(n) = args_v["tm"].as_i64() {
                core.tm_sintetico(n as usize, 42)?;
                core.tm_aquecer()?;
            }
            if let Some(pasta) = args_v["tmx"].as_str() {
                let pares = catcore::tmx::parse_pasta(pasta, "EN-GB", "PT-PT")?;
                core.tm_inserir(&pares)?;
                core.tm_aquecer()?;
            }
            app.manage(Estado { core: Mutex::new(core), args: args_v.clone(), out: out.clone() });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![rpc, args, abrir, log])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Tauri");
}
