use std::time::Instant;

/// Default maximum number of tokens the bucket can hold.
/// This basically controls how many requests are allowed in a burst.
pub const DEFAULT_CAPACITY: f64 = 10.0;

/// Default rate at which tokens are added back to the bucket.
/// Here it means 2 tokens are added every second.
pub const DEFAULT_REFILL_RATE: f64 = 2.0;

/// A simple implementation of the Token Bucket rate limiting algorithm.
///
/// The idea?
///  The bucket holds a limited number of tokens.
///  Every request needs 1 token to proceed.
///  Tokens slowly refill over time.
///  If the bucket is empty, the request is rejected.
///
/// This helps prevent too many requests from hitting the server at once.
pub struct TokenBucket {
    /// Current number of tokens available
    tokens: f64,

    /// Maximum tokens the bucket can hold
    capacity: f64,

    /// How fast tokens refill per second
    refill_rate: f64,

    /// The last time the bucket was refilled
    last_refill: Instant,
}

impl TokenBucket {

    /// The bucket starts full so initial requests are allowed.
    pub fn new(capacity: f64, refill_rate: f64) -> Self {
        TokenBucket {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Checks whether a request is allowed.
    /// Returns:
    /// - `true` -> request allowed
    /// - `false` -> request rejected (rate limit hit)
    pub fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Refill tokens based on elapsed time
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);

        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}