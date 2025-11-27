//! Schema validation tests to ensure SDK types match OpenAPI specification
//!
//! These tests validate:
//! 1. All required fields are present in response types
//! 2. Field types match the OpenAPI schema
//! 3. Enum values match OpenAPI enum definitions
//! 4. Nullable fields are correctly represented as Option<T>

use peercat::{
    Balance, GenerateResult, GenerateUsage, GenerationMode, HistoryItem, HistoryParams,
    HistoryResponse, HistoryStatus, KeyEnvironment, Model, ModelPrice, OnChainGenerationStatus,
    OnChainStatus, Pagination, PriceResponse, PromptSubmission, RequiredAmount,
};
use serde_json::json;

// ============ Deserialization Tests ============
// These tests verify that JSON matching the OpenAPI spec deserializes correctly

#[test]
fn test_model_deserialization() {
    let json = json!({
        "id": "stable-diffusion-xl",
        "name": "Stable Diffusion XL",
        "description": "High quality image generation",
        "provider": "stability",
        "maxPromptLength": 2000,
        "outputFormat": "png",
        "outputResolution": "1024x1024",
        "priceUsd": 0.28
    });

    let model: Model = serde_json::from_value(json).expect("Should deserialize Model");

    assert_eq!(model.id, "stable-diffusion-xl");
    assert_eq!(model.name, "Stable Diffusion XL");
    assert_eq!(model.description, "High quality image generation");
    assert_eq!(model.provider, "stability");
    assert_eq!(model.max_prompt_length, 2000);
    assert_eq!(model.output_format, "png");
    assert_eq!(model.output_resolution, "1024x1024");
    assert_eq!(model.price_usd, 0.28);
}

#[test]
fn test_balance_deserialization() {
    let json = json!({
        "credits": 10.50,
        "totalDeposited": 50.00,
        "totalSpent": 39.50,
        "totalWithdrawn": 0.00,
        "totalGenerated": 100
    });

    let balance: Balance = serde_json::from_value(json).expect("Should deserialize Balance");

    assert_eq!(balance.credits, 10.50);
    assert_eq!(balance.total_deposited, 50.00);
    assert_eq!(balance.total_spent, 39.50);
    assert_eq!(balance.total_withdrawn, 0.00);
    assert_eq!(balance.total_generated, 100);
}

#[test]
fn test_generate_result_production_mode() {
    let json = json!({
        "id": "gen_123",
        "imageUrl": "https://cdn.peerc.at/images/gen_123.png",
        "ipfsHash": "QmXyz123",
        "model": "stable-diffusion-xl",
        "mode": "production",
        "usage": {
            "creditsUsed": 0.28,
            "balanceRemaining": 9.72
        }
    });

    let result: GenerateResult =
        serde_json::from_value(json).expect("Should deserialize GenerateResult");

    assert_eq!(result.id, "gen_123");
    assert_eq!(result.mode, GenerationMode::Production);
    assert_eq!(result.ipfs_hash, Some("QmXyz123".to_string()));
}

#[test]
fn test_generate_result_demo_mode_null_ipfs() {
    let json = json!({
        "id": "demo_123",
        "imageUrl": "https://cdn.peerc.at/demo/placeholder.png",
        "ipfsHash": null,
        "model": "stable-diffusion-xl",
        "mode": "demo",
        "usage": {
            "creditsUsed": 0.0,
            "balanceRemaining": 10.0
        }
    });

    let result: GenerateResult =
        serde_json::from_value(json).expect("Should deserialize GenerateResult with null ipfsHash");

    assert_eq!(result.mode, GenerationMode::Demo);
    assert!(result.ipfs_hash.is_none());
    assert_eq!(result.usage.credits_used, 0.0);
}

#[test]
fn test_price_response_with_treasury() {
    // Treasury field is required per OpenAPI spec (our fix)
    let json = json!({
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
    });

    let response: PriceResponse =
        serde_json::from_value(json).expect("Should deserialize PriceResponse");

    assert_eq!(response.sol_price, 185.50);
    assert_eq!(response.slippage_tolerance, 0.05);
    assert_eq!(
        response.treasury,
        "9JKi6Tr7JdsTJw1zNedF5vML9GpPnjHD9DWuZq1oE6nV"
    );
    assert_eq!(response.models.len(), 1);
    assert_eq!(response.models[0].model, "stable-diffusion-xl");
}

#[test]
fn test_history_item_complete() {
    let json = json!({
        "id": "use_123",
        "endpoint": "/v1/generate",
        "model": "stable-diffusion-xl",
        "creditsUsed": 0.28,
        "requestId": "gen_123",
        "status": "completed",
        "createdAt": "2024-01-15T10:00:00Z",
        "completedAt": "2024-01-15T10:00:05Z"
    });

    let item: HistoryItem = serde_json::from_value(json).expect("Should deserialize HistoryItem");

    assert_eq!(item.id, "use_123");
    assert_eq!(item.status, HistoryStatus::Completed);
    assert_eq!(item.model, Some("stable-diffusion-xl".to_string()));
    assert!(item.completed_at.is_some());
}

#[test]
fn test_history_item_pending_null_fields() {
    let json = json!({
        "id": "use_456",
        "endpoint": "/v1/generate",
        "model": null,
        "creditsUsed": 0.0,
        "requestId": null,
        "status": "pending",
        "createdAt": "2024-01-15T10:00:00Z",
        "completedAt": null
    });

    let item: HistoryItem =
        serde_json::from_value(json).expect("Should deserialize HistoryItem with null fields");

    assert_eq!(item.status, HistoryStatus::Pending);
    assert!(item.model.is_none());
    assert!(item.request_id.is_none());
    assert!(item.completed_at.is_none());
}

#[test]
fn test_on_chain_generation_status_completed() {
    let json = json!({
        "txSignature": "txSig123abc",
        "status": "completed",
        "model": "stable-diffusion-xl",
        "createdAt": "2024-01-15T10:00:00Z",
        "imageUrl": "https://cdn.peerc.at/images/gen_123.png",
        "ipfsHash": "QmXyz123",
        "completedAt": "2024-01-15T10:00:10Z"
    });

    let status: OnChainGenerationStatus =
        serde_json::from_value(json).expect("Should deserialize OnChainGenerationStatus");

    assert_eq!(status.tx_signature, "txSig123abc");
    assert_eq!(status.status, OnChainStatus::Completed);
    assert!(status.image_url.is_some());
}

#[test]
fn test_on_chain_generation_status_pending_minimal() {
    let json = json!({
        "txSignature": "txSig456def",
        "status": "pending"
    });

    let status: OnChainGenerationStatus =
        serde_json::from_value(json).expect("Should deserialize minimal OnChainGenerationStatus");

    assert_eq!(status.status, OnChainStatus::Pending);
    assert!(status.image_url.is_none());
    assert!(status.completed_at.is_none());
}

// ============ Enum Value Tests ============

#[test]
fn test_generation_mode_enum_values() {
    // OpenAPI spec: enum: [production, demo]
    let production: GenerationMode = serde_json::from_str("\"production\"").unwrap();
    let demo: GenerationMode = serde_json::from_str("\"demo\"").unwrap();

    assert_eq!(production, GenerationMode::Production);
    assert_eq!(demo, GenerationMode::Demo);
}

#[test]
fn test_history_status_enum_values() {
    // OpenAPI spec: enum: [pending, completed, refunded]
    let pending: HistoryStatus = serde_json::from_str("\"pending\"").unwrap();
    let completed: HistoryStatus = serde_json::from_str("\"completed\"").unwrap();
    let refunded: HistoryStatus = serde_json::from_str("\"refunded\"").unwrap();

    assert_eq!(pending, HistoryStatus::Pending);
    assert_eq!(completed, HistoryStatus::Completed);
    assert_eq!(refunded, HistoryStatus::Refunded);
}

#[test]
fn test_key_environment_enum_values() {
    // OpenAPI spec: enum: [live, test]
    let live: KeyEnvironment = serde_json::from_str("\"live\"").unwrap();
    let test: KeyEnvironment = serde_json::from_str("\"test\"").unwrap();

    assert_eq!(live, KeyEnvironment::Live);
    assert_eq!(test, KeyEnvironment::Test);
}

#[test]
fn test_on_chain_status_enum_values() {
    // OpenAPI spec: enum: [pending, processing, completed, failed, refunded]
    let pending: OnChainStatus = serde_json::from_str("\"pending\"").unwrap();
    let processing: OnChainStatus = serde_json::from_str("\"processing\"").unwrap();
    let completed: OnChainStatus = serde_json::from_str("\"completed\"").unwrap();
    let failed: OnChainStatus = serde_json::from_str("\"failed\"").unwrap();
    let refunded: OnChainStatus = serde_json::from_str("\"refunded\"").unwrap();

    assert_eq!(pending, OnChainStatus::Pending);
    assert_eq!(processing, OnChainStatus::Processing);
    assert_eq!(completed, OnChainStatus::Completed);
    assert_eq!(failed, OnChainStatus::Failed);
    assert_eq!(refunded, OnChainStatus::Refunded);
}

// ============ Serialization Round-Trip Tests ============

#[test]
fn test_model_price_serialization_roundtrip() {
    let price = ModelPrice {
        model: "stable-diffusion-xl".to_string(),
        price_usd: 0.28,
        price_sol: 0.00151,
        price_sol_with_slippage: 0.00159,
    };

    let json = serde_json::to_value(&price).expect("Should serialize ModelPrice");
    let deserialized: ModelPrice =
        serde_json::from_value(json).expect("Should deserialize ModelPrice");

    assert_eq!(price.model, deserialized.model);
    assert_eq!(price.price_usd, deserialized.price_usd);
}

#[test]
fn test_pagination_serialization_roundtrip() {
    let pagination = Pagination {
        total: 100,
        limit: 50,
        offset: 0,
        has_more: true,
    };

    let json = serde_json::to_value(&pagination).expect("Should serialize Pagination");
    let deserialized: Pagination =
        serde_json::from_value(json).expect("Should deserialize Pagination");

    assert_eq!(pagination.total, deserialized.total);
    assert_eq!(pagination.has_more, deserialized.has_more);
}

// ============ Contract Tests ============
// These tests verify type structure matches OpenAPI at compile time

#[test]
fn test_contract_price_response_has_treasury() {
    // This test ensures our fix is in place - treasury must be present
    let response = PriceResponse {
        sol_price: 185.50,
        slippage_tolerance: 0.05,
        updated_at: "2024-01-15T12:00:00Z".to_string(),
        treasury: "9JKi6Tr7JdsTJw1zNedF5vML9GpPnjHD9DWuZq1oE6nV".to_string(),
        models: vec![],
    };

    assert!(!response.treasury.is_empty());
}

#[test]
fn test_contract_generate_usage_fields() {
    // Usage must have creditsUsed and balanceRemaining per OpenAPI
    let usage = GenerateUsage {
        credits_used: 0.28,
        balance_remaining: 9.72,
    };

    assert!(usage.credits_used >= 0.0);
    assert!(usage.balance_remaining >= 0.0);
}

#[test]
fn test_contract_required_amount_fields() {
    // RequiredAmount must have sol, lamports, and usd
    let amount = RequiredAmount {
        sol: 0.00151,
        lamports: 1510000,
        usd: 0.28,
    };

    assert!(amount.sol > 0.0);
    assert!(amount.lamports > 0);
    assert!(amount.usd > 0.0);
}

// ============ Invalid Input Tests ============

#[test]
fn test_invalid_generation_mode_fails() {
    let result: Result<GenerationMode, _> = serde_json::from_str("\"invalid\"");
    assert!(result.is_err(), "Invalid generation mode should fail to deserialize");
}

#[test]
fn test_invalid_history_status_fails() {
    let result: Result<HistoryStatus, _> = serde_json::from_str("\"cancelled\"");
    assert!(result.is_err(), "Invalid history status should fail to deserialize");
}

#[test]
fn test_invalid_key_environment_fails() {
    let result: Result<KeyEnvironment, _> = serde_json::from_str("\"development\"");
    assert!(result.is_err(), "Invalid key environment should fail to deserialize");
}

#[test]
fn test_invalid_on_chain_status_fails() {
    let result: Result<OnChainStatus, _> = serde_json::from_str("\"cancelled\"");
    assert!(result.is_err(), "Invalid on-chain status should fail to deserialize");
}
