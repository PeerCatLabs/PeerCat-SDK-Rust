//! PeerCat API client

use reqwest::{Client, StatusCode};
use std::time::Duration;

use crate::error::{PeerCatError, RateLimitInfo, Result};
use crate::types::*;

const DEFAULT_BASE_URL: &str = "https://api.peerc.at";
const DEFAULT_TIMEOUT: u64 = 60;
const DEFAULT_MAX_RETRIES: u32 = 3;
const USER_AGENT: &str = concat!("peercat-rust/", env!("CARGO_PKG_VERSION"));

/// PeerCat API client
///
/// # Example
///
/// ```no_run
/// use peercat::{PeerCat, GenerateParams};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = PeerCat::new("pcat_live_xxx")?;
///
///     let result = client.generate(
///         GenerateParams::new("A beautiful sunset over mountains")
///             .with_model("stable-diffusion-xl")
///     ).await?;
///
///     println!("Image URL: {}", result.image_url);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct PeerCat {
    api_key: String,
    base_url: String,
    client: Client,
    max_retries: u32,
}

impl PeerCat {
    /// Create a new PeerCat client with an API key
    ///
    /// # Errors
    ///
    /// Returns `PeerCatError::EmptyApiKey` if the API key is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::PeerCat;
    ///
    /// let client = PeerCat::new("pcat_live_xxx")?;
    /// # Ok::<(), peercat::PeerCatError>(())
    /// ```
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(PeerCatConfig::new(api_key))
    }

    /// Create a new PeerCat client with custom configuration
    ///
    /// # Errors
    ///
    /// Returns `PeerCatError::EmptyApiKey` if the API key is empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::{PeerCat, PeerCatConfig};
    ///
    /// let client = PeerCat::with_config(
    ///     PeerCatConfig::new("pcat_live_xxx")
    ///         .with_timeout(30)
    ///         .with_max_retries(5)
    /// )?;
    /// # Ok::<(), peercat::PeerCatError>(())
    /// ```
    pub fn with_config(config: PeerCatConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(PeerCatError::EmptyApiKey);
        }

        let timeout = config.timeout.unwrap_or(DEFAULT_TIMEOUT);
        let base_url = config
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .user_agent(USER_AGENT)
            .build()
            .expect("Failed to create HTTP client");

        Ok(Self {
            api_key: config.api_key,
            base_url,
            client,
            max_retries: config.max_retries.unwrap_or(DEFAULT_MAX_RETRIES),
        })
    }

    // ============ Image Generation ============

    /// Generate an image from a text prompt
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::{PeerCat, GenerateParams};
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    ///
    /// let result = client.generate(
    ///     GenerateParams::new("A futuristic cityscape at night")
    /// ).await?;
    ///
    /// println!("Image URL: {}", result.image_url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate(&self, params: GenerateParams) -> Result<GenerateResult> {
        self.post("/v1/generate", &params).await
    }

    // ============ Models & Pricing ============

    /// List available image generation models
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::PeerCat;
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    /// let models = client.get_models().await?;
    ///
    /// for model in models {
    ///     println!("{}: ${}", model.id, model.price_usd);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_models(&self) -> Result<Vec<Model>> {
        let response: ModelsResponse = self.get("/v1/models").await?;
        Ok(response.models)
    }

    /// Get current pricing for all models
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::PeerCat;
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    /// let prices = client.get_prices().await?;
    ///
    /// println!("SOL/USD: ${}", prices.sol_price);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_prices(&self) -> Result<PriceResponse> {
        self.get("/v1/price").await
    }

    // ============ Account ============

    /// Get current credit balance
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::PeerCat;
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    /// let balance = client.get_balance().await?;
    ///
    /// println!("Credits: ${}", balance.credits);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_balance(&self) -> Result<Balance> {
        self.get("/v1/balance").await
    }

    /// Get usage history
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::{PeerCat, HistoryParams};
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    ///
    /// let history = client.get_history(
    ///     HistoryParams::new().with_limit(10)
    /// ).await?;
    ///
    /// for item in history.items {
    ///     println!("{}: {} credits", item.endpoint, item.credits_used);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_history(&self, params: HistoryParams) -> Result<HistoryResponse> {
        let mut path = "/v1/history".to_string();
        let mut query_parts = Vec::new();

        if let Some(limit) = params.limit {
            query_parts.push(format!("limit={}", limit));
        }
        if let Some(offset) = params.offset {
            query_parts.push(format!("offset={}", offset));
        }

        if !query_parts.is_empty() {
            path = format!("{}?{}", path, query_parts.join("&"));
        }

        self.get(&path).await
    }

    // ============ API Keys ============

    /// Create a new API key (requires wallet signature)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::{PeerCat, CreateKeyParams};
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    ///
    /// let new_key = client.create_key(CreateKeyParams {
    ///     name: Some("Production App".to_string()),
    ///     message: "Create API key for PeerCat".to_string(),
    ///     signature: "base58signature...".to_string(),
    ///     public_key: "walletPublicKey...".to_string(),
    /// }).await?;
    ///
    /// // Warning: Full key is only shown once!
    /// println!("API Key: {}", new_key.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_key(&self, params: CreateKeyParams) -> Result<CreateKeyResult> {
        self.post("/v1/keys", &params).await
    }

    /// List all API keys for the authenticated wallet
    pub async fn list_keys(&self) -> Result<KeysResponse> {
        self.get("/v1/keys").await
    }

    /// Revoke an API key
    pub async fn revoke_key(&self, key_id: &str) -> Result<()> {
        let _: SuccessResponse = self.delete(&format!("/v1/keys/{}", key_id)).await?;
        Ok(())
    }

    /// Update API key name
    pub async fn update_key_name(&self, key_id: &str, name: &str) -> Result<()> {
        #[derive(serde::Serialize)]
        struct UpdateParams<'a> {
            name: &'a str,
        }

        let _: SuccessResponse = self
            .patch(&format!("/v1/keys/{}", key_id), &UpdateParams { name })
            .await?;
        Ok(())
    }

    // ============ On-Chain Payments ============

    /// Submit a prompt for on-chain payment
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::{PeerCat, SubmitPromptParams};
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    ///
    /// let submission = client.submit_prompt(
    ///     SubmitPromptParams::new("A majestic dragon")
    ///         .with_model("stable-diffusion-xl")
    /// ).await?;
    ///
    /// println!("Send {} SOL to {}", submission.required_amount.sol, submission.payment_address);
    /// println!("Memo: {}", submission.memo);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit_prompt(&self, params: SubmitPromptParams) -> Result<PromptSubmission> {
        self.post("/v1/prompts", &params).await
    }

    /// Get status of an on-chain generation by transaction signature
    ///
    /// # Example
    ///
    /// ```no_run
    /// use peercat::PeerCat;
    ///
    /// # async fn example() -> peercat::Result<()> {
    /// let client = PeerCat::new("pcat_live_xxx")?;
    ///
    /// let status = client.get_onchain_status("txSignature...").await?;
    ///
    /// match status.status {
    ///     peercat::OnChainStatus::Completed => {
    ///         println!("Image: {}", status.image_url.unwrap());
    ///     }
    ///     peercat::OnChainStatus::Pending => {
    ///         println!("Still processing...");
    ///     }
    ///     _ => {}
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_onchain_status(&self, tx_signature: &str) -> Result<OnChainGenerationStatus> {
        self.get(&format!("/v1/generate/{}", tx_signature)).await
    }

    // ============ Internal Methods ============

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(reqwest::Method::GET, path, None::<&()>).await
    }

    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.request(reqwest::Method::POST, path, Some(body)).await
    }

    async fn patch<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.request(reqwest::Method::PATCH, path, Some(body)).await
    }

    async fn delete<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(reqwest::Method::DELETE, path, None::<&()>)
            .await
    }

    async fn request<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut last_error: Option<PeerCatError> = None;

        for attempt in 0..=self.max_retries {
            let mut request = self
                .client
                .request(method.clone(), &url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json");

            if let Some(b) = body {
                request = request.json(b);
            }

            let result = request.send().await;

            match result {
                Ok(response) => {
                    let status = response.status();

                    // Parse rate limit headers
                    let rate_limit_info = RateLimitInfo::from_headers(response.headers());

                    if status.is_success() {
                        return response.json().await.map_err(|e| {
                            // Detect decode errors and convert to Json variant instead of Network
                            // reqwest::Error::is_decode() returns true for JSON deserialization failures
                            if e.is_decode() {
                                PeerCatError::Json(serde_json::Error::io(
                                    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()),
                                ))
                            } else {
                                PeerCatError::Network(e)
                            }
                        });
                    }

                    // Parse error response
                    let error_response: std::result::Result<ApiErrorResponse, _> =
                        response.json().await;

                    let error = match error_response {
                        Ok(err) => PeerCatError::from_api_error(
                            status.as_u16(),
                            err.error.error_type,
                            err.error.code,
                            err.error.message,
                            err.error.param,
                            rate_limit_info.clone(),
                        ),
                        Err(_) => PeerCatError::Unknown {
                            status: status.as_u16(),
                            error_type: "unknown".to_string(),
                            code: "parse_error".to_string(),
                            message: "Failed to parse error response".to_string(),
                            param: None,
                        },
                    };

                    // Don't retry client errors (4xx) except rate limits
                    if status.is_client_error() && status != StatusCode::TOO_MANY_REQUESTS {
                        return Err(error);
                    }

                    last_error = Some(error);
                }
                Err(e) => {
                    if e.is_timeout() {
                        last_error = Some(PeerCatError::Timeout);
                    } else {
                        last_error = Some(PeerCatError::Network(e));
                    }
                }
            }

            // Exponential backoff before retry (use Retry-After for rate limits)
            if attempt < self.max_retries {
                let mut delay = std::cmp::min(1000 * 2u64.pow(attempt), 10000);

                // Use Retry-After header if available for rate limit errors
                if let Some(ref error) = last_error {
                    if let Some(retry_after) = error.retry_after() {
                        delay = retry_after * 1000; // Convert seconds to milliseconds
                    }
                }

                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }

        Err(last_error.unwrap_or(PeerCatError::Timeout))
    }
}
