//! Processo `catcore`: JSON-RPC por stdin/stdout, uma mensagem por linha (spec §5).
//! Requisição:  {"id":1,"metodo":"importar","params":{"caminho":"..."}}
//! Resposta:    {"id":1,"ok":true,"resultado":...,"t_core_us":123}  ou  {"id":1,"ok":false,"erro":"..."}
use catcore::rpc::responder;
use catcore::Core;
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let banco = args.iter().position(|a| a == "--banco").and_then(|k| args.get(k + 1)).cloned().unwrap_or_else(|| ":memory:".into());
    let mut core = Core::abrir(&banco)?;
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                writeln!(out, "{}", json!({"id": null, "ok": false, "erro": e.to_string()}))?;
                out.flush()?;
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let metodo = req.get("metodo").and_then(|v| v.as_str()).unwrap_or("");
        if metodo == "sair" {
            std::process::exit(0);
        }
        let params = req.get("params").cloned().unwrap_or(json!({}));
        let mut resp = responder(&mut core, metodo, &params);
        resp["id"] = id;
        writeln!(out, "{}", resp)?;
        out.flush()?;
    }
    Ok(())
}
