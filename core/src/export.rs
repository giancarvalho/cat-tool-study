//! Exportação (Emenda 5): reescreve o DOCX ou PPTX original trocando só os parágrafos cujo destino mudou.
//! O resto do arquivo é copiado byte a byte. Em um parágrafo alterado, ficam o `pPr` (e o `endParaRPr` no PPTX);
//! as execuções (`r`) são refeitas a partir dos átomos do destino, com a formatação-base (`rPr`) da primeira
//! execução original, mais negrito/itálico vindos das tags `b`/`i`.
//! Limitações declaradas: links (`hyperlink`) e campos (`fld`) dentro de um parágrafo alterado viram texto simples;
//! marcadores de revisão e bookmarks do parágrafo alterado são descartados; parágrafos aninhados (caixa de texto
//! dentro de parágrafo) não são tratados; o formato do bloco é reconstruído por junção dos segmentos com um espaço.
use crate::core::Core;
use crate::model::*;
use anyhow::{anyhow, bail, Result};
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{Read, Write};

type Ev = Event<'static>;

#[derive(Clone, Copy, PartialEq)]
enum D {
    Docx,
    Pptx,
}

enum Peca {
    Texto(String, bool, bool),
    Br,
    Tab,
}

fn pecas(atoms: &[Atom]) -> Vec<Peca> {
    fn flush(out: &mut Vec<Peca>, cur: &mut String, b: i32, i: i32) {
        if !cur.is_empty() {
            out.push(Peca::Texto(std::mem::take(cur), b > 0, i > 0));
        }
    }
    let (mut b, mut i) = (0i32, 0i32);
    let (mut out, mut cur) = (vec![], String::new());
    for a in atoms {
        match a {
            Atom::Ch('\t') => {
                flush(&mut out, &mut cur, b, i);
                out.push(Peca::Tab);
            }
            Atom::Ch(c) => cur.push(*c),
            Atom::Tag(_, tipo, r) => match (tipo, r.as_str()) {
                (TagTipo::Abre, "b") | (TagTipo::Fecha, "/b") | (TagTipo::Abre, "i") | (TagTipo::Fecha, "/i") => {
                    flush(&mut out, &mut cur, b, i);
                    let d = if matches!(tipo, TagTipo::Abre) { 1 } else { -1 };
                    if r.ends_with('b') {
                        b += d
                    } else {
                        i += d
                    }
                }
                (TagTipo::Vazia, "br") => {
                    flush(&mut out, &mut cur, b, i);
                    out.push(Peca::Br);
                }
                _ => {} // links e tags desconhecidas: o texto segue sem elas
            },
        }
    }
    flush(&mut out, &mut cur, b, i);
    out
}

fn local(e: &BytesStart) -> Vec<u8> {
    e.local_name().as_ref().to_vec()
}

/// Índice do evento que fecha o `Start` em `i`.
fn fim(evs: &[Ev], i: usize) -> usize {
    let mut d = 0;
    for (k, e) in evs.iter().enumerate().skip(i) {
        match e {
            Event::Start(_) => d += 1,
            Event::End(_) => {
                d -= 1;
                if d == 0 {
                    return k;
                }
            }
            _ => {}
        }
    }
    evs.len() - 1
}

#[derive(Default)]
struct Base {
    attrs: Vec<(String, String)>,
    filhos: Vec<(Vec<u8>, Vec<Ev>)>,
}

fn capturar_base(evs: &[Ev]) -> Base {
    let mut base = Base::default();
    // primeira execução `r`, depois seu primeiro `rPr`
    let Some(ri) = evs.iter().position(|e| matches!(e, Event::Start(s) if local(s) == b"r")) else { return base };
    let rf = fim(evs, ri);
    let mut k = ri + 1;
    while k < rf {
        let (nome, eh_start, e0) = match &evs[k] {
            Event::Start(s) => (local(s), true, s.clone()),
            Event::Empty(s) => (local(s), false, s.clone()),
            Event::End(_) => {
                k += 1;
                continue;
            }
            _ => {
                k += 1;
                continue;
            }
        };
        if nome == b"rPr" {
            for a in e0.attributes().flatten() {
                base.attrs.push((String::from_utf8_lossy(a.key.as_ref()).into_owned(), a.unescape_value().map(|c| c.into_owned()).unwrap_or_default()));
            }
            if eh_start {
                let ef = fim(evs, k);
                let mut j = k + 1;
                while j < ef {
                    match &evs[j] {
                        Event::Start(s) => {
                            let f = fim(evs, j);
                            base.filhos.push((local(s), evs[j..=f].to_vec()));
                            j = f + 1;
                        }
                        Event::Empty(s) => {
                            base.filhos.push((local(s), vec![evs[j].clone()]));
                            j += 1;
                        }
                        _ => j += 1,
                    }
                }
            }
            return base;
        }
        k = if eh_start { fim(evs, k) + 1 } else { k + 1 };
    }
    base
}

fn start(nome: &str) -> Ev {
    Event::Start(BytesStart::new(nome.to_string()))
}
fn end(nome: &str) -> Ev {
    Event::End(BytesEnd::new(nome.to_string()))
}
fn empty(nome: &str) -> Ev {
    Event::Empty(BytesStart::new(nome.to_string()))
}

fn rpr(d: D, base: &Base, b: bool, i: bool) -> Vec<Ev> {
    let p = if d == D::Docx { "w:" } else { "a:" };
    let mut out = vec![];
    match d {
        D::Docx => {
            let mut filhos: Vec<&(Vec<u8>, Vec<Ev>)> = base.filhos.iter().filter(|(n, _)| n != b"b" && n != b"i").collect();
            let mut extras: Vec<Vec<Ev>> = vec![];
            if b {
                extras.push(vec![empty("w:b")]);
            }
            if i {
                extras.push(vec![empty("w:i")]);
            }
            // b e i entram depois de rStyle/rFonts (ordem do esquema)
            let pos = filhos.iter().position(|(n, _)| n != b"rStyle" && n != b"rFonts").unwrap_or(filhos.len());
            let resto = filhos.split_off(pos);
            if filhos.is_empty() && extras.is_empty() && resto.is_empty() {
                return out;
            }
            out.push(start("w:rPr"));
            for f in filhos {
                out.extend(f.1.iter().cloned());
            }
            for x in extras {
                out.extend(x);
            }
            for f in resto {
                out.extend(f.1.iter().cloned());
            }
            out.push(end("w:rPr"));
        }
        D::Pptx => {
            let mut s = BytesStart::new(format!("{}rPr", p));
            for (k, v) in &base.attrs {
                if k != "b" && k != "i" {
                    s.push_attribute((k.as_str(), v.as_str()));
                }
            }
            if b {
                s.push_attribute(("b", "1"));
            }
            if i {
                s.push_attribute(("i", "1"));
            }
            if base.filhos.is_empty() {
                out.push(Event::Empty(s));
            } else {
                out.push(Event::Start(s));
                for f in &base.filhos {
                    out.extend(f.1.iter().cloned());
                }
                out.push(end("a:rPr"));
            }
        }
    }
    out
}

fn texto_ev(t: &str) -> Ev {
    Event::Text(BytesText::new(t).into_owned())
}

fn execucoes(d: D, base: &Base, pcs: &[Peca]) -> Vec<Ev> {
    let (r, t) = if d == D::Docx { ("w:r", "w:t") } else { ("a:r", "a:t") };
    let mut out = vec![];
    for p in pcs {
        match p {
            Peca::Texto(s, b, i) => {
                out.push(start(r));
                out.extend(rpr(d, base, *b, *i));
                let mut ts = BytesStart::new(t.to_string());
                if d == D::Docx {
                    ts.push_attribute(("xml:space", "preserve"));
                }
                out.push(Event::Start(ts));
                out.push(texto_ev(s));
                out.push(end(t));
                out.push(end(r));
            }
            Peca::Br => {
                if d == D::Docx {
                    out.push(start(r));
                    out.push(empty("w:br"));
                    out.push(end(r));
                } else {
                    out.push(empty("a:br"));
                }
            }
            Peca::Tab => {
                if d == D::Docx {
                    out.push(start(r));
                    out.push(empty("w:tab"));
                    out.push(end(r));
                } else {
                    out.push(start(r));
                    out.extend(rpr(d, base, false, false));
                    out.push(start(t));
                    out.push(texto_ev("\t"));
                    out.push(end(t));
                    out.push(end(r));
                }
            }
        }
    }
    out
}

/// Refaz um parágrafo: `evs` vai do `Start p` ao `End p`.
fn refazer(d: D, evs: &[Ev], atoms: &[Atom]) -> Vec<Ev> {
    let base = capturar_base(evs);
    let mut cabeca: Vec<Ev> = vec![];
    let mut cauda: Vec<Ev> = vec![];
    let mut k = 1;
    let ultimo = evs.len() - 1;
    while k < ultimo {
        match &evs[k] {
            Event::Start(s) => {
                let f = fim(evs, k);
                match local(s).as_slice() {
                    b"pPr" => cabeca.extend(evs[k..=f].iter().cloned()),
                    b"endParaRPr" => cauda.extend(evs[k..=f].iter().cloned()),
                    _ => {}
                }
                k = f + 1;
            }
            Event::Empty(s) => {
                match local(s).as_slice() {
                    b"pPr" => cabeca.push(evs[k].clone()),
                    b"endParaRPr" => cauda.push(evs[k].clone()),
                    _ => {}
                }
                k += 1;
            }
            _ => k += 1,
        }
    }
    let mut out = vec![evs[0].clone()];
    out.extend(cabeca);
    out.extend(execucoes(d, &base, &pecas(atoms)));
    out.extend(cauda);
    out.push(evs[ultimo].clone());
    out
}

fn reescrever(xml: &[u8], d: D, edits: &HashMap<String, Vec<Atom>>) -> Result<Vec<u8>> {
    let mut rd = Reader::from_reader(xml);
    rd.config_mut().trim_text(false);
    let mut w = Writer::new(Vec::new());
    let mut buf = Vec::new();
    let pptx_slide: i64 = -1; // o número do slide vem da chave (ver `chave`)
    let _ = pptx_slide;
    let (mut par_idx, mut idx_forma, mut idx_par) = (0usize, 0usize, 0usize);
    let chave = |d: D, par_idx: usize, idx_forma: usize, idx_par: usize| -> String {
        match d {
            D::Docx => format!("p[{}]", par_idx),
            D::Pptx => format!("forma[{}]/p[{}]", idx_forma, idx_par),
        }
    };
    loop {
        let ev = rd.read_event_into(&mut buf)?;
        match &ev {
            Event::Eof => break,
            Event::Start(e) => {
                let nome = local(e);
                if d == D::Pptx && matches!(nome.as_slice(), b"sp" | b"graphicFrame" | b"pic") {
                    idx_forma += 1;
                    idx_par = 0;
                }
                if nome == b"p" {
                    if let Some(atoms) = edits.get(&chave(d, par_idx, idx_forma, idx_par)) {
                        let mut evs: Vec<Ev> = vec![ev.clone().into_owned()];
                        let mut prof = 1;
                        let mut b2 = Vec::new();
                        while prof > 0 {
                            let e2 = rd.read_event_into(&mut b2)?;
                            match &e2 {
                                Event::Start(_) => prof += 1,
                                Event::End(_) => prof -= 1,
                                Event::Eof => bail!("XML truncado dentro de um parágrafo"),
                                _ => {}
                            }
                            evs.push(e2.into_owned());
                            b2.clear();
                        }
                        for x in refazer(d, &evs, atoms) {
                            w.write_event(x)?;
                        }
                        par_idx += 1;
                        idx_par += 1;
                        buf.clear();
                        continue;
                    }
                }
                w.write_event(ev.clone())?;
            }
            Event::End(e) => {
                if e.local_name().as_ref() == b"p" {
                    par_idx += 1;
                    idx_par += 1;
                }
                w.write_event(ev.clone())?;
            }
            _ => w.write_event(ev.clone())?,
        }
        buf.clear();
    }
    Ok(w.into_inner())
}

fn colapsar(a: &[Atom]) -> Vec<Atom> {
    let mut out: Vec<Atom> = vec![];
    for x in a {
        match x {
            Atom::Ch(c) if c.is_whitespace() => {
                if !matches!(out.last(), Some(Atom::Ch(' ')) | None) {
                    out.push(Atom::Ch(' '));
                }
            }
            o => out.push(o.clone()),
        }
    }
    while matches!(out.last(), Some(Atom::Ch(' '))) {
        out.pop();
    }
    out
}

fn juntar(segs: &[&Vec<Inline>]) -> Vec<Atom> {
    let mut a: Vec<Atom> = vec![];
    for s in segs {
        let t = to_atoms(s);
        if !a.is_empty() && !t.is_empty() {
            a.push(Atom::Ch(' '));
        }
        a.extend(t);
    }
    a
}

impl Core {
    /// Grava em `saida` uma cópia do documento importado com os destinos dos segmentos.
    pub fn exportar(&self, doc_id: i64, saida: &str) -> Result<Value> {
        let (caminho, tipo): (String, String) = self
            .db
            .query_row("SELECT caminho, tipo FROM documento WHERE id=?", [doc_id], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(|_| anyhow!("documento {} inexistente", doc_id))?;
        let d = match tipo.as_str() {
            "docx" => D::Docx,
            "pptx" => D::Pptx,
            t => bail!("exportação de {} não implementada", t),
        };
        let mut st = self.db.prepare("SELECT bloco_id, origem, destino, pos FROM segmento WHERE doc_id=? ORDER BY ordem")?;
        let linhas = st
            .query_map([doc_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        // agrupa segmentos consecutivos do mesmo bloco
        let mut edits: HashMap<String, HashMap<String, Vec<Atom>>> = HashMap::new();
        let mut i = 0;
        let mut blocos_alterados = 0;
        while i < linhas.len() {
            let mut j = i;
            while j < linhas.len() && linhas[j].0 == linhas[i].0 {
                j += 1;
            }
            let orig: Vec<Vec<Inline>> = linhas[i..j].iter().map(|l| serde_json::from_str(&l.1).unwrap_or_default()).collect();
            let dest: Vec<Vec<Inline>> = linhas[i..j].iter().map(|l| serde_json::from_str(&l.2).unwrap_or_default()).collect();
            let a_o = colapsar(&juntar(&orig.iter().collect::<Vec<_>>()));
            let a_d = colapsar(&juntar(&dest.iter().collect::<Vec<_>>()));
            if a_o != a_d {
                let pos: Value = serde_json::from_str(&linhas[i].3).unwrap_or(Value::Null);
                let parte = pos["parte"].as_str().ok_or_else(|| anyhow!("segmento sem posição de origem"))?.to_string();
                let cam = pos["caminho"].as_str().unwrap_or("").to_string();
                // PPTX: a chave do analisador é "slideN/forma[i]/p[j]"; a parte já identifica o slide
                let chave = if d == D::Pptx { cam.splitn(2, '/').nth(1).unwrap_or("").to_string() } else { cam };
                edits.entry(parte).or_default().insert(chave, a_d);
                blocos_alterados += 1;
            }
            i = j;
        }
        let f = std::fs::File::open(&caminho)?;
        let mut z = zip::ZipArchive::new(f)?;
        let saida_f = std::fs::File::create(saida)?;
        let mut zw = zip::ZipWriter::new(saida_f);
        for k in 0..z.len() {
            let nome = z.by_index(k)?.name().to_string();
            if let Some(e) = edits.get(&nome) {
                let mut xml = Vec::new();
                z.by_index(k)?.read_to_end(&mut xml)?;
                let novo = reescrever(&xml, d, e)?;
                zw.start_file(nome.as_str(), zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated))?;
                zw.write_all(&novo)?;
            } else {
                zw.raw_copy_file(z.by_index_raw(k)?)?;
            }
        }
        zw.finish()?;
        Ok(json!({"saida": saida, "blocos_alterados": blocos_alterados, "partes_alteradas": edits.len()}))
    }
}
