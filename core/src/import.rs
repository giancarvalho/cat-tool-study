//! Importadores DOCX e XLSX. Cada parágrafo (ou célula) vira um bloco; formatação
//! inline (negrito/itálico), links e quebras viram tags atômicas (decisão: o importador
//! não usa `b`/`i` do nó de texto, ver spec §1).
use crate::model::*;
use anyhow::{anyhow, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use serde_json::{json, Value};
use std::io::Read;

pub struct ParsedBlock {
    pub tipo: String,
    pub nivel: i64,
    pub meta: Value,
    pub parte: String,
    pub caminho: String,
    pub atoms: Vec<Atom>,
}

fn attr(e: &BytesStart, name: &[u8]) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.local_name().as_ref() == name {
            return a.unescape_value().ok().map(|c| c.into_owned());
        }
    }
    None
}

fn read_zip_entry(path: &str, name: &str) -> Result<Option<Vec<u8>>> {
    let f = std::fs::File::open(path)?;
    let mut z = zip::ZipArchive::new(f)?;
    let r = match z.by_name(name) {
        Ok(mut f) => {
            let mut v = Vec::new();
            f.read_to_end(&mut v)?;
            Some(v)
        }
        Err(_) => None,
    };
    Ok(r)
}

struct Par {
    atoms: Vec<Atom>,
    style: String,
    num: bool,
    stack: Vec<char>, // formatação aberta: 'b' / 'i'
    next_id: u32,
    links: Vec<u32>,
}

impl Par {
    fn new() -> Par {
        Par { atoms: vec![], style: String::new(), num: false, stack: vec![], next_id: 1, links: vec![] }
    }
    fn id(&mut self) -> u32 {
        let i = self.next_id;
        self.next_id += 1;
        i
    }
    fn open(&mut self, c: char) {
        let id = self.id();
        self.atoms.push(Atom::Tag(id, TagTipo::Abre, c.to_string()));
        self.stack.push(c);
    }
    fn close_top(&mut self, ids: &mut Vec<u32>) {
        let c = self.stack.pop().unwrap();
        let id = ids.pop().unwrap();
        self.atoms.push(Atom::Tag(id, TagTipo::Fecha, format!("/{}", c)));
    }
    fn transition(&mut self, want: (bool, bool), ids: &mut Vec<u32>) {
        let wanted = |c: char| if c == 'b' { want.0 } else { want.1 };
        // fecha a partir do primeiro item indesejado
        let first_bad = self.stack.iter().position(|c| !wanted(*c));
        let mut reopen = vec![];
        if let Some(idx) = first_bad {
            while self.stack.len() > idx {
                let c = *self.stack.last().unwrap();
                self.close_top(ids);
                if wanted(c) {
                    reopen.push(c);
                }
            }
        }
        reopen.reverse();
        for c in reopen {
            self.open(c);
            ids.push(self.next_id - 1);
        }
        for c in ['b', 'i'] {
            if wanted(c) && !self.stack.contains(&c) {
                self.open(c);
                ids.push(self.next_id - 1);
            }
        }
    }
    fn text(&mut self, s: &str, want: (bool, bool), ids: &mut Vec<u32>) {
        self.transition(want, ids);
        self.atoms.extend(s.chars().map(Atom::Ch));
    }
    fn has_text(&self) -> bool {
        self.atoms.iter().any(|a| matches!(a, Atom::Ch(c) if !c.is_whitespace()))
    }
}

pub fn parse_docx(path: &str) -> Result<Vec<ParsedBlock>> {
    let xml = read_zip_entry(path, "word/document.xml")?.ok_or_else(|| anyhow!("sem word/document.xml"))?;
    let mut rd = Reader::from_reader(xml.as_slice());
    rd.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = Vec::new();
    let mut par: Option<Par> = None;
    let mut ids: Vec<u32> = vec![]; // ids das tags de formatação abertas (paralelo a par.stack)
    let (mut in_ppr, mut in_rpr, mut in_t) = (false, false, false);
    let mut run_fmt = (false, false);
    let (mut tbl, mut row, mut col) = (0i32, -1i64, -1i64);
    let mut par_idx = 0usize;
    loop {
        let ev = rd.read_event_into(&mut buf)?;
        match &ev {
            Event::Start(e) | Event::Empty(e) => {
                let is_start = matches!(ev, Event::Start(_));
                let name = e.local_name();
                match name.as_ref() {
                    b"p" => {
                        par = Some(Par::new());
                        ids.clear();
                        if !is_start {
                            par = None;
                        }
                    }
                    b"pPr" => in_ppr = is_start,
                    b"pStyle" if in_ppr => {
                        if let (Some(p), Some(v)) = (par.as_mut(), attr(e, b"val")) {
                            p.style = v;
                        }
                    }
                    b"numPr" if in_ppr => {
                        if let Some(p) = par.as_mut() {
                            p.num = true;
                        }
                    }
                    b"r" if is_start => run_fmt = (false, false),
                    b"rPr" => in_rpr = is_start,
                    b"b" | b"i" if in_rpr => {
                        let off = matches!(attr(e, b"val").as_deref(), Some("0") | Some("false"));
                        if !off {
                            if name.as_ref() == b"b" {
                                run_fmt.0 = true
                            } else {
                                run_fmt.1 = true
                            }
                        }
                    }
                    b"t" if is_start => in_t = true,
                    b"br" => {
                        if let Some(p) = par.as_mut() {
                            let id = p.id();
                            p.atoms.push(Atom::Tag(id, TagTipo::Vazia, "br".into()));
                        }
                    }
                    b"tab" => {
                        if let Some(p) = par.as_mut() {
                            p.atoms.push(Atom::Ch('\t'));
                        }
                    }
                    b"hyperlink" if is_start => {
                        if let Some(p) = par.as_mut() {
                            let mut fids = std::mem::take(&mut ids);
                            p.transition((false, false), &mut fids);
                            ids = fids;
                            let id = p.id();
                            p.links.push(id);
                            p.atoms.push(Atom::Tag(id, TagTipo::Link, "link".into()));
                        }
                    }
                    b"tbl" if is_start => tbl += 1,
                    b"tr" if is_start => {
                        row += 1;
                        col = -1;
                    }
                    b"tc" if is_start => col += 1,
                    _ => {}
                }
            }
            Event::Text(t) if in_t => {
                if let Some(p) = par.as_mut() {
                    let s = t.unescape()?.into_owned();
                    p.text(&s, run_fmt, &mut ids);
                }
            }
            Event::End(e) => {
                let name = e.local_name();
                match name.as_ref() {
                    b"t" => in_t = false,
                    b"rPr" => in_rpr = false,
                    b"pPr" => in_ppr = false,
                    b"hyperlink" => {
                        if let Some(p) = par.as_mut() {
                            let mut fids = std::mem::take(&mut ids);
                            p.transition((false, false), &mut fids);
                            ids = fids;
                            if let Some(id) = p.links.pop() {
                                p.atoms.push(Atom::Tag(id, TagTipo::Link, "/link".into()));
                            }
                        }
                    }
                    b"tbl" => {
                        tbl -= 1;
                        if tbl == 0 {
                            row = -1;
                            col = -1;
                        }
                    }
                    b"p" => {
                        if let Some(mut p) = par.take() {
                            let mut fids = std::mem::take(&mut ids);
                            p.transition((false, false), &mut fids);
                            if p.has_text() {
                                let (tipo, nivel, meta) = if tbl > 0 {
                                    ("tabela", 0, json!({"linha": row, "coluna": col}))
                                } else if p.style.starts_with("Heading") || p.style == "Title" {
                                    let n = p.style.chars().rev().find(|c| c.is_ascii_digit()).and_then(|c| c.to_digit(10)).unwrap_or(1);
                                    ("titulo", n as i64, json!({}))
                                } else if p.style.starts_with("List") || p.num {
                                    ("item_lista", 0, json!({}))
                                } else {
                                    ("paragrafo", 0, json!({}))
                                };
                                out.push(ParsedBlock {
                                    tipo: tipo.into(),
                                    nivel,
                                    meta,
                                    parte: "word/document.xml".into(),
                                    caminho: format!("p[{}]", par_idx),
                                    atoms: p.atoms,
                                });
                            }
                            par_idx += 1;
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

/// PPTX (Emenda 4): cada parágrafo `a:p` de cada forma/célula de `ppt/slides/slideN.xml` vira um bloco;
/// negrito e itálico viram tags como no DOCX. Slides em ordem numérica do nome do arquivo (premissa do
/// protótipo, como no XLSX). Notas do orador, layouts e mestres ficam fora do protótipo.
pub fn parse_pptx(path: &str) -> Result<Vec<ParsedBlock>> {
    let f = std::fs::File::open(path)?;
    let mut z = zip::ZipArchive::new(f)?;
    let mut slides: Vec<(u32, String)> = z
        .file_names()
        .filter_map(|n| {
            let r = n.strip_prefix("ppt/slides/slide")?.strip_suffix(".xml")?;
            Some((r.parse::<u32>().ok()?, n.to_string()))
        })
        .collect();
    slides.sort();
    let mut out = Vec::new();
    for (num, part) in slides {
        let mut xml = Vec::new();
        z.by_name(&part)?.read_to_end(&mut xml)?;
        let mut rd = Reader::from_reader(xml.as_slice());
        rd.config_mut().trim_text(false);
        let mut buf = Vec::new();
        let mut par: Option<Par> = None;
        let mut ids: Vec<u32> = vec![];
        let (mut in_t, mut run_fmt) = (false, (false, false));
        let (mut forma_id, mut forma_nome, mut titulo) = (String::new(), String::new(), false);
        let (mut idx_forma, mut idx_par) = (0usize, 0usize);
        loop {
            let ev = rd.read_event_into(&mut buf)?;
            match &ev {
                Event::Start(e) | Event::Empty(e) => {
                    let is_start = matches!(ev, Event::Start(_));
                    match e.local_name().as_ref() {
                        b"sp" | b"graphicFrame" | b"pic" if is_start => {
                            idx_forma += 1;
                            idx_par = 0;
                            titulo = false;
                        }
                        b"cNvPr" => {
                            forma_id = attr(e, b"id").unwrap_or_default();
                            forma_nome = attr(e, b"name").unwrap_or_default();
                        }
                        b"ph" => titulo = matches!(attr(e, b"type").as_deref(), Some("title") | Some("ctrTitle")),
                        b"p" if is_start => {
                            par = Some(Par::new());
                            ids.clear();
                        }
                        b"r" if is_start => run_fmt = (false, false),
                        b"rPr" => {
                            let on = |n: &[u8]| matches!(attr(e, n).as_deref(), Some("1") | Some("true"));
                            run_fmt = (on(b"b"), on(b"i"));
                        }
                        b"t" if is_start => in_t = true,
                        b"br" => {
                            if let Some(p) = par.as_mut() {
                                let id = p.id();
                                p.atoms.push(Atom::Tag(id, TagTipo::Vazia, "br".into()));
                            }
                        }
                        _ => {}
                    }
                }
                Event::Text(t) if in_t => {
                    if let Some(p) = par.as_mut() {
                        let s = t.unescape()?.into_owned();
                        p.text(&s, run_fmt, &mut ids);
                    }
                }
                Event::End(e) => match e.local_name().as_ref() {
                    b"t" => in_t = false,
                    b"p" => {
                        if let Some(mut p) = par.take() {
                            let mut fids = std::mem::take(&mut ids);
                            p.transition((false, false), &mut fids);
                            if p.has_text() {
                                out.push(ParsedBlock {
                                    tipo: if titulo { "titulo" } else { "forma" }.into(),
                                    nivel: 0,
                                    meta: json!({"slide": num, "forma_id": forma_id, "forma": forma_nome}),
                                    parte: part.clone(),
                                    caminho: format!("slide{}/forma[{}]/p[{}]", num, idx_forma, idx_par),
                                    atoms: p.atoms,
                                });
                            }
                            idx_par += 1;
                        }
                    }
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
    }
    Ok(out)
}

fn parse_shared_strings(path: &str) -> Result<Vec<String>> {
    let Some(xml) = read_zip_entry(path, "xl/sharedStrings.xml")? else { return Ok(vec![]) };
    let mut rd = Reader::from_reader(xml.as_slice());
    rd.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut out = vec![];
    let (mut cur, mut in_t, mut in_ph) = (String::new(), false, false);
    loop {
        match rd.read_event_into(&mut buf)? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"si" => cur.clear(),
                b"rPh" => in_ph = true,
                b"t" => in_t = true,
                _ => {}
            },
            Event::Text(t) if in_t && !in_ph => cur.push_str(&t.unescape()?),
            Event::End(e) => match e.local_name().as_ref() {
                b"si" => out.push(std::mem::take(&mut cur)),
                b"rPh" => in_ph = false,
                b"t" => in_t = false,
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

fn sheet_names(path: &str) -> Result<Vec<String>> {
    let Some(xml) = read_zip_entry(path, "xl/workbook.xml")? else { return Ok(vec![]) };
    let mut rd = Reader::from_reader(xml.as_slice());
    let mut buf = Vec::new();
    let mut out = vec![];
    loop {
        match rd.read_event_into(&mut buf)? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"sheet" => {
                out.push(attr(&e, b"name").unwrap_or_default());
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(out)
}

/// Premissa do protótipo: sheetN.xml corresponde à N-ésima planilha de workbook.xml.
pub fn parse_xlsx(path: &str) -> Result<Vec<ParsedBlock>> {
    let shared = parse_shared_strings(path)?;
    let names = sheet_names(path)?;
    let mut out = vec![];
    for n in 1.. {
        let part = format!("xl/worksheets/sheet{}.xml", n);
        let Some(xml) = read_zip_entry(path, &part)? else { break };
        let nome = names.get(n - 1).cloned().unwrap_or_else(|| format!("Sheet{}", n));
        let mut rd = Reader::from_reader(xml.as_slice());
        rd.config_mut().trim_text(false);
        let mut buf = Vec::new();
        let (mut cref, mut ctype) = (String::new(), String::new());
        let (mut in_v, mut in_t, mut in_is) = (false, false, false);
        let mut val = String::new();
        loop {
            match rd.read_event_into(&mut buf)? {
                Event::Start(e) => match e.local_name().as_ref() {
                    b"c" => {
                        cref = attr(&e, b"r").unwrap_or_default();
                        ctype = attr(&e, b"t").unwrap_or_default();
                        val.clear();
                    }
                    b"v" => in_v = true,
                    b"is" => in_is = true,
                    b"t" if in_is => in_t = true,
                    _ => {}
                },
                Event::Text(t) if in_v || in_t => val.push_str(&t.unescape()?),
                Event::End(e) => match e.local_name().as_ref() {
                    b"v" => in_v = false,
                    b"t" => in_t = false,
                    b"is" => in_is = false,
                    b"c" => {
                        let text = match ctype.as_str() {
                            "s" => val.trim().parse::<usize>().ok().and_then(|i| shared.get(i).cloned()),
                            "inlineStr" => Some(val.clone()),
                            _ => None,
                        };
                        if let Some(t) = text {
                            if !t.trim().is_empty() {
                                out.push(ParsedBlock {
                                    tipo: "celula".into(),
                                    nivel: 0,
                                    meta: json!({"planilha": nome, "celula": cref}),
                                    parte: part.clone(),
                                    caminho: format!("{}!{}", nome, cref),
                                    atoms: t.chars().map(Atom::Ch).collect(),
                                });
                            }
                        }
                    }
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
    }
    Ok(out)
}
