use serde::{Deserialize, Serialize};

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TagTipo {
    Abre,
    Fecha,
    Vazia,
    Link,
}

/// Nó inline: texto ou tag. Tags de abertura e fechamento pareiam por `id`.
/// Links usam tipo `link` e rótulo "link" (abre) ou "/link" (fecha).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "k")]
pub enum Inline {
    #[serde(rename = "t")]
    Texto {
        v: String,
        #[serde(default, skip_serializing_if = "is_false")]
        b: bool,
        #[serde(default, skip_serializing_if = "is_false")]
        i: bool,
    },
    #[serde(rename = "g")]
    Tag { id: u32, tipo: TagTipo, rotulo: String },
}

impl Inline {
    pub fn texto(s: &str) -> Inline {
        Inline::Texto { v: s.to_string(), b: false, i: false }
    }
}

/// Átomo: 1 caractere ou 1 tag. Posições de cursor e de divisão contam átomos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Atom {
    Ch(char),
    Tag(u32, TagTipo, String),
}

impl Atom {
    pub fn is_close(&self) -> bool {
        match self {
            Atom::Tag(_, TagTipo::Fecha, _) => true,
            Atom::Tag(_, TagTipo::Link, r) => r.starts_with('/'),
            _ => false,
        }
    }
    pub fn is_open(&self) -> bool {
        match self {
            Atom::Tag(_, TagTipo::Abre, _) => true,
            Atom::Tag(_, TagTipo::Link, r) => !r.starts_with('/'),
            _ => false,
        }
    }
}

pub fn to_atoms(v: &[Inline]) -> Vec<Atom> {
    let mut out = Vec::new();
    for n in v {
        match n {
            Inline::Texto { v, .. } => out.extend(v.chars().map(Atom::Ch)),
            Inline::Tag { id, tipo, rotulo } => {
                out.push(Atom::Tag(*id, tipo.clone(), rotulo.clone()))
            }
        }
    }
    out
}

pub fn from_atoms(a: &[Atom]) -> Vec<Inline> {
    let mut out: Vec<Inline> = Vec::new();
    let mut cur = String::new();
    for x in a {
        match x {
            Atom::Ch(c) => cur.push(*c),
            Atom::Tag(id, t, r) => {
                if !cur.is_empty() {
                    out.push(Inline::Texto { v: std::mem::take(&mut cur), b: false, i: false });
                }
                out.push(Inline::Tag { id: *id, tipo: t.clone(), rotulo: r.clone() });
            }
        }
    }
    if !cur.is_empty() {
        out.push(Inline::Texto { v: cur, b: false, i: false });
    }
    out
}

/// Limites de frase (spec §3 R1): terminador `.?!`, tags de fechamento opcionais,
/// 1+ espaços, tags de abertura opcionais, letra maiúscula.
/// Retorna pares (fim_do_segmento, inicio_do_proximo).
pub fn boundaries(atoms: &[Atom]) -> Vec<(usize, usize)> {
    let n = atoms.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if matches!(atoms[i], Atom::Ch('.') | Atom::Ch('?') | Atom::Ch('!')) {
            let mut j = i + 1;
            while j < n && atoms[j].is_close() {
                j += 1;
            }
            let mut k = j;
            while k < n && matches!(&atoms[k], Atom::Ch(c) if c.is_whitespace()) {
                k += 1;
            }
            if k > j {
                let mut m = k;
                while m < n && atoms[m].is_open() {
                    m += 1;
                }
                if m < n && matches!(&atoms[m], Atom::Ch(c) if c.is_uppercase()) {
                    out.push((j, k));
                    i = k;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Intervalos [início, fim) dos segmentos de uma sequência de átomos.
pub fn split_sentences(atoms: &[Atom]) -> Vec<(usize, usize)> {
    let mut res = Vec::new();
    let mut start = 0;
    for (j, k) in boundaries(atoms) {
        res.push((start, j));
        start = k;
    }
    res.push((start, atoms.len()));
    res.retain(|(a, b)| atoms[*a..*b].iter().any(|x| !matches!(x, Atom::Ch(c) if c.is_whitespace())));
    res
}
