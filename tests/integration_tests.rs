//! Integration tests for the PeerCat Rust SDK

use peercat::{
    CreateKeyParams, GenerateParams, HistoryParams, OnChainStatus, PeerCat, PeerCatConfig,
    PeerCatError, SubmitPromptParams,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a client configured for mock server
fn create_test_client(mock_server: &MockServer) -> PeerCat {
    PeerCat::with_config(
        PeerCatConfig::new("test_api_key")
            .with_base_url(&mock_server.uri())
            .with_max_retries(0),
    )
}

// ============ Generate Tests ============

#[tokio::test]
async fn test_generate_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .and(header("Authorization", "Bearer test_api_key"))
        .and(header("Content-Type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "gen_123",
            "imageUrl": "https://cdn.peerc.at/images/gen_123.png",
            "ipfsHash": "QmXyz123",
            "model": "stable-diffusion-xl",
            "mode": "production",
            "usage": {
                "creditsUsed": 0.28,
                "balanceRemaining": 9.72
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .generate(GenerateParams::new("A beautiful sunset"))
        .await
        .expect("Generate should succeed");

    assert_eq!(result.id, "gen_123");
    assert_eq!(result.image_url, "https://cdn.peerc.at/images/gen_123.png");
    assert_eq!(result.ipfs_hash, Some("QmXyz123".to_string()));
    assert_eq!(result.model, "stable-diffusion-xl");
    assert_eq!(result.usage.credits_used, 0.28);
}

#[tokio::test]
async fn test_generate_demo_mode() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "demo_123",
            "imageUrl": "https://cdn.peerc.at/demo/placeholder.png",
            "ipfsHash": null,
            "model": "stable-diffusion-xl",
            "mode": "demo",
            "usage": {
                "creditsUsed": 0.0,
                "balanceRemaining": 10.0
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .generate(GenerateParams::new("Test prompt").with_demo_mode())
        .await
        .expect("Generate should succeed");

    assert_eq!(result.id, "demo_123");
    assert!(result.ipfs_hash.is_none());
    assert_eq!(result.usage.credits_used, 0.0);
}

#[tokio::test]
async fn test_generate_with_model() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "gen_456",
            "imageUrl": "https://cdn.peerc.at/images/gen_456.png",
            "model": "imagen-3",
            "mode": "production",
            "usage": {
                "creditsUsed": 1.50,
                "balanceRemaining": 8.50
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .generate(GenerateParams::new("Test").with_model("imagen-3"))
        .await
        .expect("Generate should succeed");

    assert_eq!(result.model, "imagen-3");
}

// ============ Get Models Tests ============

#[tokio::test]
async fn test_get_models() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [
                {
                    "id": "stable-diffusion-xl",
                    "name": "Stable Diffusion XL",
                    "description": "High quality image generation",
                    "provider": "stability",
                    "maxPromptLength": 2000,
                    "outputFormat": "png",
                    "outputResolution": "1024x1024",
                    "priceUsd": 0.28
                },
                {
                    "id": "imagen-3",
                    "name": "Imagen 3",
                    "description": "Google's latest model",
                    "provider": "google",
                    "maxPromptLength": 1500,
                    "outputFormat": "png",
                    "outputResolution": "1024x1024",
                    "priceUsd": 1.50
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let models = client.get_models().await.expect("Get models should succeed");

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].id, "stable-diffusion-xl");
    assert_eq!(models[0].price_usd, 0.28);
    assert_eq!(models[1].id, "imagen-3");
}

// ============ Get Prices Tests ============

#[tokio::test]
async fn test_get_prices() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/price"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "solPrice": 185.50,
            "slippageTolerance": 0.05,
            "updatedAt": "2024-01-15T12:00:00Z",
            "treasury": "9JKi6Tr7JdsTJw1zNedF5vML9GpPnjHD9DWuZq1oE6nV",
            "models": [
                {
                    "model": "stable-diffusion-xl",
                    "priceUsd": 0.28,
                    "priceSol": 0.00151,
                    "priceSolWithSlippage": 0.00159
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let prices = client.get_prices().await.expect("Get prices should succeed");

    assert_eq!(prices.sol_price, 185.50);
    assert_eq!(prices.slippage_tolerance, 0.05);
    assert_eq!(prices.treasury, "9JKi6Tr7JdsTJw1zNedF5vML9GpPnjHD9DWuZq1oE6nV");
    assert_eq!(prices.models.len(), 1);
    assert_eq!(prices.models[0].model, "stable-diffusion-xl");
}

// ============ Get Balance Tests ============

#[tokio::test]
async fn test_get_balance() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "credits": 10.50,
            "totalDeposited": 50.00,
            "totalSpent": 39.50,
            "totalWithdrawn": 0.00,
            "totalGenerated": 100
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let balance = client
        .get_balance()
        .await
        .expect("Get balance should succeed");

    assert_eq!(balance.credits, 10.50);
    assert_eq!(balance.total_deposited, 50.00);
    assert_eq!(balance.total_spent, 39.50);
    assert_eq!(balance.total_generated, 100);
}

// ============ Get History Tests ============

#[tokio::test]
async fn test_get_history() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/history"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [
                {
                    "id": "use_123",
                    "endpoint": "/v1/generate",
                    "model": "stable-diffusion-xl",
                    "creditsUsed": 0.28,
                    "requestId": "gen_123",
                    "status": "completed",
                    "createdAt": "2024-01-15T10:00:00Z",
                    "completedAt": "2024-01-15T10:00:05Z"
                }
            ],
            "pagination": {
                "total": 100,
                "limit": 50,
                "offset": 0,
                "hasMore": true
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let history = client
        .get_history(HistoryParams::new())
        .await
        .expect("Get history should succeed");

    assert_eq!(history.items.len(), 1);
    assert_eq!(history.items[0].id, "use_123");
    assert_eq!(history.pagination.total, 100);
    assert!(history.pagination.has_more);
}

#[tokio::test]
async fn test_get_history_with_pagination() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/history"))
        .and(query_param("limit", "10"))
        .and(query_param("offset", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [],
            "pagination": {
                "total": 100,
                "limit": 10,
                "offset": 20,
                "hasMore": true
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let history = client
        .get_history(HistoryParams::new().with_limit(10).with_offset(20))
        .await
        .expect("Get history should succeed");

    assert_eq!(history.pagination.limit, 10);
    assert_eq!(history.pagination.offset, 20);
}

// ============ API Key Tests ============

#[tokio::test]
async fn test_list_keys() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "keys": [
                {
                    "id": "key_123",
                    "name": "Production Key",
                    "keyPrefix": "pcat_live_xx",
                    "environment": "live",
                    "rateLimitTier": "standard",
                    "createdAt": "2024-01-15T10:00:00Z",
                    "lastUsedAt": "2024-01-15T12:00:00Z",
                    "revoked": false
                }
            ]
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let keys = client.list_keys().await.expect("List keys should succeed");

    assert_eq!(keys.keys.len(), 1);
    assert_eq!(keys.keys[0].id, "key_123");
    assert_eq!(keys.keys[0].name, Some("Production Key".to_string()));
    assert!(!keys.keys[0].revoked);
}

#[tokio::test]
async fn test_create_key() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/keys"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "key_456",
            "key": "pcat_live_abc123xyz789",
            "keyPrefix": "pcat_live_abc",
            "name": "New Key",
            "environment": "live",
            "createdAt": "2024-01-15T14:00:00Z",
            "warning": "Store this key securely. It will not be shown again."
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .create_key(CreateKeyParams {
            name: Some("New Key".to_string()),
            message: "Create API key".to_string(),
            signature: "sig123".to_string(),
            public_key: "pubkey123".to_string(),
        })
        .await
        .expect("Create key should succeed");

    assert_eq!(result.id, "key_456");
    assert_eq!(result.key, "pcat_live_abc123xyz789");
}

#[tokio::test]
async fn test_revoke_key() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/v1/keys/key_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.revoke_key("key_123").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_key_name() {
    let mock_server = MockServer::start().await;

    Mock::given(method("PATCH"))
        .and(path("/v1/keys/key_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.update_key_name("key_123", "Updated Name").await;

    assert!(result.is_ok());
}

// ============ On-Chain Payment Tests ============

#[tokio::test]
async fn test_submit_prompt() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/prompts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "submissionId": "sub_123",
            "promptHash": "abc123def456",
            "paymentAddress": "9JKi6Tr7JdsTJw1zNedF5vML9GpPnjHD9DWuZq1oE6nV",
            "requiredAmount": {
                "sol": 0.00151,
                "lamports": 1510000,
                "usd": 0.28
            },
            "memo": "PCAT:v1:sdxl:abc123def456",
            "model": "stable-diffusion-xl",
            "slippageTolerance": 0.05,
            "expiresAt": "2024-01-15T11:00:00Z",
            "instructions": {
                "1": "Send SOL to payment address",
                "2": "Include memo in transaction"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .submit_prompt(SubmitPromptParams::new("A beautiful sunset"))
        .await
        .expect("Submit prompt should succeed");

    assert_eq!(result.submission_id, "sub_123");
    assert_eq!(result.memo, "PCAT:v1:sdxl:abc123def456");
    assert_eq!(result.required_amount.sol, 0.00151);
}

#[tokio::test]
async fn test_get_onchain_status_completed() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/generate/txSig123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "txSignature": "txSig123",
            "status": "completed",
            "model": "stable-diffusion-xl",
            "createdAt": "2024-01-15T10:00:00Z",
            "imageUrl": "https://cdn.peerc.at/images/gen_123.png",
            "ipfsHash": "QmXyz123",
            "completedAt": "2024-01-15T10:00:10Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let status = client
        .get_onchain_status("txSig123")
        .await
        .expect("Get status should succeed");

    assert_eq!(status.tx_signature, "txSig123");
    assert_eq!(status.status, OnChainStatus::Completed);
    assert_eq!(
        status.image_url,
        Some("https://cdn.peerc.at/images/gen_123.png".to_string())
    );
}

#[tokio::test]
async fn test_get_onchain_status_pending() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/generate/txSig456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "txSignature": "txSig456",
            "status": "pending",
            "model": "stable-diffusion-xl",
            "createdAt": "2024-01-15T10:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let status = client
        .get_onchain_status("txSig456")
        .await
        .expect("Get status should succeed");

    assert_eq!(status.status, OnChainStatus::Pending);
    assert!(status.image_url.is_none());
}

// ============ Error Handling Tests ============

#[tokio::test]
async fn test_authentication_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "type": "authentication_error",
                "code": "invalid_api_key",
                "message": "Invalid API key provided"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        PeerCatError::Authentication { ref code, ref message, .. } => {
            assert_eq!(code, "invalid_api_key");
            assert!(message.contains("Invalid API key"));
        }
        _ => panic!("Expected Authentication error, got {:?}", error),
    }

    assert!(!error.is_retryable());
}

#[tokio::test]
async fn test_insufficient_credits_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(402).set_body_json(serde_json::json!({
            "error": {
                "type": "insufficient_credits",
                "code": "insufficient_balance",
                "message": "Insufficient credits. Required: 0.28, Available: 0.10"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .generate(GenerateParams::new("Test"))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        PeerCatError::InsufficientCredits { ref code, .. } => {
            assert_eq!(code, "insufficient_balance");
        }
        _ => panic!("Expected InsufficientCredits error, got {:?}", error),
    }

    assert!(!error.is_retryable());
}

#[tokio::test]
async fn test_invalid_request_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "type": "invalid_request_error",
                "code": "invalid_prompt",
                "message": "Prompt cannot be empty",
                "param": "prompt"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client
        .generate(GenerateParams::new(""))
        .await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        PeerCatError::InvalidRequest { ref code, ref param, .. } => {
            assert_eq!(code, "invalid_prompt");
            assert_eq!(param, &Some("prompt".to_string()));
        }
        _ => panic!("Expected InvalidRequest error, got {:?}", error),
    }

    assert!(!error.is_retryable());
}

#[tokio::test]
async fn test_rate_limit_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "type": "rate_limit_error",
                "code": "rate_limit_exceeded",
                "message": "Rate limit exceeded. Try again in 30 seconds."
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match &error {
        PeerCatError::RateLimit { ref code, .. } => {
            assert_eq!(code, "rate_limit_exceeded");
        }
        _ => panic!("Expected RateLimit error, got {:?}", error),
    }

    assert!(error.is_retryable());
}

#[tokio::test]
async fn test_not_found_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/generate/invalid_tx"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": {
                "type": "not_found",
                "code": "generation_not_found",
                "message": "Generation not found"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_onchain_status("invalid_tx").await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match error {
        PeerCatError::NotFound { code, .. } => {
            assert_eq!(code, "generation_not_found");
        }
        _ => panic!("Expected NotFound error, got {:?}", error),
    }
}

#[tokio::test]
async fn test_server_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "type": "server_error",
                "code": "internal_error",
                "message": "Internal server error"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match error {
        PeerCatError::Server { status, .. } => {
            assert_eq!(status, 500);
        }
        _ => panic!("Expected Server error, got {:?}", error),
    }

    assert!(error.is_retryable());
}

// ============ Configuration Tests ============

#[tokio::test]
async fn test_custom_base_url() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "credits": 10.0,
            "totalDeposited": 10.0,
            "totalSpent": 0.0,
            "totalWithdrawn": 0.0,
            "totalGenerated": 0
        })))
        .mount(&mock_server)
        .await;

    let client = PeerCat::with_config(
        PeerCatConfig::new("test_key")
            .with_base_url(&format!("{}/", mock_server.uri())) // Trailing slash should be stripped
            .with_max_retries(0),
    );

    let result = client.get_balance().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_error_code_accessor() {
    let error = PeerCatError::Authentication {
        message: "test".to_string(),
        code: "invalid_key".to_string(),
        param: None,
    };

    assert_eq!(error.code(), Some("invalid_key"));

    let network_error = PeerCatError::Timeout;
    assert_eq!(network_error.code(), None);
}
