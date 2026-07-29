use std::{
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const RETRY_BASE_DELAY: Duration = Duration::from_millis(100);
const RETRY_MAX_DELAY: Duration = Duration::from_secs(2);
const RETRY_JITTER_RANGE: u64 = 50;
const RETRY_AFTER_MAX_DELAY: Duration = Duration::from_secs(30);

pub(crate) fn is_retryable_error(error: &ureq::Error) -> bool {
    matches!(
        error,
        ureq::Error::Protocol(_)
            | ureq::Error::Io(_)
            | ureq::Error::Timeout(_)
            | ureq::Error::HostNotFound
            | ureq::Error::ConnectionFailed
            | ureq::Error::ConnectProxyFailed(_)
    )
}

pub(crate) fn sleep_before_retry(attempt_index: u32, retry_after: Option<&str>) {
    thread::sleep(retry_delay(attempt_index, retry_after, SystemTime::now()));
}

fn retry_delay(attempt_index: u32, retry_after: Option<&str>, now: SystemTime) -> Duration {
    if let Some(delay) = retry_after.and_then(|value| retry_after_delay(value, now)) {
        return delay;
    }

    let multiplier = 1_u32.checked_shl(attempt_index.min(5)).unwrap_or(u32::MAX);
    let exponential_delay = RETRY_BASE_DELAY
        .saturating_mul(multiplier)
        .min(RETRY_MAX_DELAY);

    exponential_delay + Duration::from_millis(retry_jitter_millis(attempt_index))
}

fn retry_after_delay(value: &str, now: SystemTime) -> Option<Duration> {
    let value = value.trim();
    let delay = if let Ok(seconds) = value.parse::<u64>() {
        Duration::from_secs(seconds)
    } else {
        let retry_at = httpdate::parse_http_date(value).ok()?;
        retry_at.duration_since(now).unwrap_or(Duration::ZERO)
    };

    Some(delay.min(RETRY_AFTER_MAX_DELAY))
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
    use std::time::{Duration, SystemTime};

    use super::{
        RETRY_AFTER_MAX_DELAY, RETRY_BASE_DELAY, RETRY_MAX_DELAY, is_retryable_error,
        retry_after_delay, retry_delay,
    };

    #[test]
    fn transient_transport_errors_are_retryable() {
        let errors = [
            ureq::Error::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "connection reset",
            )),
            ureq::Error::Timeout(ureq::Timeout::Global),
            ureq::Error::HostNotFound,
            ureq::Error::ConnectionFailed,
            ureq::Error::ConnectProxyFailed("proxy unavailable".to_owned()),
        ];

        assert!(errors.iter().all(is_retryable_error));
    }

    #[test]
    fn permanent_transport_errors_are_not_retryable() {
        let errors = [
            ureq::Error::BadUri("missing host".to_owned()),
            ureq::Error::InvalidProxyUrl,
            ureq::Error::RedirectFailed,
            ureq::Error::TooManyRedirects,
            ureq::Error::BodyExceedsLimit(1024),
            ureq::Error::Tls("certificate rejected"),
            ureq::Error::TlsRequired,
        ];

        assert!(errors.iter().all(|error| !is_retryable_error(error)));
    }

    #[test]
    fn retry_delay_uses_exponential_backoff() {
        assert!(retry_delay(1, None, SystemTime::now()) >= RETRY_BASE_DELAY * 2);
        assert!(retry_delay(2, None, SystemTime::now()) >= RETRY_BASE_DELAY * 4);
    }

    #[test]
    fn retry_delay_caps_exponential_component() {
        assert!(retry_delay(20, None, SystemTime::now()) < RETRY_MAX_DELAY * 2);
    }

    #[test]
    fn retry_after_accepts_delta_seconds() {
        assert_eq!(
            retry_after_delay("7", SystemTime::UNIX_EPOCH),
            Some(Duration::from_secs(7))
        );
    }

    #[test]
    fn retry_after_accepts_http_dates() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let retry_at = httpdate::fmt_http_date(now + Duration::from_secs(9));

        assert_eq!(
            retry_after_delay(&retry_at, now),
            Some(Duration::from_secs(9))
        );
    }

    #[test]
    fn retry_after_caps_server_directed_delays() {
        assert_eq!(
            retry_after_delay("3600", SystemTime::UNIX_EPOCH),
            Some(RETRY_AFTER_MAX_DELAY)
        );
    }

    #[test]
    fn retry_after_treats_past_dates_as_immediate() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        let retry_at = httpdate::fmt_http_date(now - Duration::from_secs(1));

        assert_eq!(retry_after_delay(&retry_at, now), Some(Duration::ZERO));
    }

    #[test]
    fn retry_delay_falls_back_for_invalid_retry_after() {
        assert!(retry_delay(1, Some("not-a-delay"), SystemTime::now()) >= RETRY_BASE_DELAY * 2);
    }
}
