//! # PeerCat SDK
//!
//! Official Rust SDK for the PeerCat AI image generation API.
//!
//! ## Quick Start
//!
//! ```no_run
//! use peercat::{PeerCat, GenerateParams};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client with your API key
//!     let client = PeerCat::new("pcat_live_xxx");
//!
//!     // Generate an image
//!     let result = client.generate(
//!         GenerateParams::new("A beautiful sunset over mountains")
//!             .with_model("stable-diffusion-xl")
//!     ).await?;
//!
//!     println!("Image URL: {}", result.image_url);
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! ```no_run
//! use peercat::{PeerCat, PeerCatConfig};
//!
//! let client = PeerCat::with_config(
//!     PeerCatConfig::new("pcat_live_xxx")
//!         .with_base_url("https://custom.api.url")
//!         .with_timeout(30)
//!         .with_max_retries(5)
//! );
//! ```
//!
//! ## Demo Mode
//!
//! Use demo mode to test without spending credits:
//!
//! ```no_run
//! use peercat::{PeerCat, GenerateParams};
//!
//! # async fn example() -> peercat::Result<()> {
//! let client = PeerCat::new("pcat_live_xxx");
//!
//! let result = client.generate(
//!     GenerateParams::new("Test prompt")
//!         .with_demo_mode()
//! ).await?;
//!
//! // Returns placeholder image, no credits charged
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! ```no_run
//! use peercat::{PeerCat, GenerateParams, PeerCatError};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = PeerCat::new("pcat_live_xxx");
//!
//! match client.generate(GenerateParams::new("test")).await {
//!     Ok(result) => println!("Image: {}", result.image_url),
//!     Err(PeerCatError::Authentication { message, .. }) => {
//!         eprintln!("Invalid API key: {}", message);
//!     }
//!     Err(PeerCatError::InsufficientCredits { message, .. }) => {
//!         eprintln!("Add more credits: {}", message);
//!     }
//!     Err(PeerCatError::RateLimit { retry_after, .. }) => {
//!         if let Some(secs) = retry_after {
//!             eprintln!("Rate limited, retry after {} seconds", secs);
//!         }
//!     }
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## On-Chain Payments
//!
//! For direct SOL payments without credits:
//!
//! ```no_run
//! use peercat::{PeerCat, SubmitPromptParams, OnChainStatus};
//!
//! # async fn example() -> peercat::Result<()> {
//! let client = PeerCat::new("pcat_live_xxx");
//!
//! // Step 1: Submit prompt and get payment details
//! let submission = client.submit_prompt(
//!     SubmitPromptParams::new("A majestic dragon")
//! ).await?;
//!
//! println!("Send {} SOL to {}", submission.required_amount.sol, submission.payment_address);
//! println!("Include memo: {}", submission.memo);
//!
//! // Step 2: After sending payment, check status
//! let status = client.get_onchain_status("txSignature...").await?;
//!
//! if status.status == OnChainStatus::Completed {
//!     println!("Image: {}", status.image_url.unwrap());
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod error;
mod types;

// Re-export main types
pub use client::PeerCat;
pub use error::{PeerCatError, Result};
pub use types::{
    // Configuration
    PeerCatConfig,
    // Models
    Model,
    ModelsResponse,
    // Pricing
    ModelPrice,
    PriceResponse,
    // Generation
    GenerateParams,
    GenerateResult,
    GenerateUsage,
    GenerationMode,
    // Account
    Balance,
    HistoryItem,
    HistoryParams,
    HistoryResponse,
    HistoryStatus,
    Pagination,
    // API Keys
    ApiKey,
    CreateKeyParams,
    CreateKeyResult,
    KeyEnvironment,
    KeysResponse,
    // On-Chain Payments
    OnChainGenerationStatus,
    OnChainStatus,
    PromptSubmission,
    RequiredAmount,
    SubmitPromptParams,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = PeerCatConfig::new("test_key")
            .with_base_url("https://custom.url")
            .with_timeout(30)
            .with_max_retries(5);

        assert_eq!(config.api_key, "test_key");
        assert_eq!(config.base_url, Some("https://custom.url".to_string()));
        assert_eq!(config.timeout, Some(30));
        assert_eq!(config.max_retries, Some(5));
    }

    #[test]
    fn test_generate_params_builder() {
        let params = GenerateParams::new("test prompt")
            .with_model("stable-diffusion-xl")
            .with_demo_mode();

        assert_eq!(params.prompt, "test prompt");
        assert_eq!(params.model, Some("stable-diffusion-xl".to_string()));
        assert_eq!(params.mode, Some(GenerationMode::Demo));
    }

    #[test]
    fn test_history_params_builder() {
        let params = HistoryParams::new().with_limit(10).with_offset(20);

        assert_eq!(params.limit, Some(10));
        assert_eq!(params.offset, Some(20));
    }

    #[test]
    fn test_submit_prompt_params_builder() {
        let params = SubmitPromptParams::new("test prompt")
            .with_model("stable-diffusion-xl")
            .with_callback_url("https://callback.url");

        assert_eq!(params.prompt, "test prompt");
        assert_eq!(params.model, Some("stable-diffusion-xl".to_string()));
        assert_eq!(params.callback_url, Some("https://callback.url".to_string()));
    }

    #[test]
    fn test_error_is_retryable() {
        let auth_error = PeerCatError::Authentication {
            message: "test".to_string(),
            code: "invalid_key".to_string(),
            param: None,
        };
        assert!(!auth_error.is_retryable());

        let server_error = PeerCatError::Server {
            message: "test".to_string(),
            code: "internal_error".to_string(),
            status: 500,
        };
        assert!(server_error.is_retryable());

        let rate_limit = PeerCatError::RateLimit {
            message: "test".to_string(),
            code: "rate_limit".to_string(),
            retry_after: Some(60),
        };
        assert!(rate_limit.is_retryable());
    }

    #[test]
    fn test_error_code() {
        let error = PeerCatError::Authentication {
            message: "test".to_string(),
            code: "invalid_key".to_string(),
            param: None,
        };
        assert_eq!(error.code(), Some("invalid_key"));
    }
}
