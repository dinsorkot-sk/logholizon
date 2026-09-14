use crate::error::AppError;
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::time::{Duration, Instant};

pub const MAX_WEBHOOK_BODY_BYTES: usize = 1_000_000;
pub const MAX_HEADER_COUNT: usize = 32;
pub const MAX_HEADER_VALUE_BYTES: usize = 4096;

pub fn validate_outbound_url(raw: &str) -> anyhow::Result<()> {
    let value = raw.trim();
    let url = reqwest::Url::parse(value)
        .map_err(|_| AppError::BadRequest("invalid outbound URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::BadRequest("outbound URL must use http or https".into()).into());
    }
    if url.username() != "" || url.password().is_some() || url.fragment().is_some() {
        return Err(
            AppError::BadRequest("outbound URL contains forbidden components".into()).into(),
        );
    }
    let host = url
        .host_str()
        .ok_or_else(|| AppError::BadRequest("outbound URL requires a host".into()))?;
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return Err(AppError::BadRequest("outbound URL targets localhost".into()).into());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_private_or_local(ip) {
            return Err(AppError::BadRequest(
                "outbound URL targets a private or local address".into(),
            )
            .into());
        }
    }
    Ok(())
}

fn is_private_or_local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_unspecified()
                || ip.octets()[0] == 0
                || (224..=239).contains(&ip.octets()[0])
        }
        IpAddr::V6(ip) => {
            ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() || is_ipv6_private(ip)
        }
    }
}

fn is_ipv6_private(ip: std::net::Ipv6Addr) -> bool {
    let segments = ip.segments();
    (segments[0] & 0xfe00) == 0xfc00 || (segments[0] & 0xffc0) == 0xfe80
}

pub fn validate_webhook_headers(headers: &serde_json::Value) -> anyhow::Result<()> {
    let object = headers
        .as_object()
        .ok_or_else(|| AppError::BadRequest("webhook headers must be an object".into()))?;
    if object.len() > MAX_HEADER_COUNT {
        return Err(AppError::BadRequest("too many webhook headers".into()).into());
    }
    for (name, value) in object {
        let lower = name.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "host" | "content-length" | "transfer-encoding"
        ) {
            return Err(AppError::BadRequest(format!("forbidden webhook header: {name}")).into());
        }
        if value.as_str().is_none() || value.as_str().unwrap().len() > MAX_HEADER_VALUE_BYTES {
            return Err(AppError::BadRequest(format!("invalid webhook header: {name}")).into());
        }
    }
    Ok(())
}

pub fn validate_filename(filename: &str) -> anyhow::Result<()> {
    let name = filename.trim();
    if name.is_empty() || name.len() > 255 {
        return Err(AppError::BadRequest("invalid filename".into()).into());
    }
    if name.contains(['/', '\\']) || name.contains('\0') || name.chars().any(char::is_control) {
        return Err(
            AppError::BadRequest("filename contains forbidden path characters".into()).into(),
        );
    }
    if name == "." || name == ".." {
        return Err(AppError::BadRequest("invalid filename".into()).into());
    }
    Ok(())
}

/// Sliding-window rate limiter for auth endpoints (brute-force protection).
/// Keyed by client IP; only failed attempts count toward the limit so normal
/// logins are never throttled. Single-process in-memory state: sufficient for
/// the SQLite single-host deployment model; a multi-instance follow-up would
/// move this to shared storage (see Phase C Postgres readiness).
#[derive(Debug, Default)]
pub struct AuthRateLimiter {
    failures: HashMap<String, VecDeque<Instant>>,
}

impl AuthRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a failed auth attempt for `key` (client IP). Returns `true`
    /// when the key is now over `max_attempts` inside `window`.
    pub fn record_failure(&mut self, key: &str, max_attempts: u32, window: Duration) -> bool {
        let now = Instant::now();
        let entries = self.failures.entry(key.to_string()).or_default();
        while entries
            .front()
            .is_some_and(|t| now.duration_since(*t) > window)
        {
            entries.pop_front();
        }
        entries.push_back(now);
        entries.len() as u32 > max_attempts.max(1)
    }

    /// Check whether `key` is currently rate-limited (without recording).
    pub fn is_limited(&mut self, key: &str, max_attempts: u32, window: Duration) -> bool {
        let now = Instant::now();
        match self.failures.get_mut(key) {
            Some(entries) => {
                while entries
                    .front()
                    .is_some_and(|t| now.duration_since(*t) > window)
                {
                    entries.pop_front();
                }
                entries.len() as u32 > max_attempts.max(1)
            }
            None => false,
        }
    }

    /// Clear all recorded failures for `key` (called after a successful login).
    pub fn clear(&mut self, key: &str) {
        self.failures.remove(key);
    }
}

/// Extract the client IP for rate-limit keying. Prefers `X-Forwarded-For`
/// (first entry) when the server sits behind a proxy, else the socket peer.
/// Takes the raw header value so this module stays free of HTTP types.
pub fn client_ip_key(forwarded_for: Option<&str>, fallback: &str) -> String {
    if let Some(first) = forwarded_for
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return first.to_string();
    }
    fallback.to_string()
}

/// Verify an inbound webhook signature produced by [`crate::notification`]'s
/// HMAC-SHA256 signer (`sha256=<hex>` over the raw payload bytes).
/// Uses constant-time comparison so the secret cannot be probed byte-by-byte.
pub fn verify_webhook_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    use hmac::Mac;
    let expected = {
        let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(payload);
        mac.finalize().into_bytes()
    };
    let Some(hex) = signature.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(bytes) = hex_to_bytes(hex) else {
        return false;
    };
    use subtle::ConstantTimeEq;
    expected.as_slice().ct_eq(bytes.as_slice()).into()
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, ()> {
    if !hex.len().is_multiple_of(2) || hex.is_empty() {
        return Err(());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ()))
        .collect()
}
