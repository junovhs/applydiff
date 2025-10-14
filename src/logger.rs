use chrono::Utc;
use serde_json::json;

#[derive(Clone, Copy, Debug)]
pub struct Logger {
    rid: u64,
}

impl Logger {
    pub fn new(rid: u64) -> Self {
        Self { rid }
    }

    /// Structured JSONL info record (ts/level/rid/subsystem/action/msg).
    pub fn info(&self, subsystem: &str, action: &str, message: &str) {
        self.emit("info", subsystem, action, None, message);
    }

    fn emit(
        &self,
        level: &str,
        subsystem: &str,
        action: &str,
        code: Option<u32>,
        message: &str,
    ) {
        let rec = json!({
            "ts": Utc::now().to_rfc3339(),
            "level": level,
            "rid": self.rid,
            "subsystem": subsystem,
            "action": action,
            "code": code,
            "msg": message,
        });
        println!("{}", rec);
    }
}
