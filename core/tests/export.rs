//! Exportação (Emenda 5): sem edição o arquivo é idêntico por parte; com edição só o texto alterado muda
//! e o arquivo reimporta com os mesmos segmentos.
use catcore::model::*;
use catcore::Core;
use std::io::Read;

fn corpus(n: &str) -> String {
    format!("{}/../corpus/out/{}", env!("CARGO_MANIFEST_DIR"), n)
}
fn saida(n: &str) -> String {
    let d = format!("{}/target/export-teste", env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(&d).unwrap();
    format!("{}/{}", d, n)
}
fn parte(path: &str, nome: &str) -> Vec<u8> {
    let mut z = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut v = vec![];
    z.by_name(nome).unwrap().read_to_end(&mut v).unwrap();
    v
}

#[test]
fn exportar_sem_edicao_preserva_partes() {
    let mut c = Core::abrir(":memory:").unwrap();
    let doc = c.importar(&corpus("pequeno.docx")).unwrap()["doc_id"].as_i64().unwrap();
    let out = saida("sem_edicao.docx");
    let r = c.exportar(doc, &out).unwrap();
    assert_eq!(r["blocos_alterados"], 0);
    assert_eq!(parte(&out, "word/document.xml"), parte(&corpus("pequeno.docx"), "word/document.xml"));
}

#[test]
fn exportar_com_edicao_reimporta_igual() {
    let mut c = Core::abrir(":memory:").unwrap();
    let doc = c.importar(&corpus("pequeno.docx")).unwrap()["doc_id"].as_i64().unwrap();
    let segs = c.segmentos(doc, 0, 1000).unwrap();
    // escolhe 3 segmentos com tags e 1 sem, edita prefixando "Zz" no primeiro texto
    let mut alvos: Vec<i64> = segs.iter().filter(|s| s.destino.iter().any(|n| matches!(n, Inline::Tag { .. }))).map(|s| s.id).step_by(7).take(3).collect();
    alvos.push(segs.iter().find(|s| !s.destino.iter().any(|n| matches!(n, Inline::Tag { .. }))).unwrap().id);
    for id in &alvos {
        let mut d = c.segmento(*id).unwrap().destino;
        if let Some(Inline::Texto { v, .. }) = d.iter_mut().find(|n| matches!(n, Inline::Texto { .. })) {
            *v = format!("Zz{}", v);
        }
        c.salvar_segmento(*id, d).unwrap();
    }
    let out = saida("com_edicao.docx");
    let r = c.exportar(doc, &out).unwrap();
    assert!(r["blocos_alterados"].as_i64().unwrap() >= 1);
    let esperado: Vec<_> = c.segmentos(doc, 0, 1000).unwrap().into_iter().map(|s| s.destino).collect();
    let mut c2 = Core::abrir(":memory:").unwrap();
    let doc2 = c2.importar(&out).unwrap()["doc_id"].as_i64().unwrap();
    let lido: Vec<_> = c2.segmentos(doc2, 0, 1000).unwrap().into_iter().map(|s| s.origem).collect();
    assert_eq!(lido.len(), esperado.len(), "contagem de segmentos");
    for (i, (a, b)) in lido.iter().zip(esperado.iter()).enumerate() {
        assert_eq!(a, b, "segmento {}", i);
    }
}

#[test]
fn exportar_pptx_com_edicao_reimporta_igual() {
    // PPTX sintético mínimo gerado por corpus/gen_pptx.py (ver README); pula se não existir
    let p = corpus("sintetico.pptx");
    if !std::path::Path::new(&p).exists() {
        eprintln!("sintetico.pptx ausente; pulando");
        return;
    }
    let mut c = Core::abrir(":memory:").unwrap();
    let doc = c.importar(&p).unwrap()["doc_id"].as_i64().unwrap();
    let segs = c.segmentos(doc, 0, 5000).unwrap();
    for id in segs.iter().step_by(11).take(5).map(|s| s.id) {
        let mut d = c.segmento(id).unwrap().destino;
        if let Some(Inline::Texto { v, .. }) = d.iter_mut().find(|n| matches!(n, Inline::Texto { .. })) {
            *v = format!("Zz{}", v);
        }
        c.salvar_segmento(id, d).unwrap();
    }
    let out = saida("com_edicao.pptx");
    assert!(c.exportar(doc, &out).unwrap()["blocos_alterados"].as_i64().unwrap() >= 1);
    let esperado: Vec<_> = c.segmentos(doc, 0, 5000).unwrap().into_iter().map(|s| s.destino).collect();
    let mut c2 = Core::abrir(":memory:").unwrap();
    let doc2 = c2.importar(&out).unwrap()["doc_id"].as_i64().unwrap();
    let lido: Vec<_> = c2.segmentos(doc2, 0, 5000).unwrap().into_iter().map(|s| s.origem).collect();
    assert_eq!(lido.len(), esperado.len());
    for (i, (a, b)) in lido.iter().zip(esperado.iter()).enumerate() {
        assert_eq!(a, b, "segmento {}", i);
    }
}
