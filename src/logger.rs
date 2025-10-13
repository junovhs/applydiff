#[derive(Clone, Copy, Debug)]
pub struct Logger(pub u64);

impl Logger {
    pub fn new(rid: u64) -> Self { Self(rid) }
    #[allow(dead_code)]
    pub fn log(&self, msg: &str) {
        println!("[rid={}]: {}", self.0, msg);
    }
}
