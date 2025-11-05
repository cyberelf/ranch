//! SSRF (Server-Side Request Forgery) protection for webhook URLs
//!
//! This module provides validation to prevent SSRF attacks by blocking
//! webhook URLs that target private IP ranges, localhost, and cloud metadata endpoints.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

/// Validate a webhook URL against SSRF vulnerabilities
///
/// # Arguments
/// * `url` - The URL to validate
///
/// # Returns
/// Ok(()) if the URL is safe to use, Err with a description if it's potentially dangerous
///
/// # Security Checks
/// - Requires HTTPS scheme
/// - Blocks private IPv4 ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
/// - Blocks localhost (127.0.0.0/8)
/// - Blocks link-local addresses (169.254.0.0/16)
/// - Blocks AWS metadata endpoint (169.254.169.254)
/// - Blocks private IPv6 ranges (::1, fc00::/7, fe80::/10)
/// - Blocks IPv6 localhost
pub fn validate_webhook_url(url: &Url) -> Result<(), String> {
    // Require HTTPS
    if url.scheme() != "https" {
        return Err("Webhook URL must use HTTPS".to_string());
    }

    // Get host and validate based on type
    match url.host() {
        Some(url::Host::Ipv4(ipv4)) => validate_ipv4_address(&ipv4)?,
        Some(url::Host::Ipv6(ipv6)) => validate_ipv6_address(&ipv6)?,
        Some(url::Host::Domain(domain)) => validate_hostname(domain)?,
        None => return Err("Webhook URL must have a valid host".to_string()),
    }

    Ok(())
}

/// Validate an IPv4 address against SSRF vulnerabilities
fn validate_ipv4_address(ip: &Ipv4Addr) -> Result<(), String> {
    let octets = ip.octets();

    // Check for private ranges
    if is_private_ipv4(ip) {
        return Err("Webhook URL cannot target private IP addresses".to_string());
    }

    // Check for localhost
    if is_localhost_ipv4(ip) {
        return Err("Webhook URL cannot target localhost".to_string());
    }

    // Check for link-local addresses (169.254.0.0/16)
    if octets[0] == 169 && octets[1] == 254 {
        // Special check for AWS metadata endpoint
        if octets[2] == 169 && octets[3] == 254 {
            return Err(
                "Webhook URL cannot target AWS metadata endpoint (169.254.169.254)".to_string(),
            );
        }
        return Err("Webhook URL cannot target link-local addresses".to_string());
    }

    // Check for broadcast address
    if ip.is_broadcast() {
        return Err("Webhook URL cannot target broadcast address".to_string());
    }

    // Check for multicast
    if ip.is_multicast() {
        return Err("Webhook URL cannot target multicast addresses".to_string());
    }

    // Check for unspecified (0.0.0.0)
    if ip.is_unspecified() {
        return Err("Webhook URL cannot target unspecified address (0.0.0.0)".to_string());
    }

    Ok(())
}

/// Check if IPv4 address is private
fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    // 10.0.0.0/8
    octets[0] == 10
        // 172.16.0.0/12
        || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
        // 192.168.0.0/16
        || (octets[0] == 192 && octets[1] == 168)
}

/// Check if IPv4 address is localhost
fn is_localhost_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    // 127.0.0.0/8
    octets[0] == 127
}

/// Validate an IPv6 address against SSRF vulnerabilities
fn validate_ipv6_address(ip: &Ipv6Addr) -> Result<(), String> {
    // Check for localhost (::1)
    if ip.is_loopback() {
        return Err("Webhook URL cannot target localhost (IPv6 ::1)".to_string());
    }

    // Check for unspecified (::)
    if ip.is_unspecified() {
        return Err("Webhook URL cannot target unspecified address (IPv6 ::)".to_string());
    }

    // Check for unique local addresses (fc00::/7)
    let segments = ip.segments();
    if (segments[0] & 0xfe00) == 0xfc00 {
        return Err("Webhook URL cannot target unique local IPv6 addresses (fc00::/7)".to_string());
    }

    // Check for link-local addresses (fe80::/10)
    if (segments[0] & 0xffc0) == 0xfe80 {
        return Err("Webhook URL cannot target link-local IPv6 addresses (fe80::/10)".to_string());
    }

    // Check for multicast
    if ip.is_multicast() {
        return Err("Webhook URL cannot target multicast IPv6 addresses".to_string());
    }

    Ok(())
}

/// Validate a hostname against SSRF vulnerabilities
fn validate_hostname(hostname: &str) -> Result<(), String> {
    let lower = hostname.to_lowercase();

    // Block common localhost names
    if lower == "localhost"
        || lower.ends_with(".localhost")
        || lower == "localhost.localdomain"
    {
        return Err("Webhook URL cannot target localhost hostname".to_string());
    }

    // Block .local domains (often used for internal networks)
    if lower.ends_with(".local") {
        return Err("Webhook URL cannot target .local domains".to_string());
    }

    // Block .internal domains
    if lower.ends_with(".internal") {
        return Err("Webhook URL cannot target .internal domains".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // HTTPS validation tests
    #[test]
    fn test_require_https() {
        let url = Url::parse("http://example.com/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTPS"));
    }

    #[test]
    fn test_https_allowed() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_ok());
    }

    // IPv4 private range tests
    #[test]
    fn test_block_10_network() {
        let url = Url::parse("https://10.0.0.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("private"));
    }

    #[test]
    fn test_block_172_16_network() {
        let url = Url::parse("https://172.16.0.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("private"));
    }

    #[test]
    fn test_block_172_31_network() {
        let url = Url::parse("https://172.31.255.255/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("private"));
    }

    #[test]
    fn test_allow_172_32_network() {
        // 172.32.x.x is not in the private range (only 172.16-31)
        let url = Url::parse("https://172.32.0.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_192_168_network() {
        let url = Url::parse("https://192.168.1.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("private"));
    }

    // Localhost tests
    #[test]
    fn test_block_localhost_127() {
        let url = Url::parse("https://127.0.0.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    #[test]
    fn test_block_localhost_127_others() {
        let url = Url::parse("https://127.1.2.3/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    // Link-local tests
    #[test]
    fn test_block_link_local() {
        let url = Url::parse("https://169.254.1.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("link-local"));
    }

    #[test]
    fn test_block_aws_metadata() {
        let url = Url::parse("https://169.254.169.254/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("AWS metadata"));
    }

    // Broadcast and multicast tests
    #[test]
    fn test_block_broadcast() {
        let url = Url::parse("https://255.255.255.255/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("broadcast"));
    }

    #[test]
    fn test_block_multicast() {
        let url = Url::parse("https://224.0.0.1/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("multicast"));
    }

    #[test]
    fn test_block_unspecified_ipv4() {
        let url = Url::parse("https://0.0.0.0/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unspecified"));
    }

    // IPv6 tests
    #[test]
    fn test_block_ipv6_localhost() {
        let url = Url::parse("https://[::1]/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    #[test]
    fn test_block_ipv6_unspecified() {
        let url = Url::parse("https://[::]/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unspecified"));
    }

    #[test]
    fn test_block_ipv6_unique_local() {
        let url = Url::parse("https://[fc00::1]/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unique local"));
    }

    #[test]
    fn test_block_ipv6_link_local() {
        let url = Url::parse("https://[fe80::1]/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("link-local"));
    }

    #[test]
    fn test_block_ipv6_multicast() {
        let url = Url::parse("https://[ff02::1]/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("multicast"));
    }

    #[test]
    fn test_allow_public_ipv6() {
        let url = Url::parse("https://[2001:4860:4860::8888]/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_ok());
    }

    // Hostname tests
    #[test]
    fn test_block_localhost_hostname() {
        let url = Url::parse("https://localhost/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    #[test]
    fn test_block_localhost_subdomain() {
        let url = Url::parse("https://api.localhost/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    #[test]
    fn test_block_local_domain() {
        let url = Url::parse("https://myserver.local/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".local"));
    }

    #[test]
    fn test_block_internal_domain() {
        let url = Url::parse("https://api.internal/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains(".internal"));
    }

    // Public IP tests
    #[test]
    fn test_allow_public_ipv4() {
        let url = Url::parse("https://8.8.8.8/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_allow_public_domain() {
        let url = Url::parse("https://example.com/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_allow_public_subdomain() {
        let url = Url::parse("https://api.example.com/webhook").unwrap();
        let result = validate_webhook_url(&url);
        assert!(result.is_ok());
    }
}
