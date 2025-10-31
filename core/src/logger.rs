use chrono::Utc;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct Logger {
    rid: u64,
}

impl Logger {
    /// Creates a new `Logger`.
    ///
    /// # Panics
    ///
    /// Panics if `rid` is zero.
    #[must_use]
    pub fn new(rid: u64) -> Self {
        assert!(rid > 0, "Logger rid must be non-zero");
        Self { rid }
    }

    pub fn info(&self, subsystem: &str, action: &str, message: &str) {
        self.emit("info", subsystem, action, message);
    }

    pub fn error(&self, subsystem: &str, action: &str, message: &str) {
        self.emit("error", subsystem, action, message);
    }

    fn emit(&self, level: &str, subsystem: &str, action: &str, message: &str) {
        let log_entry = json!({
            "ts": Utc::now().to_rfc3339(),
            "level": level,
            "rid": self.rid,
            "subsystem": subsystem,
            "action": action,
            "msg": message,
        });

        if level == "error" {
            eprintln!("{log_entry}");
        } else {
            println!("{log_entry}");
        }
    }
}