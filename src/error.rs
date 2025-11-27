//! PeerCat SDK error types

use thiserror::Error;

/// Rate limit information from response headers
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    /// Maximum requests allowed in the window
    pub limit: Option<u32>,
    /// Remaining requests in the current window
    pub remaining: Option<u32>,
    /// Unix timestamp when the rate limit resets
    pub reset: Option<i64>,
    /// Seconds to wait before retrying (from Retry-After header)
    pub retry_after: Option<u64>,
}

impl RateLimitInfo {
    /// Parse rate limit information from HTTP headers
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Option<Self> {
        let limit = headers
            .get("X-RateLimit-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let remaining = headers
            .get("X-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let reset = headers
            .get("X-RateLimit-Reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let retry_after = headers
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        if limit.is_some() || remaining.is_some() || reset.is_some() || retry_after.is_some() {
            Some(Self {
                limit,
                remaining,
                reset,
                retry_after,
            })
        } else {
            None
        }
    }
}

/// All possible errors from the PeerCat SDK
#[derive(Error, Debug)]
pub enum PeerCatError {
    /// Authentication error (invalid or missing API key)
    #[error("Authentication error: {message}")]
    Authentication {
        message: String,
        code: String,
        param: Option<String>,
    },

    /// Invalid request error (bad parameters)
    #[error("Invalid request: {message}")]
    InvalidRequest {
        message: String,
        code: String,
        param: Option<String>,
    },

    /// Insufficient credits error
    #[error("Insufficient credits: {message}")]
    InsufficientCredits { message: String, code: String },

    /// Rate limit error
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        code: String,
        rate_limit_info: Option<RateLimitInfo>,
    },

    /// Resource not found
    #[error("Not found: {message}")]
    NotFound {
        message: String,
        code: String,
        param: Option<String>,
    },

    /// Server error
    #[error("Server error: {message}")]
    Server {
        message: String,
        code: String,
        status: u16,
    },

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Request timeout
    #[error("Request timed out")]
    Timeout,

    /// Unknown API error
    #[error("API error ({status}): {message}")]
    Unknown {
        status: u16,
        error_type: String,
        code: String,
        message: String,
        param: Option<String>,
    },
}

impl PeerCatError {
    /// Create an error from an API error response
    pub(crate) fn from_api_error(
        status: u16,
        error_type: String,
        code: String,
        message: String,
        param: Option<String>,
        rate_limit_info: Option<RateLimitInfo>,
    ) -> Self {
        match error_type.as_str() {
            "authentication_error" => PeerCatError::Authentication {
                message,
                code,
                param,
            },
            "invalid_request_error" => PeerCatError::InvalidRequest {
                message,
                code,
                param,
            },
            "insufficient_credits" => PeerCatError::InsufficientCredits { message, code },
            "rate_limit_error" => PeerCatError::RateLimit {
                message,
                code,
                rate_limit_info,
            },
            "not_found" => PeerCatError::NotFound {
                message,
                code,
                param,
            },
            _ if status >= 500 => PeerCatError::Server {
                message,
                code,
                status,
            },
            _ => PeerCatError::Unknown {
                status,
                error_type,
                code,
                message,
                param,
            },
        }
    }

    /// Returns the retry-after value in seconds if available
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            PeerCatError::RateLimit { rate_limit_info, .. } => {
                rate_limit_info.as_ref().and_then(|info| info.retry_after)
            }
            _ => None,
        }
    }

    /// Returns the rate limit info if this is a rate limit error
    pub fn rate_limit_info(&self) -> Option<&RateLimitInfo> {
        match self {
            PeerCatError::RateLimit { rate_limit_info, .. } => rate_limit_info.as_ref(),
            _ => None,
        }
    }

    /// Returns true if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            PeerCatError::Network(_)
                | PeerCatError::Timeout
                | PeerCatError::Server { .. }
                | PeerCatError::RateLimit { .. }
        )
    }

    /// Returns the error code if available
    pub fn code(&self) -> Option<&str> {
        match self {
            PeerCatError::Authentication { code, .. } => Some(code),
            PeerCatError::InvalidRequest { code, .. } => Some(code),
            PeerCatError::InsufficientCredits { code, .. } => Some(code),
            PeerCatError::RateLimit { code, .. } => Some(code),
            PeerCatError::NotFound { code, .. } => Some(code),
            PeerCatError::Server { code, .. } => Some(code),
            PeerCatError::Unknown { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Returns the parameter that caused the error, if available
    pub fn param(&self) -> Option<&str> {
        match self {
            PeerCatError::Authentication { param, .. } => param.as_deref(),
            PeerCatError::InvalidRequest { param, .. } => param.as_deref(),
            PeerCatError::NotFound { param, .. } => param.as_deref(),
            PeerCatError::Unknown { param, .. } => param.as_deref(),
            _ => None,
        }
    }
}

/// Result type for PeerCat operations
pub type Result<T> = std::result::Result<T, PeerCatError>;
