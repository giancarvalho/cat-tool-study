//! Processo `catcore`: JSON-RPC por stdin/stdout, uma mensagem por linha (spec §5).
//! Requisição:  {"id":1,"metodo":"importar","params":{"caminho":"..."}}
//! Resposta:    {"id":1,"ok":true,"resultado":...,"t_core_us":123}  ou  {"id":1,"ok":false,"erro":"..."}
use catcore::model::Inline;
use catcore::Core;
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::time::Instant;

fn handle(core: &mut Core, metodo: &str, p: &Value) -> anyhow::Result<Value> {
    let i = |k: &str| p.get(k).and_then(|v| v.as_i64()).ok_or_else(|| anyhow::anyhow!("parâmetro {} ausente", k));
    let s = |k: &str| p.get(k).and_then(|v| v.as_str()).map(|x| x.to_string()).ok_or_else(|| anyhow::anyhow!("parâmetro {} ausente", k));
    Ok(match metodo {
        "importar" => core.importar(&s("caminho")?)?,
        "contagem" => json!(core.contagem(i("doc_id")?)?),
        "segmentos" => serde_json::to_value(core.segmentos(i("doc_id")?, i("de")?, i("n")?)?)?,
        "segmento" => serde_json::to_value(core.segmento(i("id")?)?)?,
        "blocos_do_doc" => json!(core.blocos_do_doc(i("doc_id")?)?),
        "blocos_meta" => json!(core.blocos_meta(i("doc_id")?)?),
        "blocos" => {
            let ids: Vec<i64> = serde_json::from_value(p.get("ids").cloned().unwrap_or(Value::Null))?;
            serde_json::to_value(core.blocos(&ids)?)?
        }
        "salvar_segmento" => {
            let d: Vec<Inline> = serde_json::from_value(p.get("destino").cloned().unwrap_or(Value::Null))?;
            core.salvar_segmento(i("id")?, d)?
        }
        "mesclar" => serde_json::to_value(core.mesclar(i("id")?)?)?,
        "dividir" => serde_json::to_value(core.dividir(i("id")?, i("pos")? as usize)?)?,
        "buscar" => serde_json::to_value(core.buscar(&s("modo")?, &s("texto")?)?)?,
        "tm_sintetico" => json!(core.tm_sintetico(i("n")? as usize, i("semente")? as u64)?),
        "tm_aquecer" => json!(core.tm_aquecer()?),
        "fila" => json!(core.fila(&s("estado")?)?),
        "sair" => std::process::exit(0),
        m => return Err(anyhow::anyhow!("método desconhecido: {}", m)),
    })
}

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
        let params = req.get("params").cloned().unwrap_or(json!({}));
        let t0 = Instant::now();
        let r = handle(&mut core, metodo, &params);
        let t = t0.elapsed().as_micros() as u64;
        let resp = match r {
            Ok(v) => json!({"id": id, "ok": true, "resultado": v, "t_core_us": t}),
            Err(e) => json!({"id": id, "ok": false, "erro": e.to_string(), "t_core_us": t}),
        };
        writeln!(out, "{}", resp)?;
        out.flush()?;
    }
    Ok(())
}
