pub fn now() -> f64 {
    use std::time::*;
    SystemTime::UNIX_EPOCH.elapsed().unwrap().as_secs_f64()
}
