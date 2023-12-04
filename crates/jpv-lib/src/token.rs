use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct Token {
    data: Arc<AtomicBool>,
}

impl Token {
    /// Test if the token is set.
    pub fn is_set(&self) -> bool {
        self.data.load(Ordering::Acquire)
    }

    /// Set the token.
    pub fn set(&self) {
        self.data.store(true, Ordering::Release);
    }
}
