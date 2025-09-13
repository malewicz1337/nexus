use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
};
use clap::Parser;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{collections::HashMap, env, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "6666")]
    port: u16,

    #[arg(short, long, env = "GITHUB_WEBHOOK_SECRET")]
    secret: Option<String>,
}

#[derive(Clone)]
struct AppState {
    webhook_secret: Option<String>,
    http_client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    action: Option<String>,
    repository: Option<Repository>,
    sender: Option<User>,
    pull_request: Option<PullRequest>,
    issue: Option<Issue>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct User {
    login: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    number: u64,
    title: String,
    html_url: String,
    state: String,
    user: User,
}

#[derive(Debug, Deserialize)]
struct Issue {
    number: u64,
    title: String,
    html_url: String,
    state: String,
    user: User,
}

#[derive(Serialize)]
struct WebhookResponse {
    message: String,
    processed: bool,
}

#[derive(Deserialize)]
struct HealthQuery {
    token: Option<String>,
}

fn verify_signature(secret: &str, payload: &[u8], signature: &str) -> bool {
    if !signature.starts_with("sha256=") {
        return false;
    }

    let signature = &signature[7..];

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(_) => return false,
    };

    mac.update(payload);

    match hex::decode(signature) {
        Ok(expected) => mac.verify_slice(&expected).is_ok(),
        Err(_) => false,
    }
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<WebhookResponse>, StatusCode> {
    if let Some(secret) = &state.webhook_secret {
        if let Some(signature) = headers.get("x-hub-signature-256") {
            let signature_str = signature.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            if !verify_signature(secret, &body, signature_str) {
                warn!("Invalid webhook signature");
                return Err(StatusCode::UNAUTHORIZED);
            }
        } else {
            warn!("Missing webhook signature");
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let payload: WebhookPayload = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse webhook payload: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let event_type = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    info!("Received {} event", event_type);

    match event_type {
        "push" => {
            info!(
                "Processing push event for repository: {:?}",
                payload.repository.as_ref().map(|r| &r.full_name)
            );
            // your push event logic here
        }
        "pull_request" => {
            if let Some(pr) = &payload.pull_request {
                info!(
                    "Processing pull request #{}: {} ({})",
                    pr.number, pr.title, pr.state
                );
                // your PR event logic here
                handle_pull_request_event(&state, &payload).await?;
            }
        }
        "issues" => {
            if let Some(issue) = &payload.issue {
                info!(
                    "Processing issue #{}: {} ({})",
                    issue.number, issue.title, issue.state
                );
                // your issue event logic here
            }
        }
        "ping" => {
            info!("Received ping event - webhook is configured correctly!");
        }
        _ => {
            info!("Unhandled event type: {}", event_type);
        }
    }

    Ok(Json(WebhookResponse {
        message: format!("Successfully processed {} event", event_type),
        processed: true,
    }))
}

async fn handle_pull_request_event(
    _state: &AppState,
    payload: &WebhookPayload,
) -> Result<(), StatusCode> {
    if let (Some(action), Some(pr), Some(repo)) =
        (&payload.action, &payload.pull_request, &payload.repository)
    {
        match action.as_str() {
            "opened" => {
                info!("New PR opened: {} in {}", pr.title, repo.full_name);
            }
            "closed" => {
                info!("PR closed: {} in {}", pr.title, repo.full_name);
            }
            "synchronize" => {
                info!("PR updated: {} in {}", pr.title, repo.full_name);
            }
            _ => {}
        }
    }
    Ok(())
}

async fn health_check(Query(params): Query<HealthQuery>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "github-webhook-service",
        "version": env!("CARGO_PKG_VERSION"),
        "authenticated": params.token.is_some()
    }))
}

async fn webhook_info() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "service": "GitHub Webhook Service",
        "endpoints": {
            "webhook": "/webhook",
            "health": "/health",
            "info": "/"
        },
        "supported_events": [
            "push",
            "pull_request",
            "issues",
            "ping"
        ]
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let state = Arc::new(AppState {
        webhook_secret: args.secret.clone(),
        http_client: reqwest::Client::new(),
    });

    let app = Router::new()
        .route("/", get(webhook_info))
        .route("/health", get(health_check))
        .route("/webhook", post(handle_webhook))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    info!("GitHub Webhook Service starting on {}", addr);
    if args.secret.is_some() {
        info!("Webhook signature verification enabled");
    } else {
        warn!("No webhook secret configured - signatures will not be verified");
    }

    axum::serve(listener, app).await.unwrap();
}
