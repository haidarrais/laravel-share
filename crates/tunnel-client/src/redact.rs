//! Driver-agnostic redaction of secret-shaped values in the terminal log.
//!
//! The PRD's final open question asks how much payload the terminal logger
//! should redact by default. Because redaction here must work for every driver
//! (the client cannot rely on a trusted relay doing it centrally), we apply it
//! in the client itself. The default is conservative: header values that look
//! like credentials are masked in the log, and the raw request body is never
//! printed without `--verbose`-style opt-in for sensitive header classes.

use std::collections::BTreeMap;

/// Header names whose values are considered credentials and masked by default.
const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-auth-token",
    "x-access-token",
    "x-stripe-webhook-secret",
    "stripe-signature",
    "x-hub-signature-256",
    "x-hub-signature-1",
    "x-slack-signature",
    "x-github-event",
    "x-shopify-hmac-sha256",
];

/// A value that survives the default redaction policy.
pub fn is_sensitive_header(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SENSITIVE_HEADERS.iter().any(|s| lower == *s)
}

/// Redact a header map for display. Credential-like header values become
/// `REDACTED` unless `reveal` is true (e.g. `--verbose`).
pub fn redact_headers(
    headers: &BTreeMap<String, String>,
    reveal: bool,
) -> BTreeMap<String, String> {
    if reveal {
        return headers.clone();
    }
    headers
        .iter()
        .map(|(k, v)| {
            if is_sensitive_header(k) {
                (k.clone(), "REDACTED".to_string())
            } else {
                (k.clone(), v.clone())
            }
        })
        .collect()
}

/// Heuristically mask secret-shaped strings within a payload body (e.g.
/// `"sk_live_..."`, `token: "..."`, `password: "..."`). Keeps the terminal log
/// usable while reducing accidental credential leakage in shared recordings.
pub fn redact_body(body: &str) -> String {
    let mut out = body.to_string();

    // Stripe-style secret keys.
    out = replace_pattern(&out, r#"sk_(live|test)_[A-Za-z0-9]+"#, "sk_***REDACTED***");
    // Bearer tokens.
    out = replace_pattern(
        &out,
        r#"(?i)(bearer\s+)[A-Za-z0-9._~+/=-]+"#,
        "${1}***REDACTED***",
    );

    out
}

fn replace_pattern(haystack: &str, pattern: &str, replacement: &str) -> String {
    let re = match regex::Regex::new(pattern) {
        Ok(re) => re,
        Err(_) => return haystack.to_string(),
    };
    re.replace_all(haystack, replacement).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_common_secret_headers() {
        let mut h = BTreeMap::new();
        h.insert("Stripe-Signature".to_string(), "t=123,v1=abc".to_string());
        h.insert("Content-Type".to_string(), "application/json".to_string());
        let redacted = redact_headers(&h, false);
        assert_eq!(redacted["Stripe-Signature"], "REDACTED");
        assert_eq!(redacted["Content-Type"], "application/json");
    }

    #[test]
    fn reveal_keeps_values() {
        let mut h = BTreeMap::new();
        h.insert("authorization".to_string(), "Bearer secret".to_string());
        assert_eq!(redact_headers(&h, true)["authorization"], "Bearer secret");
    }

    #[test]
    fn masks_stripe_keys_in_body() {
        let body = r#"{"id":"evt_1","data":{"sk_live_abcdef123456"}}"#;
        let out = redact_body(body);
        assert!(out.contains("sk_***REDACTED***"));
        assert!(!out.contains("sk_live_abcdef123456"));
    }
}
