# Nexus | GitHub Webhook Service

A lightweight, secure webhook receiver service built in Rust that integrates with GitHub repositories to automate workflows and respond to repository events.

## Features

-  Secure webhook signature verification
-  Support for multiple GitHub event types (push, pull_request, issues, ping)
-  JSON logging and structured responses
-  Health check endpoint
-  CORS support for web integrations
-  Configurable via CLI arguments or environment variables
-  Production-ready with proper error handling

## Quick Start

### 1. Build and Run

```bash
git clone <your-repo>
cd github-webhook-service
cargo build --release

# Run with webhook secret
cargo run -- --port 6666 --secret your-webhook-secret
```

### 2. Set Up GitHub Webhook

1. Go to your GitHub repository
2. Navigate to Settings → Webhooks
3. Click "Add webhook"
4. Configure:
   - **Payload URL**: `https://your-domain.com/webhook`
   - **Content type**: `application/json`
   - **Secret**: Use the same secret as your service
   - **Events**: Select events you want to handle

### 3. Test the Integration

```bash
# Health check
curl http://localhost:6666/health

# Service info
curl http://localhost:6666/

# GitHub will automatically send a ping event to test the webhook
```

## Configuration

### Command Line Options

```bash
github-webhook-service --help

Options:
  -p, --port <PORT>        Port to run the server on [default: 6666]
  -s, --secret <SECRET>    GitHub webhook secret [env: GITHUB_WEBHOOK_SECRET]
  -h, --help               Print help
  -V, --version            Print version
```

### Environment Variables

- `GITHUB_WEBHOOK_SECRET`: Your GitHub webhook secret

## Supported Events

The service currently handles these GitHub events:

- **push**: Repository push events
- **pull_request**: PR opened, closed, synchronized, etc.
- **issues**: Issue opened, closed, edited, etc.
- **ping**: GitHub webhook test event

## Extending the Service

### Adding Custom Event Handlers

```rust
// In handle_webhook function, add new event types:
match event_type {
    "release" => {
        info!("Processing release event");
    }
    "workflow_run" => {
        info!("Processing workflow run event");
    }
}
```

### Adding External API Calls

The service includes a `reqwest::Client` in the app state for making HTTP requests:

```rust
async fn handle_custom_event(state: &AppState, payload: &WebhookPayload) -> Result<(), StatusCode> {
    let response = state.http_client
        .post("https://api.example.com/notify")
        .json(&payload)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(())
}
```

## API Endpoints

### `POST /webhook`
Receives GitHub webhook events. Requires proper signature if secret is configured.

### `GET /health`
Health check endpoint. Returns service status and version.

### `GET /`
Service information endpoint. Lists supported events and endpoints.

## Example Use Cases

1. **Automated Testing**: Trigger test suites on PR events
2. **Deployment Pipeline**: Deploy on push to main branch
3. **Issue Management**: Auto-assign labels or notify teams
4. **Code Quality**: Run linters and security scans
5. **Notifications**: Send Slack/Discord messages on events

## Troubleshooting

### Common Issues

1. **Webhook delivery failed**: Check URL accessibility and HTTPS certificate
2. **Signature verification failed**: Ensure secret matches between GitHub and service
3. **Events not processing**: Check GitHub event selection in webhook settings
4. **Service unreachable**: Verify firewall settings and port configuration

### Debug Mode

Run with debug logging:

```bash
RUST_LOG=debug cargo run -- --port 6666 --secret your-secret
```

## Contributing

1. Fork the repository
2. Create a feature branch
4. Submit a pull request

## License

MIT License - see LICENSE file for details.
