//! Operações do núcleo: SQLite, importação, edição, fila de sync e busca (spec §3, §5).
use crate::import::{parse_docx, parse_pptx, parse_xlsx, ParsedBlock};
use crate::model::*;
use anyhow::{anyhow, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

const GAP: i64 = 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Segmento {
    pub id: i64,
    pub bloco_id: i64,
    pub ordem: i64,
    pub origem: Vec<Inline>,
    pub destino: Vec<Inline>,
    pub estado: String,
    pub versao: i64,
    pub pos: Value,
}

#[derive(Serialize, Clone, Debug)]
pub struct BlocoOut {
    pub id: i64,
    pub ordem: i64,
    pub tipo: String,
    pub nivel: i64,
    pub meta: Value,
    pub segmentos: Vec<Segmento>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ResultadoBusca {
    pub id: i64,
    pub origem: String,
    pub destino: String,
    pub pontuacao: f64,
}

pub struct Core {
    pub db: Connection,
    tm_cache: Option<Vec<(i64, Vec<u32>)>>,
}

fn now_us() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_micros() as i64).unwrap_or(0)
}

fn bigrams(s: &str) -> Vec<u32> {
    let cs: Vec<char> = s.to_lowercase().chars().collect();
    let mut v: Vec<u32> = cs.windows(2).map(|w| (w[0] as u32).wrapping_mul(0x9E37_79B1) ^ (w[1] as u32)).collect();
    v.sort_unstable();
    v.dedup();
    v
}

fn jaccard(a: &[u32], b: &[u32]) -> f64 {
    let (mut i, mut j, mut inter) = (0, 0, 0usize);
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            inter += 1;
            i += 1;
            j += 1;
        } else if a[i] < b[j] {
            i += 1
        } else {
            j += 1
        }
    }
    let uni = a.len() + b.len() - inter;
    if uni == 0 { 0.0 } else { inter as f64 / uni as f64 }
}

/// Texto simples de uma sequência inline (tags omitidas).
pub fn texto_simples(v: &[Inline]) -> String {
    v.iter()
        .filter_map(|n| if let Inline::Texto { v, .. } = n { Some(v.as_str()) } else { None })
        .collect()
}

fn merge_inlines(a: &[Inline], b: &[Inline]) -> Vec<Inline> {
    let mut at = to_atoms(a);
    if !at.is_empty() && !b.is_empty() {
        at.push(Atom::Ch(' '));
    }
    at.extend(to_atoms(b));
    from_atoms(&at)
}

impl Core {
    pub fn abrir(path: &str) -> Result<Core> {
        let db = Connection::open(path)?;
        if path != ":memory:" {
            db.pragma_update(None, "journal_mode", "WAL")?;
            db.pragma_update(None, "synchronous", "NORMAL")?;
        }
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS documento(id INTEGER PRIMARY KEY, caminho TEXT, tipo TEXT, criado_em INTEGER);
             CREATE TABLE IF NOT EXISTS bloco(id INTEGER PRIMARY KEY, doc_id INTEGER, ordem INTEGER, tipo TEXT, nivel INTEGER, meta TEXT);
             CREATE INDEX IF NOT EXISTS bloco_doc ON bloco(doc_id, ordem);
             CREATE TABLE IF NOT EXISTS segmento(id INTEGER PRIMARY KEY, doc_id INTEGER, bloco_id INTEGER, ordem INTEGER,
                 origem TEXT, destino TEXT, estado TEXT, versao INTEGER, pos TEXT);
             CREATE INDEX IF NOT EXISTS seg_doc ON segmento(doc_id, ordem);
             CREATE INDEX IF NOT EXISTS seg_bloco ON segmento(bloco_id, ordem);
             CREATE TABLE IF NOT EXISTS segmento_versao(id INTEGER PRIMARY KEY, segmento_id INTEGER, versao INTEGER, destino TEXT, criado_em INTEGER);
             CREATE TABLE IF NOT EXISTS sync_queue(id INTEGER PRIMARY KEY, op TEXT, segmento_id INTEGER, versao INTEGER, payload TEXT, criado_em INTEGER, estado TEXT);
             CREATE TABLE IF NOT EXISTS tm(id INTEGER PRIMARY KEY, origem TEXT, destino TEXT);
             CREATE VIRTUAL TABLE IF NOT EXISTS tm_fts USING fts5(origem, destino, content='tm', content_rowid='id', tokenize='trigram');",
        )?;
        Ok(Core { db, tm_cache: None })
    }

    // ---------- importação (R1) ----------
    pub fn importar(&mut self, caminho: &str) -> Result<Value> {
        let lower = caminho.to_lowercase();
        let (tipo, blocos) = if lower.ends_with(".docx") {
            ("docx", parse_docx(caminho)?)
        } else if lower.ends_with(".xlsx") {
            ("xlsx", parse_xlsx(caminho)?)
        } else if lower.ends_with(".pptx") {
            ("pptx", parse_pptx(caminho)?)
        } else {
            return Err(anyhow!("formato não suportado: {}", caminho));
        };
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO documento(caminho, tipo, criado_em) VALUES(?,?,?)", params![caminho, tipo, now_us()])?;
        let doc_id = tx.last_insert_rowid();
        let (mut nb, mut ns, mut ordem) = (0i64, 0i64, 0i64);
        {
            let mut ib = tx.prepare("INSERT INTO bloco(doc_id, ordem, tipo, nivel, meta) VALUES(?,?,?,?,?)")?;
            let mut is = tx.prepare(
                "INSERT INTO segmento(doc_id, bloco_id, ordem, origem, destino, estado, versao, pos) VALUES(?,?,?,?,?,'novo',0,?)",
            )?;
            for (bi, b) in blocos.iter().enumerate() {
                let ParsedBlock { tipo, nivel, meta, parte, caminho, atoms } = b;
                ib.execute(params![doc_id, (bi as i64 + 1) * GAP, tipo, nivel, meta.to_string()])?;
                let bloco_id = tx.last_insert_rowid();
                nb += 1;
                for (a, z) in split_sentences(atoms) {
                    ordem += GAP;
                    let ins = serde_json::to_string(&from_atoms(&atoms[a..z]))?;
                    let pos = json!({"parte": parte, "caminho": caminho, "intervalo": [a, z]}).to_string();
                    is.execute(params![doc_id, bloco_id, ordem, ins, ins, pos])?;
                    ns += 1;
                }
            }
        }
        tx.commit()?;
        Ok(json!({"doc_id": doc_id, "n_segmentos": ns, "n_blocos": nb}))
    }

    // ---------- leitura ----------
    fn row_seg(r: &rusqlite::Row) -> rusqlite::Result<Segmento> {
        let o: String = r.get(4)?;
        let d: String = r.get(5)?;
        let p: String = r.get(8)?;
        Ok(Segmento {
            id: r.get(0)?,
            bloco_id: r.get(2)?,
            ordem: r.get(3)?,
            origem: serde_json::from_str(&o).unwrap_or_default(),
            destino: serde_json::from_str(&d).unwrap_or_default(),
            estado: r.get(6)?,
            versao: r.get(7)?,
            pos: serde_json::from_str(&p).unwrap_or(Value::Null),
        })
    }
    const SEG_COLS: &'static str = "id, doc_id, bloco_id, ordem, origem, destino, estado, versao, pos";

    pub fn segmento(&self, id: i64) -> Result<Segmento> {
        let sql = format!("SELECT {} FROM segmento WHERE id=?", Self::SEG_COLS);
        self.db.query_row(&sql, [id], Self::row_seg).optional()?.ok_or_else(|| anyhow!("segmento {} inexistente", id))
    }

    /// Segmentos nas posições [de, de+n) da ordem do documento.
    pub fn segmentos(&self, doc_id: i64, de: i64, n: i64) -> Result<Vec<Segmento>> {
        let sql = format!("SELECT {} FROM segmento WHERE doc_id=? ORDER BY ordem LIMIT ? OFFSET ?", Self::SEG_COLS);
        let mut st = self.db.prepare_cached(&sql)?;
        let rows = st.query_map(params![doc_id, n, de], Self::row_seg)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn contagem(&self, doc_id: i64) -> Result<i64> {
        Ok(self.db.query_row("SELECT COUNT(*) FROM segmento WHERE doc_id=?", [doc_id], |r| r.get(0))?)
    }

    pub fn blocos(&self, ids: &[i64]) -> Result<Vec<BlocoOut>> {
        let mut out = vec![];
        for id in ids {
            let (ordem, tipo, nivel, meta): (i64, String, i64, String) = self.db.query_row(
                "SELECT ordem, tipo, nivel, meta FROM bloco WHERE id=?",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )?;
            let sql = format!("SELECT {} FROM segmento WHERE bloco_id=? ORDER BY ordem", Self::SEG_COLS);
            let mut st = self.db.prepare_cached(&sql)?;
            let segs = st.query_map([id], Self::row_seg)?.collect::<rusqlite::Result<Vec<_>>>()?;
            out.push(BlocoOut { id: *id, ordem, tipo, nivel, meta: serde_json::from_str(&meta).unwrap_or(Value::Null), segmentos: segs });
        }
        Ok(out)
    }

    /// Lista de blocos do documento, em ordem (sem segmentos) para montar a prévia.
    pub fn blocos_do_doc(&self, doc_id: i64) -> Result<Vec<i64>> {
        let mut st = self.db.prepare_cached("SELECT id FROM bloco WHERE doc_id=? ORDER BY ordem")?;
        let r = st.query_map([doc_id], |r| r.get(0))?.collect::<rusqlite::Result<Vec<i64>>>()?;
        Ok(r)
    }

    /// Metadados dos blocos do documento, em ordem (para montar a prévia).
    pub fn blocos_meta(&self, doc_id: i64) -> Result<Vec<Value>> {
        let mut st = self.db.prepare_cached("SELECT id, tipo, nivel, meta FROM bloco WHERE doc_id=? ORDER BY ordem")?;
        let r = st
            .query_map([doc_id], |r| {
                let m: String = r.get(3)?;
                Ok(json!({"id": r.get::<_, i64>(0)?, "tipo": r.get::<_, String>(1)?, "nivel": r.get::<_, i64>(2)?, "meta": serde_json::from_str::<Value>(&m).unwrap_or(Value::Null)}))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(r)
    }

    // ---------- edição (R5, R6) ----------
    fn enfileirar(tx: &rusqlite::Transaction, op: &str, seg: i64, versao: i64, payload: Value) -> Result<()> {
        tx.execute(
            "INSERT INTO sync_queue(op, segmento_id, versao, payload, criado_em, estado) VALUES(?,?,?,?,?,'pendente')",
            params![op, seg, versao, payload.to_string(), now_us()],
        )?;
        Ok(())
    }

    pub fn salvar_segmento(&mut self, id: i64, destino: Vec<Inline>) -> Result<Value> {
        let tx = self.db.transaction()?;
        let s = serde_json::to_string(&destino)?;
        let n = tx.execute("UPDATE segmento SET destino=?, estado='confirmado', versao=versao+1 WHERE id=?", params![s, id])?;
        if n == 0 {
            return Err(anyhow!("segmento {} inexistente", id));
        }
        let versao: i64 = tx.query_row("SELECT versao FROM segmento WHERE id=?", [id], |r| r.get(0))?;
        tx.execute(
            "INSERT INTO segmento_versao(segmento_id, versao, destino, criado_em) VALUES(?,?,?,?)",
            params![id, versao, s, now_us()],
        )?;
        Self::enfileirar(&tx, "salvar", id, versao, json!({"segmento_id": id, "versao": versao}))?;
        tx.commit()?;
        Ok(json!({"versao": versao}))
    }

    /// Une o segmento ao seguinte do mesmo bloco. Retorna o segmento resultante.
    pub fn mesclar(&mut self, id: i64) -> Result<Segmento> {
        let a = self.segmento(id)?;
        let next_id: Option<i64> = self
            .db
            .query_row(
                "SELECT id FROM segmento WHERE bloco_id=? AND ordem>? ORDER BY ordem LIMIT 1",
                params![a.bloco_id, a.ordem],
                |r| r.get(0),
            )
            .optional()?;
        let b = self.segmento(next_id.ok_or_else(|| anyhow!("sem segmento seguinte no bloco"))?)?;
        let origem = merge_inlines(&a.origem, &b.origem);
        let destino = merge_inlines(&a.destino, &b.destino);
        let versao = a.versao + 1;
        let (so, sd) = (serde_json::to_string(&origem)?, serde_json::to_string(&destino)?);
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM segmento WHERE id=?", [b.id])?;
        tx.execute("UPDATE segmento SET origem=?, destino=?, versao=? WHERE id=?", params![so, sd, versao, a.id])?;
        tx.execute(
            "INSERT INTO segmento_versao(segmento_id, versao, destino, criado_em) VALUES(?,?,?,?)",
            params![a.id, versao, sd, now_us()],
        )?;
        Self::enfileirar(&tx, "mesclar", a.id, versao, json!({"segmento_id": a.id, "removido": b.id}))?;
        tx.commit()?;
        self.segmento(a.id)
    }

    /// Divide o destino na posição `pos` (índice de átomo: caractere ou tag).
    /// A origem acompanha o limite de frase mais próximo da proporção `pos/len`;
    /// sem limite, a origem fica inteira no primeiro e o segundo recebe origem vazia.
    pub fn dividir(&mut self, id: i64, pos: usize) -> Result<Vec<Segmento>> {
        let s = self.segmento(id)?;
        let dest = to_atoms(&s.destino);
        if pos == 0 || pos >= dest.len() {
            return Err(anyhow!("posição inválida"));
        }
        let orig = to_atoms(&s.origem);
        let alvo = orig.len() as f64 * pos as f64 / dest.len() as f64;
        let cut = boundaries(&orig)
            .into_iter()
            .min_by(|x, y| ((x.0 as f64) - alvo).abs().partial_cmp(&((y.0 as f64) - alvo).abs()).unwrap());
        let (o1, o2) = match cut {
            Some((j, k)) => (from_atoms(&orig[..j]), from_atoms(&orig[k..])),
            None => (s.origem.clone(), vec![]),
        };
        let (d1, d2) = (from_atoms(&dest[..pos]), from_atoms(&dest[pos..]));
        // nova ordem: ponto médio até o próximo segmento do documento
        let prox: Option<i64> = self
            .db
            .query_row("SELECT ordem FROM segmento WHERE doc_id=(SELECT doc_id FROM segmento WHERE id=?1) AND ordem>?2 ORDER BY ordem LIMIT 1", params![id, s.ordem], |r| r.get(0))
            .optional()?;
        let doc_id: i64 = self.db.query_row("SELECT doc_id FROM segmento WHERE id=?", [id], |r| r.get(0))?;
        let mut nova = match prox {
            Some(p) => (s.ordem + p) / 2,
            None => s.ordem + GAP,
        };
        let renumerar = nova <= s.ordem || prox.map_or(false, |p| nova >= p);
        let versao = s.versao + 1;
        let tx = self.db.transaction()?;
        if renumerar {
            // sem espaço: reabre o espaço deslocando os seguintes
            tx.execute("UPDATE segmento SET ordem=ordem+? WHERE doc_id=? AND ordem>?", params![GAP * 2, doc_id, s.ordem])?;
            nova = s.ordem + GAP;
        }
        let (so1, sd1, so2, sd2) = (
            serde_json::to_string(&o1)?,
            serde_json::to_string(&d1)?,
            serde_json::to_string(&o2)?,
            serde_json::to_string(&d2)?,
        );
        tx.execute("UPDATE segmento SET origem=?, destino=?, versao=? WHERE id=?", params![so1, sd1, versao, id])?;
        tx.execute(
            "INSERT INTO segmento(doc_id, bloco_id, ordem, origem, destino, estado, versao, pos) VALUES(?,?,?,?,?,'novo',0,?)",
            params![doc_id, s.bloco_id, nova, so2, sd2, s.pos.to_string()],
        )?;
        let novo_id = tx.last_insert_rowid();
        tx.execute(
            "INSERT INTO segmento_versao(segmento_id, versao, destino, criado_em) VALUES(?,?,?,?)",
            params![id, versao, sd1, now_us()],
        )?;
        Self::enfileirar(&tx, "dividir", id, versao, json!({"segmento_id": id, "novo": novo_id, "pos": pos}))?;
        tx.commit()?;
        Ok(vec![self.segmento(id)?, self.segmento(novo_id)?])
    }

    pub fn fila(&self, estado: &str) -> Result<i64> {
        Ok(self.db.query_row("SELECT COUNT(*) FROM sync_queue WHERE estado=?", [estado], |r| r.get(0))?)
    }

    // ---------- memória de tradução (R7) ----------
    pub fn tm_inserir(&mut self, pares: &[(String, String)]) -> Result<i64> {
        let tx = self.db.transaction()?;
        {
            let mut st = tx.prepare("INSERT INTO tm(origem, destino) VALUES(?,?)")?;
            for (o, d) in pares {
                st.execute(params![o, d])?;
            }
        }
        tx.execute("INSERT INTO tm_fts(tm_fts) VALUES('rebuild')", [])?;
        tx.commit()?;
        self.tm_cache = None;
        Ok(self.db.query_row("SELECT COUNT(*) FROM tm", [], |r| r.get(0))?)
    }

    /// TM sintética determinística (para testes e para o roteiro quando o DGT-TM não está disponível).
    pub fn tm_sintetico(&mut self, n: usize, semente: u64) -> Result<i64> {
        const EN: [&str; 24] = ["system","record","segment","document","review","policy","section","value","process","report","field","table","update","source","target","model","number","version","output","window","editor","memory","result","change"];
        const PT: [&str; 24] = ["sistema","registro","segmento","documento","revisao","politica","secao","valor","processo","relatorio","campo","tabela","atualizar","fonte","alvo","modelo","numero","versao","saida","janela","editor","memoria","resultado","mudanca"];
        let mut x = semente | 1;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut pares = Vec::with_capacity(n);
        for _ in 0..n {
            let len = 6 + (next() % 8) as usize;
            let (mut o, mut d) = (vec![], vec![]);
            for _ in 0..len {
                let k = (next() % 24) as usize;
                o.push(EN[k]);
                d.push(PT[k]);
            }
            pares.push((o.join(" "), d.join(" ")));
        }
        self.tm_inserir(&pares)
    }

    fn tm_carregar_cache(&mut self) -> Result<()> {
        if self.tm_cache.is_some() {
            return Ok(());
        }
        let mut st = self.db.prepare("SELECT id, origem FROM tm")?;
        let v = st.query_map([], |r| Ok((r.get::<_, i64>(0)?, bigrams(&r.get::<_, String>(1)?))))?.collect::<rusqlite::Result<Vec<_>>>()?;
        drop(st);
        self.tm_cache = Some(v);
        Ok(())
    }

    /// Pré-carrega o índice de bigramas (chamar na abertura do documento).
    pub fn tm_aquecer(&mut self) -> Result<usize> {
        self.tm_carregar_cache()?;
        Ok(self.tm_cache.as_ref().map_or(0, |c| c.len()))
    }

    pub fn buscar(&mut self, modo: &str, texto: &str) -> Result<Vec<ResultadoBusca>> {
        match modo {
            "concordancia" => {
                if texto.chars().count() < 3 {
                    return Ok(vec![]);
                }
                let q = format!("\"{}\"", texto.replace('"', "\"\""));
                let mut st = self.db.prepare_cached(
                    "SELECT tm.id, tm.origem, tm.destino FROM tm_fts JOIN tm ON tm.id = tm_fts.rowid WHERE tm_fts MATCH ? LIMIT 20",
                )?;
                let r = st
                    .query_map([q], |r| Ok(ResultadoBusca { id: r.get(0)?, origem: r.get(1)?, destino: r.get(2)?, pontuacao: 1.0 }))?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                Ok(r)
            }
            "memoria" => {
                self.tm_carregar_cache()?;
                let q = bigrams(texto);
                let mut top: Vec<(f64, i64)> = self
                    .tm_cache
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|(id, b)| (jaccard(&q, b), *id))
                    .filter(|(s, _)| *s >= 0.5)
                    .collect();
                top.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.cmp(&b.1)));
                top.truncate(5);
                let mut out = vec![];
                for (s, id) in top {
                    let (o, d): (String, String) = self.db.query_row("SELECT origem, destino FROM tm WHERE id=?", [id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                    out.push(ResultadoBusca { id, origem: o, destino: d, pontuacao: s });
                }
                Ok(out)
            }
            _ => Err(anyhow!("modo de busca desconhecido: {}", modo)),
        }
    }
}
