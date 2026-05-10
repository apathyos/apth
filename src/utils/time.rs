use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_time_now() -> u128 {
    let result = SystemTime::now().duration_since(UNIX_EPOCH);

    match result {
        Ok(time) => time.as_millis(),
        _ => 0,
    }
}
