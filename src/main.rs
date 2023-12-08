use std::error::Error;
use std::sync::Arc;

use axum::extract::{Extension, Json, Path};
use axum::{http::StatusCode, routing::post, Router};
use reqwest::Client;
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};
use serde_json::{self, json, Value};
use serde_yaml;

// Configuration struct
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Config {
    server: ServerConfig,
    discord: Vec<WebhookConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ServerConfig {
    port: String,
    host: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct WebhookConfig {
    name: String,
    path: String,
    url: String,
    logger: Option<bool>,
}

// Main entry point
#[tokio::main]
async fn main() {
    // Load configuration
    let config_file = std::fs::File::open("config.yml").expect("Failed to Find config.yml");
    let cfg: Config = serde_yaml::from_reader(config_file).expect("Failed to Read config.yml");

    // Create shared state
    let shared_cfg = Arc::new(cfg);
    println!("Config: {:?}", shared_cfg.clone());

    // Build our application with a route
    let app = Router::new()
        .route("/*path", post(generic_webhook_handler))
        .layer(Extension(shared_cfg.clone()));

    // Run our application
    let addr_string = format!("{}:{}", shared_cfg.server.host, shared_cfg.server.port);
    let listener = TcpListener::bind(addr_string).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn generic_webhook_handler(
    Path(path): Path<String>,
    Extension(cfg): Extension<Arc<Config>>,
    Json(payload): Json<Value>,
) -> StatusCode {
    // Print path requested by the client
    println!("Path: {:?}", path);

    // Find webhook that the matches path provided
    for webhook in cfg.discord.iter() {
        // If webhook path matches path provided send message to Webhook
        if webhook.path == "/".to_string() + &path {
            println!("Webhook we are using: {:?}", webhook);

            println!("Payload: {:?}", payload);

            // get the "data" feild from the JSON payload without the quotes
            let data = payload.get("data").unwrap().to_string();

            send_data_to_webhook(&webhook.url, data.trim())
                .await
                .unwrap();
        }
    }

    // Return a 200 OK
    StatusCode::OK
}

// Send a message to Discord
async fn send_data_to_webhook(webhook_url: &str, data: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let data = json!({
        "content": data
    });

    // send JSON to the webhook URL
    client.post(webhook_url).json(&data).send().await?;

    Ok(())
}

#[tokio::test]
async fn test_send_data_to_webhook() {
    let webhook_url = "https://discord.com/api/webhooks/1181408999732695070/P5yI267tzCAc-FU2zZqwBcgm_YSIfWCmqnd048HGm-6qsZ4462XEDHms87WrwH-7yQNz";
    let data = json!({
        "content": "Test"
    });

    let client = Client::new();

    client.post(webhook_url).json(&data).send().await.unwrap();
}
