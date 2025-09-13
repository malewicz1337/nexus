use chrono;

pub mod webhook {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct GitHubEvent {
        pub event_type: String,
        pub repository: String,
        pub sender: String,
        pub timestamp: chrono::DateTime<chrono::Utc>,
    }

    pub fn process_event(event: GitHubEvent) -> Result<String, Box<dyn std::error::Error>> {
        // you can add more complex processing logic here
        Ok(format!(
            "Processed {} event from {}",
            event.event_type, event.repository
        ))
    }
}
