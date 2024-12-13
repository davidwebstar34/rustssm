use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
struct DeviceAuthorizationResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // Step 1: Request device authorization
    let device_auth_url = "https://sso.eu-central-1.amazonaws.com/start";
    let device_auth_request = serde_json::json!({
        "client_id": "<client-id>", // Replace with your client ID
        "client_secret": "<client-secret>", // Replace with your client secret
        "start_url": "https://webstar34.awsapps.com/start",
        "scopes": ["sso"]
    });

    println!("Requesting device authorization...");
    let device_auth_response = client
        .post(device_auth_url)
        .json(&device_auth_request)
        .send()
        .await?
        .json::<DeviceAuthorizationResponse>()
        .await?;

    println!(
        "Please go to {} and enter the code: {}",
        device_auth_response.verification_uri, device_auth_response.user_code
    );

    // Step 2: Poll for access token
    let token_url = "https://sso.eu-west-1.amazonaws.com/token";
    let mut access_token: Option<String> = None;

    println!("Waiting for user to authenticate...");
    for _ in 0..(device_auth_response.expires_in / device_auth_response.interval) {
        let token_response = client
            .post(token_url)
            .json(&serde_json::json!({
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
                "device_code": device_auth_response.device_code,
                "client_id": "<client-id>", // Replace with your client ID
                "client_secret": "<client-secret>" // Replace with your client secret
            }))
            .send()
            .await?;

        if token_response.status().is_success() {
            let token_data = token_response.json::<TokenResponse>().await?;
            access_token = Some(token_data.access_token);
            println!("Authentication successful!");
            break;
        } else if token_response.status() == 400 {
            let error_response = token_response.json::<serde_json::Value>().await?;
            if error_response["error"] == "authorization_pending" {
                tokio::time::sleep(Duration::from_secs(device_auth_response.interval)).await;
            } else {
                eprintln!("Error: {}", error_response["error_description"]);
                return Err("Authentication failed".into());
            }
        }
    }

    let access_token = access_token.ok_or("Failed to authenticate within the time limit")?;

    // Step 3: Query AWS SSO APIs
    let accounts_url = "https://sso.eu-central-1.amazonaws.com/accounts";
    let accounts_response = client
        .get(accounts_url)
        .bearer_auth(&access_token)
        .send()
        .await?
        .text()
        .await?;

    println!("Accounts accessible via SSO:\n{}", accounts_response);

    // Step 4: Save token to cache
    let cache_dir = dirs::home_dir()
        .unwrap()
        .join(".aws")
        .join("sso")
        .join("cache");
    fs::create_dir_all(&cache_dir)?;

    let token_path = cache_dir.join("cached_token.json");
    let mut token_file = fs::File::create(token_path)?;
    let token_data = serde_json::json!({
        "access_token": access_token,
        "expires_in": 3600 // Adjust based on your configuration
    });
    token_file.write_all(token_data.to_string().as_bytes())?;

    println!("Token saved to cache!");

    Ok(())
}
