use chrono::Utc;
use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct UsageLedger {
    conn: Connection,
}

impl UsageLedger {
    pub fn init(project_root: &Path) -> Result<Self> {
        let ind_dir = project_root.join(".ind");
        let _ = std::fs::create_dir_all(&ind_dir);
        let db_path = ind_dir.join("usage.db");
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS usage_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                prompt_tokens INTEGER NOT NULL,
                completion_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                cost_estimate REAL NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn record(
        &self,
        provider: &str,
        model: &str,
        prompt_tokens: usize,
        completion_tokens: usize,
    ) -> Result<()> {
        let timestamp = Utc::now().to_rfc3339();
        let total_tokens = prompt_tokens + completion_tokens;
        // Simple benchmark cost estimate ($0.002 per 1k tokens)
        let cost_estimate = (total_tokens as f64 / 1000.0) * 0.002;

        self.conn.execute(
            "INSERT INTO usage_records (timestamp, provider, model, prompt_tokens, completion_tokens, total_tokens, cost_estimate)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                timestamp,
                provider,
                model,
                prompt_tokens as i64,
                completion_tokens as i64,
                total_tokens as i64,
                cost_estimate,
            ],
        )?;

        Ok(())
    }

    pub fn summary(&self) -> Result<(usize, usize, f64)> {
        let mut stmt = self.conn.prepare(
            "SELECT COALESCE(SUM(prompt_tokens), 0), COALESCE(SUM(completion_tokens), 0), COALESCE(SUM(cost_estimate), 0.0)
             FROM usage_records",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let prompt: i64 = row.get(0)?;
            let completion: i64 = row.get(1)?;
            let cost: f64 = row.get(2)?;
            Ok((prompt as usize, completion as usize, cost))
        } else {
            Ok((0, 0, 0.0))
        }
    }
}
