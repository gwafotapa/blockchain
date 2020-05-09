use std::sync::{Arc, Barrier, Mutex};

pub struct Synchronizer {
    barrier: Arc<Barrier>,
    state: Arc<Mutex<Vec<bool>>>,
    shut_down: bool,
}

impl Synchronizer {
    pub fn new(barrier: Arc<Barrier>, state: Arc<Mutex<Vec<bool>>>) -> Self {
        Self {
            barrier,
            state,
            shut_down: false,
        }
    }

    pub fn barrier(&self) -> &Arc<Barrier> {
        &self.barrier
    }

    pub fn state(&self) -> Arc<Mutex<Vec<bool>>> {
        Arc::clone(&self.state)
    }

    pub fn shut_down(&mut self) {
        self.shut_down = true;
    }

    pub fn has_shut_down(&self) -> bool {
        self.shut_down
    }
}
