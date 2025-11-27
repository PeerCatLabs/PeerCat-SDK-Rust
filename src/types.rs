//! PeerCat API types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============ Configuration ============

/// Configuration for the PeerCat client
#[derive(Debug, Clone)]
pub struct PeerCatConfig {
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API (default: https://api.peerc.at)
    pub base_url: Option<String>,
    /// Request timeout in seconds (default: 60)
    pub timeout: Option<u64>,
    /// Number of retry attempts for failed requests (default: 3)
    pub max_retries: Option<u32>,
}

impl PeerCatConfig {
    /// Create a new configuration with just an API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            timeout: None,
            max_retries: None,
        }
    }

    /// Set a custom base URL
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set a custom timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }
}

// ============ Models ============

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Model identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Model description
    pub description: String,
    /// Model provider
    pub provider: String,
    /// Maximum prompt length in characters
    pub max_prompt_length: u32,
    /// Output image format
    pub output_format: String,
    /// Output resolution
    pub output_resolution: String,
    /// Price in USD
    pub price_usd: f64,
}

/// Response containing available models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub models: Vec<Model>,
}

// ============ Pricing ============

/// Price information for a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPrice {
    /// Model identifier
    pub model: String,
    /// Price in USD
    pub price_usd: f64,
    /// Price in SOL
    pub price_sol: f64,
    /// Price in SOL including slippage tolerance
    pub price_sol_with_slippage: f64,
}

/// Response containing pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    /// Current SOL/USD price
    pub sol_price: f64,
    /// Slippage tolerance (e.g., 0.02 = 2%)
    pub slippage_tolerance: f64,
    /// Timestamp of price update
    pub updated_at: String,
    /// Treasury PDA address to send payments to
    pub treasury: String,
    /// Prices for each model
    pub models: Vec<ModelPrice>,
}

// ============ Generation ============

/// Generation mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GenerationMode {
    /// Production mode - uses credits
    #[default]
    Production,
    /// Demo mode - free, returns placeholder images
    Demo,
}

/// Parameters for image generation
#[derive(Debug, Clone, Serialize)]
pub struct GenerateParams {
    /// Text prompt for image generation (max 2000 characters)
    pub prompt: String,
    /// Model to use (default: stable-diffusion-xl)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Mode: production (default) or demo (free, placeholder images)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<GenerationMode>,
    /// Additional model-specific options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

impl GenerateParams {
    /// Create new generation parameters with just a prompt
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            mode: None,
            options: None,
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set to demo mode (free, placeholder images)
    pub fn with_demo_mode(mut self) -> Self {
        self.mode = Some(GenerationMode::Demo);
        self
    }

    /// Set to production mode (default)
    pub fn with_production_mode(mut self) -> Self {
        self.mode = Some(GenerationMode::Production);
        self
    }

    /// Add a custom option
    pub fn with_option(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        let options = self.options.get_or_insert_with(HashMap::new);
        options.insert(key.into(), value);
        self
    }
}

/// Usage information from a generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateUsage {
    /// Credits used for this generation
    pub credits_used: f64,
    /// Remaining credit balance
    pub balance_remaining: f64,
}

/// Result of an image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    /// Unique generation ID
    pub id: String,
    /// URL to the generated image
    pub image_url: String,
    /// IPFS hash (if uploaded)
    pub ipfs_hash: Option<String>,
    /// Model used
    pub model: String,
    /// Mode used
    pub mode: GenerationMode,
    /// Usage information
    pub usage: GenerateUsage,
}

// ============ Balance ============

/// Account balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// Current credit balance in USD
    pub credits: f64,
    /// Total amount deposited
    pub total_deposited: f64,
    /// Total amount spent
    pub total_spent: f64,
    /// Total amount withdrawn
    pub total_withdrawn: f64,
    /// Total number of generations
    pub total_generated: u64,
}

// ============ History ============

/// Parameters for fetching usage history
#[derive(Debug, Clone, Default, Serialize)]
pub struct HistoryParams {
    /// Number of items to return (default: 50, max: 100)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Pagination offset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

impl HistoryParams {
    /// Create new history parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the offset
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Status of a usage record
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryStatus {
    Pending,
    Completed,
    Refunded,
}

/// A single usage history item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    /// Usage record ID
    pub id: String,
    /// API endpoint called
    pub endpoint: String,
    /// Model used
    pub model: Option<String>,
    /// Credits used
    pub credits_used: f64,
    /// Request ID (for generation requests)
    pub request_id: Option<String>,
    /// Status
    pub status: HistoryStatus,
    /// Creation timestamp
    pub created_at: String,
    /// Completion timestamp
    pub completed_at: Option<String>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// Response containing usage history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    /// Usage history items
    pub items: Vec<HistoryItem>,
    /// Pagination info
    pub pagination: Pagination,
}

// ============ API Keys ============

/// Parameters for creating an API key
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyParams {
    /// Optional name for the key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Message to sign
    pub message: String,
    /// Wallet signature (base58)
    pub signature: String,
    /// Wallet public key (base58)
    pub public_key: String,
}

/// Environment type for API keys
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KeyEnvironment {
    Live,
    Test,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    /// Key ID
    pub id: String,
    /// Key name
    pub name: Option<String>,
    /// Key prefix (for display)
    pub key_prefix: String,
    /// Environment
    pub environment: KeyEnvironment,
    /// Rate limit tier
    pub rate_limit_tier: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last used timestamp
    pub last_used_at: Option<String>,
    /// Whether the key has been revoked
    pub revoked: bool,
}

/// Result of creating an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyResult {
    /// Key ID
    pub id: String,
    /// Full API key (only shown once!)
    pub key: String,
    /// Key prefix
    pub key_prefix: String,
    /// Key name
    pub name: Option<String>,
    /// Environment
    pub environment: KeyEnvironment,
    /// Creation timestamp
    pub created_at: String,
    /// Warning message
    pub warning: String,
}

/// Response containing API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeysResponse {
    pub keys: Vec<ApiKey>,
}

// ============ On-Chain Payments ============

/// Parameters for submitting a prompt for on-chain payment
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitPromptParams {
    /// Text prompt for image generation
    pub prompt: String,
    /// Model to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Additional options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<HashMap<String, serde_json::Value>>,
    /// Callback URL for result notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_url: Option<String>,
}

impl SubmitPromptParams {
    /// Create new prompt submission parameters
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            model: None,
            options: None,
            callback_url: None,
        }
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set a callback URL
    pub fn with_callback_url(mut self, url: impl Into<String>) -> Self {
        self.callback_url = Some(url.into());
        self
    }
}

/// Required payment amount in different units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredAmount {
    /// Amount in SOL
    pub sol: f64,
    /// Amount in lamports
    pub lamports: u64,
    /// Amount in USD
    pub usd: f64,
}

/// Result of submitting a prompt for on-chain payment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSubmission {
    /// Submission ID
    pub submission_id: String,
    /// Prompt hash (for memo)
    pub prompt_hash: String,
    /// Treasury address to send payment
    pub payment_address: String,
    /// Required payment amount
    pub required_amount: RequiredAmount,
    /// Memo to include in transaction
    pub memo: String,
    /// Model to use
    pub model: String,
    /// Slippage tolerance
    pub slippage_tolerance: f64,
    /// Expiration timestamp
    pub expires_at: String,
    /// Payment instructions
    pub instructions: HashMap<String, String>,
}

/// Status of an on-chain generation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OnChainStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Refunded,
}

/// Status and result of an on-chain generation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnChainGenerationStatus {
    /// Transaction signature
    pub tx_signature: String,
    /// Status
    pub status: OnChainStatus,
    /// Model used
    pub model: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Image URL (when completed)
    pub image_url: Option<String>,
    /// IPFS hash (when completed)
    pub ipfs_hash: Option<String>,
    /// Completion timestamp
    pub completed_at: Option<String>,
    /// Error message (when failed)
    pub error: Option<String>,
    /// Status message
    pub message: Option<String>,
}

// ============ Internal Types ============

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ApiErrorResponse {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ApiErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: String,
    pub message: String,
    pub param: Option<String>,
}

/// Simple success response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SuccessResponse {
    pub success: bool,
}
