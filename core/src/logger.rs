use chrono::Utc;
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Logger {
    rid: u64,
    // Optional buffer for capturing logs during tests
    output: Option<Rc<RefCell<String>>>,
}

impl Logger {
    /// Creates a new logger that prints to standard output.
    pub fn new(rid: u64) -> Self {
        Self { rid, output: None }
    }

    /// Creates a new logger for testing that captures output to a string buffer.
    pub fn new_for_test(rid: u64, buffer: Option<Rc<RefCell<String>>>) -> Self {
        Self { rid, output: buffer }
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
        
        // If an output buffer is configured (for testing), write to it.
        // Otherwise, print to standard output for live runs.
        if let Some(output_rc) = &self.output {
            let mut writer = output_rc.borrow_mut();
            writer.push_str(&rec.to_string());
            writer.push('\n');
        } else {
            println!("{}", rec);
        }
    }
}