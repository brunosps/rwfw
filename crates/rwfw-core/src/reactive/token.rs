use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;

const SUPPORTED_VERSION: u8 = 1;

type HmacSha256 = Hmac<Sha256>;

/// Signed reactive component identity stored in the DOM.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Payload {
    pub v: u8,
    pub c: String,
    pub s: Value,
}

impl Payload {
    pub fn new(component: impl Into<String>, state: Value) -> Self {
        Self {
            v: SUPPORTED_VERSION,
            c: component.into(),
            s: state,
        }
    }
}

/// Token verification failures. Endpoint code maps every variant to 400.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TokenError {
    #[error("token is malformed")]
    Malformed,
    #[error("token payload is not valid base64")]
    InvalidPayloadEncoding,
    #[error("token signature is not valid base64")]
    InvalidSignatureEncoding,
    #[error("token payload is not valid JSON")]
    InvalidJson,
    #[error("token signature is invalid")]
    InvalidSignature,
    #[error("token version is not supported")]
    UnsupportedVersion,
    #[error("token could not be signed")]
    SigningFailed,
}

/// Sign a compact JSON payload with HMAC-SHA256 and return a base64-url token.
pub fn sign(payload: &Payload, key: &[u8]) -> Result<String, TokenError> {
    let json = serde_json::to_vec(payload).map_err(|_| TokenError::InvalidJson)?;
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| TokenError::SigningFailed)?;
    mac.update(&json);
    let signature = mac.finalize().into_bytes();
    Ok(format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(json),
        URL_SAFE_NO_PAD.encode(signature)
    ))
}

/// Verify a signed token and return its decoded payload.
pub fn verify(token: &str, key: &[u8]) -> Result<Payload, TokenError> {
    let (payload_part, signature_part) =
        token.split_once('.').ok_or(TokenError::Malformed)?;
    if payload_part.contains('.') || signature_part.contains('.') || signature_part.is_empty() {
        return Err(TokenError::Malformed);
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_part)
        .map_err(|_| TokenError::InvalidPayloadEncoding)?;
    let signature = URL_SAFE_NO_PAD
        .decode(signature_part)
        .map_err(|_| TokenError::InvalidSignatureEncoding)?;

    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| TokenError::SigningFailed)?;
    mac.update(&payload_bytes);
    mac.verify_slice(&signature)
        .map_err(|_| TokenError::InvalidSignature)?;

    let payload: Payload =
        serde_json::from_slice(&payload_bytes).map_err(|_| TokenError::InvalidJson)?;
    if payload.v != SUPPORTED_VERSION {
        return Err(TokenError::UnsupportedVersion);
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8] = b"test-key";

    #[test]
    fn sign_verify_roundtrip() {
        let payload = Payload::new("counter", serde_json::json!({ "count": 1 }));
        let token = sign(&payload, KEY).unwrap();
        assert_eq!(verify(&token, KEY).unwrap(), payload);
    }

    #[test]
    fn tamper_fails() {
        let payload = Payload::new("counter", serde_json::json!({ "count": 1 }));
        let mut token = sign(&payload, KEY).unwrap();
        token.push('x');
        assert_eq!(verify(&token, KEY), Err(TokenError::InvalidSignature));
    }

    #[test]
    fn unknown_version_fails_closed() {
        let payload = Payload {
            v: 2,
            c: "counter".to_string(),
            s: serde_json::json!({ "count": 1 }),
        };
        let token = sign(&payload, KEY).unwrap();
        assert_eq!(verify(&token, KEY), Err(TokenError::UnsupportedVersion));
    }
}
