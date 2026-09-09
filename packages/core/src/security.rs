use crate::error::AppError;
use std::net::IpAddr;

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
