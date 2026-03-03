use actix_governor::{GovernorConfig, GovernorConfigBuilder, PeerIpKeyExtractor, KeyExtractor};
use actix_web::dev::ServiceRequest;
use actix_web::Error;

use crate::errors::ApiError;

/// Configures a rate limiter for API endpoints using actix-governor.
///
/// # Rate Limiting Strategy
///
/// This configuration implements a **token bucket algorithm** where:
/// - Clients receive a quota (`burst_size`) of request tokens
/// - Tokens replenish at a fixed interval (every `seconds` seconds)
/// - Requests exceeding the quota return HTTP 429 (Too Many Requests)
///
/// # Parameters
///
/// * `seconds` - Interval in seconds after which ONE token replenishes.
///               Example: `seconds = 2` means one token refills every 2 seconds.
/// * `burst_size` - Maximum number of tokens (concurrent requests) allowed.
///                  Clients can exhaust this quota in rapid succession, then must
///                  wait for token replenishment.
///
/// # Key Extractor: PeerIpKeyExtractor
///
/// Rate limits are enforced **per unique IP address**. Critical considerations:
///
/// - **Behind proxies/NATs**: Multiple users sharing the same IP will share the same quota
/// - **Rate limit independence**: Each IP address maintains its own separate token bucket
/// - **Production deployments**: Consider using alternative extractors:
///   - `HeaderExtractor`: Rate limit by API key or custom header
///   - Custom extractors: Rate limit by authenticated user ID (from JWT/session)
///
/// # Response Headers
///
/// With `use_headers()` enabled, responses include:
/// - `x-ratelimit-limit`: Maximum requests allowed in the time window
/// - `x-ratelimit-remaining`: Tokens remaining in current window
/// - `x-ratelimit-reset`: Unix timestamp when quota fully resets
/// - `x-ratelimit-after`: Seconds until next request is allowed (on 429 errors)
///
/// # Example Behavior
///
/// Configuration: `api_rate_limiter(2, 10)`
/// - Initial quota: 10 requests available immediately
/// - After exhausting quota: 1 request allowed every 2 seconds
/// - Full quota recovery: 20 seconds (10 tokens × 2 seconds/token)
///
/// # Returns
///
/// Returns a `GovernorConfig` ready to be applied as actix-web middleware:
/// ```rust
/// App::new().wrap(Governor::new(&api_rate_limiter(2, 10)))
/// ```
/// 
/// # Other example of use:
/// 
/// With: `api_rate_limiter(12, 1)`:
/// - Initial quota: 1 request available immediately
/// - After exhausting quota: 1 request allowed every 12 seconds (5 requests per minute)
/// - Full quota recovery: 12 seconds (1 token × 12 seconds/token)
/// This is ideal for very sensitive endpoints that require strict rate limiting, such as system information or administrative actions.
pub fn api_rate_limiter(
    seconds: u64,
    burst_size: u32,
) -> GovernorConfig<PeerIpKeyExtractor, actix_governor::governor::middleware::StateInformationMiddleware> {
    GovernorConfigBuilder::default()
        .per_second(seconds)
        .burst_size(burst_size)
        .key_extractor(PeerIpKeyExtractor)
        .use_headers()
        .finish()
        .expect("Failed to build rate limiter configuration: invalid parameters")
}

/// Personalised extractor that identifies unique devices
#[derive(Clone)]
pub struct DeviceIdExtractor;

impl KeyExtractor for DeviceIdExtractor {
    type Key = String;
    type KeyExtractionError = ApiError;

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        // 1. Try to read the unique device identifier from the headers
        if let Some(device_id) = req.headers().get("x-device-id") {
            if let Ok(id_str) = device_id.to_str() {
                // If we find the device identifier, this will be the key for the counter
                return Ok(id_str.to_string());
            }
        }

        // 2. Fallback: If the client is old or doesn't send the header, we fall back to using the IP address as the key. 
        let ip = req.connection_info().realip_remote_addr()
            .unwrap_or("unknown_ip")
            .to_string();

        Ok(ip)
    }
}

// Example of use of the custom extractor for device ID in a rate limiter configuration
pub fn api_rate_limiter_with_device_id(
    seconds: u64,
    burst_size: u32,
) -> GovernorConfig<DeviceIdExtractor, actix_governor::governor::middleware::StateInformationMiddleware> {
    GovernorConfigBuilder::default()
        .per_second(seconds)
        .burst_size(burst_size)
        .key_extractor(DeviceIdExtractor)
        .use_headers()
        .finish()
        .expect("Failed to build rate limiter configuration: invalid parameters")
}