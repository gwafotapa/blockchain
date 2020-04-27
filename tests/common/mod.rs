use std::sync::Once;

static INIT: Once = Once::new();

pub fn log_setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}
