//! Despacho dos métodos do núcleo (spec §5), compartilhado pelo processo `catcore` (JSON-RPC por stdin/stdout)
//! e pelo Tauri (em processo).
use crate::model::Inline;
use crate::Core;
use serde_json::{json, Value};
use std::time::Instant;

pub fn handle(core: &mut Core, metodo: &str, p: &Value) -> anyhow::Result<Value> {
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
        "tm_carregar_tmx" => {
            let pares = crate::tmx::parse_pasta(&s("pasta")?, &s("origem")?, &s("destino")?)?;
            json!({"pares": pares.len(), "total": core.tm_inserir(&pares)?})
        }
        "previa_compilada" => core.previa_compilada(i("doc_id")?, p["pagina"].as_u64().unwrap_or(1) as u32)?,
        "exportar" => core.exportar(i("doc_id")?, &s("saida")?)?,
        "tm_aquecer" => json!(core.tm_aquecer()?),
        "fila" => json!(core.fila(&s("estado")?)?),
        m => return Err(anyhow::anyhow!("método desconhecido: {}", m)),
    })
}

/// Executa um método e devolve a resposta no formato do protocolo: {ok, resultado|erro, t_core_us}.
pub fn responder(core: &mut Core, metodo: &str, params: &Value) -> Value {
    let t0 = Instant::now();
    let r = handle(core, metodo, params);
    let t = t0.elapsed().as_micros() as u64;
    match r {
        Ok(v) => json!({"ok": true, "resultado": v, "t_core_us": t}),
        Err(e) => json!({"ok": false, "erro": e.to_string(), "t_core_us": t}),
    }
}
