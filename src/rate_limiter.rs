use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Reddit returns headers from requests sent to its API that gives us
/// rate limit information, such as how many requests we have remaining
/// in this instance of time, and when the number of requests we can make
/// resets. This file deals with parsing these headers to limit our outbound
/// requests.

/// The remaining requests we can make
const HEADER_RATELIMIT_REMAINING: &str = "x-ratelimit-remaining";
/// The time at which our remining requests will reset
const HEADER_RATELIMIT_RESET: &str = "x-ratelimit-reset";

#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// The amount of requests remaining for this portion of time.
    remaining: u32,
    /// When the amount of requests we can make resets.
    reset_time: Instant,
}

impl Default for RateLimitState {
    fn default() -> Self {
        Self {
            remaining: 1,
            reset_time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    state: Arc<Mutex<RateLimitState>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimitState::default())),
        }
    }

    pub async fn wait(&self) {
        let mut state = self.state.lock().unwrap();

        if state.remaining > 0 && state.reset_time > Instant::now() {
            state.remaining -= 1;
            return;
        }

        let now = Instant::now();
        if state.reset_time > now {
            let sleep_duration = state.reset_time - now;
            println!(
                "[RateLimiter] Out of requests. Sleeping for {:.2} seconds",
                sleep_duration.as_secs_f32()
            );
            sleep(sleep_duration).await;
        }
    }

    pub fn update(&self, headers: &reqwest::header::HeaderMap) {
        let mut state = self.state.lock().unwrap();

        if let Some(remaining_str) = headers
            .get(HEADER_RATELIMIT_REMAINING)
            .and_then(|v| v.to_str().ok())
        {
            if let Ok(remaining) = remaining_str.split('.').next().unwrap_or("").parse::<u32>() {
                state.remaining = remaining;
            }
        }

        if let Some(reset_str) = headers
            .get(HEADER_RATELIMIT_RESET)
            .and_then(|v| v.to_str().ok())
        {
            if let Ok(reset_seconds) = reset_str.parse::<u64>() {
                state.reset_time = Instant::now() + Duration::from_secs(reset_seconds);
            }
        }

        println!(
            "[RateLimiter] State updated: {} requests remaining, resets in {:.2}s",
            state.remaining,
            (state.reset_time - Instant::now()).as_secs_f32()
        )
    }
}
