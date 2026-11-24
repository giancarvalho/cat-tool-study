//! Leitor de TMX (DGT-TM). Os arquivos fornecidos estão em UTF-16 e trazem várias línguas por unidade;
//! extraímos só o par origem/destino pedido (ex.: EN-GB e PT-PT). Conteúdo de tags inline é descartado.
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;

fn decodificar(b: &[u8]) -> String {
    if b.len() >= 2 && b[0] == 0xff && b[1] == 0xfe {
        let u: Vec<u16> = b[2..].chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u)
    } else if b.len() >= 2 && b[0] == 0xfe && b[1] == 0xff {
        let u: Vec<u16> = b[2..].chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&u)
    } else {
        String::from_utf8_lossy(b).into_owned()
    }
}

pub fn parse_tmx(path: &std::path::Path, origem: &str, destino: &str) -> Result<Vec<(String, String)>> {
    let texto = decodificar(&std::fs::read(path)?);
    let mut rd = Reader::from_str(&texto);
    rd.config_mut().trim_text(false);
    let mut out = Vec::new();
    let (mut lang, mut in_seg, mut skip) = (String::new(), false, 0usize);
    let (mut o, mut d, mut cur) = (None::<String>, None::<String>, String::new());
    loop {
        match rd.read_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"tu" => {
                    o = None;
                    d = None;
                }
                b"tuv" => {
                    lang = e
                        .attributes()
                        .flatten()
                        .find(|a| a.key.local_name().as_ref() == b"lang")
                        .and_then(|a| a.unescape_value().ok().map(|c| c.to_uppercase()))
                        .unwrap_or_default();
                }
                b"seg" => {
                    in_seg = true;
                    cur.clear();
                }
                b"bpt" | b"ept" | b"ph" | b"it" | b"hi" if in_seg => skip += 1,
                _ => {}
            },
            Event::Text(t) if in_seg && skip == 0 => cur.push_str(&t.unescape()?),
            Event::End(e) => match e.local_name().as_ref() {
                b"bpt" | b"ept" | b"ph" | b"it" | b"hi" if in_seg => skip = skip.saturating_sub(1),
                b"seg" => {
                    in_seg = false;
                    let v = cur.trim().to_string();
                    if lang == origem.to_uppercase() {
                        o = Some(v);
                    } else if lang == destino.to_uppercase() {
                        d = Some(v);
                    }
                }
                b"tu" => {
                    if let (Some(a), Some(b)) = (o.take(), d.take()) {
                        if !a.is_empty() && !b.is_empty() {
                            out.push((a, b));
                        }
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(out)
}

/// Lê todos os `.tmx` de uma pasta (ordem alfabética, determinística).
pub fn parse_pasta(pasta: &str, origem: &str, destino: &str) -> Result<Vec<(String, String)>> {
    let mut arq: Vec<_> = std::fs::read_dir(pasta)?.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x.eq_ignore_ascii_case("tmx"))).collect();
    arq.sort();
    let mut out = Vec::new();
    for p in arq {
        out.extend(parse_tmx(&p, origem, destino)?);
    }
    Ok(out)
}
