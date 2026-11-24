//! Casos T-R1, T-R5, T-R6, T-R7 (spec/SCRIPT.md §5). Exigem o corpus em corpus/out
//! (python corpus/gen_corpus.py corpus/out).
use catcore::model::*;
use catcore::Core;
use serde_json::Value;

fn corpus(n: &str) -> String {
    format!("{}/../corpus/out/{}", env!("CARGO_MANIFEST_DIR"), n)
}

fn esperado() -> Value {
    serde_json::from_str(&std::fs::read_to_string(corpus("esperado.json")).expect("rode gen_corpus.py")).unwrap()
}

fn balanceado(v: &[Inline]) -> bool {
    let mut abertos: Vec<u32> = vec![];
    for n in v {
        if let Inline::Tag { id, tipo, rotulo } = n {
            let abre = matches!(tipo, TagTipo::Abre) || (matches!(tipo, TagTipo::Link) && !rotulo.starts_with('/'));
            let fecha = matches!(tipo, TagTipo::Fecha) || (matches!(tipo, TagTipo::Link) && rotulo.starts_with('/'));
            if abre {
                abertos.push(*id);
            } else if fecha && abertos.pop() != Some(*id) {
                return false;
            }
        }
    }
    abertos.is_empty()
}

#[test]
fn t_r1_importacao_contagens() {
    let exp = esperado();
    for nome in ["pequeno.docx", "medio.docx", "longo.docx", "pior_caso.docx", "textos.xlsx"] {
        let mut c = Core::abrir(":memory:").unwrap();
        let r = c.importar(&corpus(nome)).unwrap();
        let n = r["n_segmentos"].as_i64().unwrap();
        assert_eq!(n, exp[nome]["segmentos"].as_i64().unwrap(), "{}", nome);
        let doc = r["doc_id"].as_i64().unwrap();
        for s in c.segmentos(doc, 0, 50).unwrap() {
            assert!(!s.pos.is_null() && s.pos["caminho"].is_string(), "posição de origem vazia em {}", nome);
            assert!(balanceado(&s.origem), "tags desbalanceadas em {}: {:?}", nome, s.origem);
        }
    }
}

#[test]
fn t_r1_pior_caso_tem_muitas_tags() {
    let mut c = Core::abrir(":memory:").unwrap();
    let r = c.importar(&corpus("pior_caso.docx")).unwrap();
    let doc = r["doc_id"].as_i64().unwrap();
    let segs = c.segmentos(doc, 0, 10000).unwrap();
    let com3 = segs.iter().filter(|s| s.origem.iter().filter(|n| matches!(n, Inline::Tag { .. })).count() >= 3).count();
    assert!(com3 * 2 > segs.len(), "maioria deveria ter 3+ tags: {}/{}", com3, segs.len());
}

#[test]
fn t_r5_confirmar_grava_versao_e_fila() {
    let mut c = Core::abrir(":memory:").unwrap();
    let doc = c.importar(&corpus("pequeno.docx")).unwrap()["doc_id"].as_i64().unwrap();
    let s = c.segmentos(doc, 3, 1).unwrap().remove(0);
    let mut d = s.destino.clone();
    d.push(Inline::texto(" revisado"));
    let v = c.salvar_segmento(s.id, d.clone()).unwrap()["versao"].as_i64().unwrap();
    assert_eq!(v, 1);
    let nv: i64 = c.db.query_row("SELECT COUNT(*) FROM segmento_versao WHERE segmento_id=?", [s.id], |r| r.get(0)).unwrap();
    assert_eq!(nv, 1);
    assert_eq!(c.fila("pendente").unwrap(), 1);
    assert_eq!(c.segmento(s.id).unwrap().destino, d);
}

#[test]
fn t_r6_mesclar_e_dividir() {
    let mut c = Core::abrir(":memory:").unwrap();
    let doc = c.importar(&corpus("pequeno.docx")).unwrap()["doc_id"].as_i64().unwrap();
    let n0 = c.contagem(doc).unwrap();
    // acha um segmento com vizinho no mesmo bloco
    let segs = c.segmentos(doc, 0, 200).unwrap();
    let i = (0..segs.len() - 1).find(|&i| segs[i].bloco_id == segs[i + 1].bloco_id).unwrap();
    let (a, b) = (segs[i].clone(), segs[i + 1].clone());
    let m = c.mesclar(a.id).unwrap();
    assert_eq!(c.contagem(doc).unwrap(), n0 - 1);
    assert_eq!(to_atoms(&m.destino).len(), to_atoms(&a.destino).len() + to_atoms(&b.destino).len() + 1);
    assert!(balanceado(&m.destino));
    // dividir de volta no ponto onde juntou
    let pos = to_atoms(&a.destino).len() + 1;
    let r = c.dividir(m.id, pos).unwrap();
    assert_eq!(c.contagem(doc).unwrap(), n0);
    assert_eq!(r.len(), 2);
    assert!(r[0].ordem < r[1].ordem);
    // a ordem do documento continua crescente
    let todos = c.segmentos(doc, 0, 1000).unwrap();
    assert!(todos.windows(2).all(|w| w[0].ordem < w[1].ordem));
    assert!(c.fila("pendente").unwrap() >= 2);
}

#[test]
fn t_r7_busca_na_memoria() {
    let mut c = Core::abrir(":memory:").unwrap();
    let n = c.tm_sintetico(100_000, 42).unwrap();
    assert_eq!(n, 100_000);
    c.tm_aquecer().unwrap();
    let alvo: String = c.db.query_row("SELECT origem FROM tm WHERE id=777", [], |r| r.get(0)).unwrap();
    let t = std::time::Instant::now();
    let r = c.buscar("memoria", &alvo).unwrap();
    let dt = t.elapsed();
    assert!(!r.is_empty() && r[0].pontuacao > 0.99, "casamento exato deve vir primeiro");
    assert!(r.len() <= 5);
    let t = std::time::Instant::now();
    let r2 = c.buscar("concordancia", "system record").unwrap();
    let dt2 = t.elapsed();
    assert!(!r2.is_empty());
    println!("memoria: {:?}, concordancia: {:?} (build de teste, nao e medicao)", dt, dt2);
}

// Arquivos reais em docs/ (fora do git): os testes são ignorados se não existirem.
fn docs(n: &str) -> Option<String> {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs").join(n);
    p.exists().then(|| p.to_string_lossy().into_owned())
}

#[test]
fn importa_pptx_real() {
    let Some(f) = std::fs::read_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs")).ok().and_then(|d| d.flatten().map(|e| e.path()).find(|p| p.extension().map_or(false, |x| x == "pptx"))) else { return };
    let mut c = Core::abrir(":memory:").unwrap();
    let r = c.importar(f.to_str().unwrap()).unwrap();
    assert!(r["n_segmentos"].as_i64().unwrap() > 100);
}

#[test]
fn carrega_tmx_real() {
    let Some(_) = docs("translation memory") else { return };
    let pares = catcore::tmx::parse_pasta(&docs("translation memory").unwrap(), "EN-GB", "PT-PT").unwrap();
    assert!(pares.len() > 50_000);
    assert!(pares.iter().all(|(a, b)| !a.is_empty() && !b.is_empty()));
}
