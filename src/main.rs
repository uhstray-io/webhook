mod config;

use std::error::Error;
use std::io::Write;
use std::sync::Arc;

use axum::extract::{Extension, Json, Path};
use axum::{http::StatusCode, routing::post, Router};
use reqwest::{Client, Response};
use tokio::net::TcpListener;

use serde_json::{self, json, Value};

use config::{Config, WebhookConfig};

// Main entry point
#[tokio::main]
async fn main() {
    // Load configuration
    let cfg = Config::load("config.yaml")
        .expect("Failed to load config")
        .shared();
    println!("Config: {:?}", cfg.clone());

    // Build our application with a route
    let app = Router::new()
        .route("/*path", post(generic_webhook_handler))
        .layer(Extension(cfg.clone()));

    // Run our application
    let addr_string = format!("{}:{}", cfg.server.host, cfg.server.port);
    println!("Listening on: {}", addr_string);
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
            println!("Payload: {:#?}", payload);

            log_payload(webhook, &payload);

            // get the "data" field from the JSON payload without the quotes
            let data = payload.get("data").unwrap().to_string();

            let res = send_data_to_webhook(&webhook.url, data.trim())
                .await
                .unwrap();

            return axum::http::StatusCode::from_u16(res.status().as_u16()).unwrap();
        }
    }

    StatusCode::NOT_FOUND
}

fn log_payload(webhook: &WebhookConfig, payload: &Value) {
    // If logging is enabled, save the payload to a file
    if webhook.logging.unwrap_or(false) {
        // Create a folder called "logs" if it doesn't exist
        std::fs::create_dir_all("logs").unwrap();

        // Get the current time and format it
        let time = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        let file_name = format!("logs/payload_{}.json", time);

        // Create a file with the current time as the name and write the payload to it
        let mut file = std::fs::File::create(file_name).unwrap();
        file.write_all(payload.to_string().as_bytes()).unwrap();
    }
}

// async fn log_payload_to_db(webhook: &WebhookConfig, payload: &Value) {
//     // If logging is enabled, save the payload to db
//     if webhook.logging.unwrap_or(false) {
//         // Create a SQLite database using sqlx if it doesn't exist

//         // sqlx::Sqlite::connect("sqlite:db.sqlite3").await.unwrap();

//         use sqlx::sqlite::Sqlite;

//         Sqlite::create_database("sqlite:db.sqlite3").await.unwrap();

//         // conn.execute(
//         //     "CREATE TABLE IF NOT EXISTS payloads (
//         //         ID INTEGER PRIMARY KEY,
//         //         PAYLOAD TEXT NOT NULL,
//         //         TIMESTAMP DATETIME DEFAULT CURRENT_TIMESTAMP
//         //     )",
//         //     [],
//         // );

//         // // Add the payload to the database
//         // conn.execute(
//         //     "INSERT INTO payloads (PAYLOAD) VALUES (?1)",
//         //     [payload.to_string()],
//         // );
//     }
// }

// Send a message to Discord
async fn send_data_to_webhook(webhook_url: &str, data: &str) -> Result<Response, Box<dyn Error>> {
    let client = Client::new();

    let data = json!({
        "content": data
    });

    // send JSON to the webhook URL
    let res = client.post(webhook_url).json(&data).send().await?;

    Ok(res)
}

#[tokio::test]
async fn test_mock_webhook() {
    use mockito;

    let mut server = mockito::Server::new();

    // Create a mock
    server
        .mock("POST", "/webhook")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status": "OK"}"#)
        .create();

    let payload = json!({
    "content": "Test"
    });

    let url = format!("{}{}", server.url(), "/webhook");
    let data = payload.get("content").unwrap().to_string();

    let res = send_data_to_webhook(&url, &data).await.unwrap();

    assert_eq!(res.status(), 200);
}
