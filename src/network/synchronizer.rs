use std::sync::{Arc, Barrier, Mutex};

pub struct Synchronizer {
    barrier: Arc<Barrier>,
    state: Arc<Mutex<Vec<bool>>>,
}

impl Synchronizer {
    pub fn new(barrier: Arc<Barrier>, state: Arc<Mutex<Vec<bool>>>) -> Self {
        Self { barrier, state }
    }

    pub fn barrier(&self) -> &Arc<Barrier> {
        &self.barrier
    }

    pub fn state(&self) -> Arc<Mutex<Vec<bool>>> {
        Arc::clone(&self.state)
    }
}
