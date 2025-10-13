use base_crypto::time::Timestamp;

pub struct OnDrop<F: FnOnce()> {
    f: Option<F>,
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            f()
        }
    }
}

impl<F> OnDrop<F>
where
    F: FnOnce(),
{
    pub fn new(f: F) -> Self {
        Self { f: Some(f) }
    }
    pub fn cancel(&mut self) {
        self.f = None;
    }
}

pub fn get_current_time() -> Timestamp {
    Timestamp::from_secs(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_secs(),
    )
}
