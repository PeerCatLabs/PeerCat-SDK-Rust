# peercat

Official Rust SDK for the PeerCat AI image generation API.

[![Crates.io](https://img.shields.io/crates/v/peercat.svg)](https://crates.io/crates/peercat)
[![Documentation](https://docs.rs/peercat/badge.svg)](https://docs.rs/peercat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
peercat = "0.1"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use peercat::{PeerCat, GenerateParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PeerCat::new("pcat_live_xxx");

    let result = client.generate(
        GenerateParams::new("A beautiful sunset over mountains")
            .with_model("stable-diffusion-xl")
    ).await?;

    println!("Image URL: {}", result.image_url);
    Ok(())
}
```

## Features

- Async/await support with Tokio
- Automatic retries with exponential backoff
- Strongly typed API responses
- Comprehensive error handling
- Builder pattern for configuration
- On-chain SOL payment support

## Configuration

```rust
use peercat::{PeerCat, PeerCatConfig};

let client = PeerCat::with_config(
    PeerCatConfig::new("pcat_live_xxx")
        .with_base_url("https://custom.api.url")
        .with_timeout(30)       // seconds
        .with_max_retries(5)
);
```

## API Reference

### Image Generation

```rust
use peercat::{PeerCat, GenerateParams, GenerationMode};

let client = PeerCat::new("pcat_live_xxx");

// Basic generation
let result = client.generate(
    GenerateParams::new("A futuristic cityscape")
).await?;

// With options
let result = client.generate(
    GenerateParams::new("A majestic dragon")
        .with_model("stable-diffusion-xl")
        .with_demo_mode()  // Free, returns placeholder
).await?;

println!("Image: {}", result.image_url);
println!("Credits used: {}", result.usage.credits_used);
```

### Models & Pricing

```rust
// List available models
let models = client.get_models().await?;
for model in models {
    println!("{}: ${}", model.id, model.price_usd);
}

// Get current prices (including SOL conversion)
let prices = client.get_prices().await?;
println!("SOL/USD: ${}", prices.sol_price);
```

### Account

```rust
// Get balance
let balance = client.get_balance().await?;
println!("Credits: ${}", balance.credits);

// Get usage history
use peercat::HistoryParams;

let history = client.get_history(
    HistoryParams::new().with_limit(10)
).await?;

for item in history.items {
    println!("{}: {} credits", item.endpoint, item.credits_used);
}
```

### API Keys

```rust
use peercat::CreateKeyParams;

// Create a new key (requires wallet signature)
let new_key = client.create_key(CreateKeyParams {
    name: Some("Production App".to_string()),
    message: "Create API key for PeerCat".to_string(),
    signature: "base58signature...".to_string(),
    public_key: "walletPublicKey...".to_string(),
}).await?;

// Warning: Full key only shown once!
println!("API Key: {}", new_key.key);

// List keys
let keys = client.list_keys().await?;

// Revoke a key
client.revoke_key("key_id").await?;
```

### On-Chain Payments

For direct SOL payments without credits:

```rust
use peercat::{PeerCat, SubmitPromptParams, OnChainStatus};

let client = PeerCat::new("pcat_live_xxx");

// Step 1: Submit prompt and get payment details
let submission = client.submit_prompt(
    SubmitPromptParams::new("A majestic dragon")
        .with_model("stable-diffusion-xl")
).await?;

println!("Send {} SOL to {}", submission.required_amount.sol, submission.payment_address);
println!("Include memo: {}", submission.memo);

// Step 2: After sending payment, check status
let status = client.get_onchain_status("txSignature...").await?;

match status.status {
    OnChainStatus::Completed => {
        println!("Image: {}", status.image_url.unwrap());
    }
    OnChainStatus::Pending | OnChainStatus::Processing => {
        println!("Still processing...");
    }
    OnChainStatus::Failed => {
        println!("Failed: {}", status.error.unwrap_or_default());
    }
    _ => {}
}
```

## Error Handling

```rust
use peercat::{PeerCat, GenerateParams, PeerCatError};

match client.generate(GenerateParams::new("test")).await {
    Ok(result) => println!("Image: {}", result.image_url),
    Err(PeerCatError::Authentication { message, .. }) => {
        eprintln!("Invalid API key: {}", message);
    }
    Err(PeerCatError::InsufficientCredits { message, .. }) => {
        eprintln!("Add more credits: {}", message);
    }
    Err(PeerCatError::RateLimit { retry_after, .. }) => {
        if let Some(secs) = retry_after {
            eprintln!("Rate limited, retry after {} seconds", secs);
        }
    }
    Err(PeerCatError::InvalidRequest { message, param, .. }) => {
        eprintln!("Invalid request: {} (param: {:?})", message, param);
    }
    Err(e) => eprintln!("Error: {}", e),
}

// Check if error is retryable
if let Err(e) = result {
    if e.is_retryable() {
        // Retry the request
    }
}
```

## TLS Features

By default, the SDK uses the system's native TLS. You can switch to rustls:

```toml
[dependencies]
peercat = { version = "0.1", features = ["rustls-tls"] }
```

Or explicitly use native TLS:

```toml
[dependencies]
peercat = { version = "0.1", features = ["native-tls"] }
```

## License

MIT
