//! Error scenario and resilience tests for the PeerCat Rust SDK
//!
//! These tests cover edge cases, network failures, malformed responses,
//! and retry/rate-limit behavior to ensure SDK robustness.

use peercat::{GenerateParams, PeerCat, PeerCatConfig, PeerCatError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a client configured for mock server
fn create_test_client(mock_server: &MockServer) -> PeerCat {
    PeerCat::with_config(
        PeerCatConfig::new("test_api_key")
            .with_base_url(&mock_server.uri())
            .with_max_retries(0),
    )
    .expect("Failed to create test client")
}

// ============ Malformed Response Tests ============

#[tokio::test]
async fn test_malformed_json_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json {"))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err(), "Expected error for malformed JSON response");
}

#[tokio::test]
async fn test_malformed_json_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(500).set_body_string("invalid error json"))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err(), "Expected error for malformed JSON error response");
}

#[tokio::test]
async fn test_empty_response_body() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_string(""))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    // Should either error or handle gracefully, not panic
    // Empty body will cause JSON parse error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_error_response_without_error_wrapper() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_json(serde_json::json!({"message": "Something went wrong"})),
        )
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err(), "Expected error for 500 response");
}

// ============ HTTP Status Code Tests ============

#[tokio::test]
async fn test_http_403_forbidden() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": {
                "type": "authentication_error",
                "code": "forbidden",
                "message": "Access denied"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err());
    let error = result.unwrap_err();

    match error {
        PeerCatError::Authentication { ref code, .. } => {
            assert_eq!(code, "forbidden");
        }
        _ => panic!("Expected Authentication error, got {:?}", error),
    }
}

#[tokio::test]
async fn test_http_404_not_found() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/generate/invalid_tx"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "error": {
                "type": "not_found",
                "code": "resource_not_found",
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
        PeerCatError::NotFound { ref code, .. } => {
            assert_eq!(code, "resource_not_found");
        }
        _ => panic!("Expected NotFound error, got {:?}", error),
    }
}

#[tokio::test]
async fn test_http_502_bad_gateway() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(502).set_body_json(serde_json::json!({
            "error": {
                "type": "server_error",
                "code": "bad_gateway",
                "message": "Bad gateway"
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
            assert_eq!(status, 502);
        }
        _ => panic!("Expected Server error, got {:?}", error),
    }
}

#[tokio::test]
async fn test_http_503_service_unavailable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(503).set_body_json(serde_json::json!({
            "error": {
                "type": "server_error",
                "code": "service_unavailable",
                "message": "Service temporarily unavailable"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_http_504_gateway_timeout() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(504).set_body_json(serde_json::json!({
            "error": {
                "type": "server_error",
                "code": "gateway_timeout",
                "message": "Gateway timeout"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let result = client.get_balance().await;

    assert!(result.is_err());
}

// ============ Error Property Tests ============

#[tokio::test]
async fn test_error_status_code() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "type": "authentication_error",
                "code": "invalid_api_key",
                "message": "Invalid API key"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let error = client.get_balance().await.unwrap_err();

    match &error {
        PeerCatError::Authentication { ref code, ref message, .. } => {
            assert_eq!(code, "invalid_api_key");
            assert!(message.contains("Invalid API key"));
        }
        _ => panic!("Expected Authentication error, got {:?}", error),
    }

    // Test helper methods
    assert_eq!(error.code(), Some("invalid_api_key"));
    assert!(!error.is_retryable());
}

#[tokio::test]
async fn test_error_with_param() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "type": "invalid_request_error",
                "code": "invalid_param",
                "message": "Model not found",
                "param": "model"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let error = client
        .generate(GenerateParams::new("test").with_model("invalid"))
        .await
        .unwrap_err();

    match &error {
        PeerCatError::InvalidRequest { ref param, .. } => {
            assert_eq!(param, &Some("model".to_string()));
        }
        _ => panic!("Expected InvalidRequest error, got {:?}", error),
    }

    assert_eq!(error.param(), Some("model"));
}

#[tokio::test]
async fn test_rate_limit_is_retryable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "type": "rate_limit_error",
                "code": "rate_limit_exceeded",
                "message": "Rate limited"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let error = client.get_balance().await.unwrap_err();

    match &error {
        PeerCatError::RateLimit { .. } => {}
        _ => panic!("Expected RateLimit error, got {:?}", error),
    }

    assert!(error.is_retryable());
}

#[tokio::test]
async fn test_server_error_is_retryable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(500).set_body_json(serde_json::json!({
            "error": {
                "type": "server_error",
                "code": "internal_error",
                "message": "Internal error"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let error = client.get_balance().await.unwrap_err();

    match &error {
        PeerCatError::Server { .. } => {}
        _ => panic!("Expected Server error, got {:?}", error),
    }

    assert!(error.is_retryable());
}

#[tokio::test]
async fn test_auth_error_not_retryable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": {
                "type": "authentication_error",
                "code": "invalid_key",
                "message": "Invalid"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let error = client.get_balance().await.unwrap_err();

    assert!(!error.is_retryable());
}

#[tokio::test]
async fn test_insufficient_credits_not_retryable() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(402).set_body_json(serde_json::json!({
            "error": {
                "type": "insufficient_credits",
                "code": "insufficient_balance",
                "message": "Not enough"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let error = client
        .generate(GenerateParams::new("test"))
        .await
        .unwrap_err();

    assert!(!error.is_retryable());
}

// ============ Edge Case Tests ============

#[tokio::test]
async fn test_very_long_prompt() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": {
                "type": "invalid_request_error",
                "code": "prompt_too_long",
                "message": "Prompt exceeds maximum length"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let long_prompt = "x".repeat(10000);
    let result = client.generate(GenerateParams::new(&long_prompt)).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        PeerCatError::InvalidRequest { ref code, .. } => {
            assert_eq!(code, "prompt_too_long");
        }
        e => panic!("Expected InvalidRequest error, got {:?}", e),
    }
}

#[tokio::test]
async fn test_special_characters_in_prompt() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "gen_123",
            "imageUrl": "https://example.com/image.png",
            "model": "stable-diffusion-xl",
            "mode": "production",
            "usage": {
                "creditsUsed": 0.05,
                "balanceRemaining": 9.95
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let special_prompt = r#"Test with "quotes" and <tags> and émojis 🎨"#;

    let result = client.generate(GenerateParams::new(special_prompt)).await;

    assert!(result.is_ok(), "Expected success with special characters");
    assert_eq!(result.unwrap().id, "gen_123");
}

#[tokio::test]
async fn test_unicode_in_prompt() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "gen_123",
            "imageUrl": "https://example.com/image.png",
            "model": "stable-diffusion-xl",
            "mode": "production",
            "usage": {
                "creditsUsed": 0.05,
                "balanceRemaining": 9.95
            }
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let unicode_prompt = "日本語テスト 中文测试 한국어테스트";

    let result = client.generate(GenerateParams::new(unicode_prompt)).await;

    assert!(result.is_ok(), "Expected success with unicode characters");
}

#[tokio::test]
async fn test_extra_fields_in_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "credits": 10.50,
            "totalDeposited": 50.00,
            "totalSpent": 39.50,
            "totalWithdrawn": 0.00,
            "totalGenerated": 100,
            "unexpectedField": "should be ignored",
            "anotherUnknown": {"nested": "data"}
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let balance = client.get_balance().await.expect("Should handle extra fields");

    assert_eq!(balance.credits, 10.50);
}

#[tokio::test]
async fn test_very_large_numeric_values() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "credits": 999999999.99,
            "totalDeposited": 1000000000.0,
            "totalSpent": 0.000001,
            "totalWithdrawn": 0.0,
            "totalGenerated": 9007199254740991_i64
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let balance = client.get_balance().await.expect("Should handle large values");

    assert_eq!(balance.credits, 999999999.99);
    assert_eq!(balance.total_generated, 9007199254740991);
}

#[tokio::test]
async fn test_zero_credits() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "credits": 0.0,
            "totalDeposited": 0.0,
            "totalSpent": 0.0,
            "totalWithdrawn": 0.0,
            "totalGenerated": 0
        })))
        .mount(&mock_server)
        .await;

    let client = create_test_client(&mock_server);
    let balance = client.get_balance().await.expect("Should handle zero values");

    assert_eq!(balance.credits, 0.0);
}

// ============ Configuration Error Tests ============

#[test]
fn test_empty_api_key_returns_error() {
    let result = PeerCat::new("");

    assert!(result.is_err());
    match result {
        Err(PeerCatError::EmptyApiKey) => {}
        Err(e) => panic!("Expected EmptyApiKey error, got {:?}", e),
        Ok(_) => panic!("Expected error for empty API key"),
    }
}

#[test]
fn test_valid_api_key_succeeds() {
    let result = PeerCat::new("pcat_test_key");

    assert!(result.is_ok());
}
