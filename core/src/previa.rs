//! Prévia compilada (Emenda 5, M11): exporta o documento com os destinos, converte uma página em PNG com o
//! LibreOffice (motor externo comum aos quatro candidatos) e devolve o caminho da imagem e os tempos de cada etapa.
use crate::core::Core;
use anyhow::{anyhow, bail, Result};
use serde_json::{json, Value};
use std::process::Command;
use std::time::Instant;

fn soffice() -> String {
    std::env::var("SOFFICE").unwrap_or_else(|_| r"C:\Program Files\LibreOffice\program\soffice.exe".to_string())
}

impl Core {
    /// `pagina` conta a partir de 1 (slide, no PPTX). Sem cache: cada pedido exporta e converte de novo.
    pub fn previa_compilada(&self, doc_id: i64, pagina: u32) -> Result<Value> {
        let tipo: String = self
            .db
            .query_row("SELECT tipo FROM documento WHERE id=?", [doc_id], |r| r.get(0))
            .map_err(|_| anyhow!("documento {} inexistente", doc_id))?;
        let pasta = std::env::temp_dir().join("catstudy-previa");
        std::fs::create_dir_all(&pasta)?;
        let ext = if tipo == "pptx" { "pptx" } else { "docx" };
        let origem = pasta.join(format!("previa.{}", ext));
        let saida_png = pasta.join("png");
        let _ = std::fs::remove_dir_all(&saida_png);

        let t0 = Instant::now();
        let ex = self.exportar(doc_id, origem.to_str().unwrap())?;
        let t_export = t0.elapsed();

        let t1 = Instant::now();
        let filtro = if ext == "pptx" { "impress_png_Export" } else { "writer_png_Export" };
        let opcoes = format!(r#"png:{}:{{"PageRange":{{"type":"string","value":"{}"}}}}"#, filtro, pagina.max(1));
        let mut cmd = Command::new(soffice());
        cmd.arg(format!("-env:UserInstallation=file:///{}", pasta.join("perfil").to_string_lossy().replace('\\', "/")))
            .args(["--headless", "--convert-to", &opcoes, "--outdir"])
            .arg(&saida_png)
            .arg(&origem);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000); // sem janela de console
        }
        let r = cmd.output()?;
        let t_render = t1.elapsed();
        let png = saida_png.join("previa.png");
        if !r.status.success() || !png.exists() {
            bail!("LibreOffice não gerou a página {}: {}", pagina, String::from_utf8_lossy(&r.stderr).trim());
        }
        Ok(json!({
            "png": png.to_string_lossy(),
            "pagina": pagina,
            "t_export_us": t_export.as_micros() as i64,
            "t_render_us": t_render.as_micros() as i64,
            "blocos_alterados": ex["blocos_alterados"],
        }))
    }
}
