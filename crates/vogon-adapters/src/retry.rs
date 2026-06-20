use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const RETRY_BASE_DELAY: Duration = Duration::from_millis(100);
const RETRY_MAX_DELAY: Duration = Duration::from_secs(2);
const RETRY_JITTER_RANGE: u64 = 50;

pub(crate) fn sleep_before_retry(attempt_index: u32) {
    thread::sleep(retry_delay(attempt_index));
}

fn retry_delay(attempt_index: u32) -> Duration {
    let multiplier = 1_u32.checked_shl(attempt_index.min(5)).unwrap_or(u32::MAX);
    let exponential_delay = RETRY_BASE_DELAY
        .saturating_mul(multiplier)
        .min(RETRY_MAX_DELAY);

    exponential_delay + Duration::from_millis(retry_jitter_millis(attempt_index))
}

fn retry_jitter_millis(attempt_index: u32) -> u64 {
    let now_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| u64::from(duration.subsec_millis()))
        .unwrap_or(0);

    (now_millis + u64::from(attempt_index) * 37) % RETRY_JITTER_RANGE
}

#[cfg(test)]
mod tests {
    use super::{RETRY_BASE_DELAY, RETRY_MAX_DELAY, retry_delay};

    #[test]
    fn retry_delay_uses_exponential_backoff() {
        assert!(retry_delay(1) >= RETRY_BASE_DELAY * 2);
        assert!(retry_delay(2) >= RETRY_BASE_DELAY * 4);
    }

    #[test]
    fn retry_delay_caps_exponential_component() {
        assert!(retry_delay(20) < RETRY_MAX_DELAY * 2);
    }
}
