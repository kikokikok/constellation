//! Example HTTP server for Constellation A2A Message Broker
//!
//! This example demonstrates a complete HTTP server implementing the A2A API
//! using Axum web framework.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, delete},
    Json, Router,
};
use constellation_core::message_broker::{LlmMessageBroker, LlmMessageBrokerBuilder};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

// Application state
#[derive(Clone)]
struct AppState {
    broker: Arc<LlmMessageBroker>,
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

// Agent model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Agent {
    id: String,
    name: String,
    status: String,
    registered_at: String,
    last_seen: Option<String>,
    capabilities: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
}

// Message model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    id: String,
    sender: String,
    recipient: String,
    #[serde(rename = "type")]
    message_type: String,
    payload: serde_json::Value,
    timestamp: String,
    priority: Option<u8>,
    correlation_id: Option<String>,
    a2a_version: Option<String>,
    headers: Option<HashMap<String, String>>,
    ttl: Option<u32>,
}

// Health response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
    components: HashMap<String, String>,
}

// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code.as_str() {
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "CONFLICT" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        (status, Json(self)).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting Constellation HTTP Server");

    // Create message broker
    let broker = LlmMessageBrokerBuilder::new()
        .max_queue_size(1000)
        .message_ttl(3600)
        .max_retries(3)
        .retry_delay(30)
        .session_timeout(300)
        .build();

    // Create application state
    let state = AppState {
        broker: Arc::new(broker),
        agents: Arc::new(RwLock::new(HashMap::new())),
    };

    // Build router
    let app = Router::new()
        // System endpoints
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        
        // Agent endpoints
        .route("/agents", get(list_agents).post(register_agent))
        .route("/agents/:agent_id", get(get_agent).delete(deregister_agent))
        .route("/agents/:agent_id/status", get(get_agent_status))
        
        // Message endpoints
        .route("/agents/:agent_id/messages", get(get_agent_messages).post(send_message))
        .route("/agents/:agent_id/messages/:message_id", get(get_message).delete(delete_message))
        
        // Broadcast endpoint
        .route("/broadcast", post(broadcast_message))
        
        // Authentication endpoint
        .route("/auth/token", post(generate_token))
        
        // Add state
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("📡 Server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime: 0, // Would track actual uptime in production
        components: HashMap::from([
            ("message_broker".to_string(), "healthy".to_string()),
            ("database".to_string(), "healthy".to_string()),
            ("authentication".to_string(), "healthy".to_string()),
        ]),
    })
}

// Metrics endpoint
async fn metrics() -> impl IntoResponse {
    // In production, this would return Prometheus metrics
    "# HELP constellation_messages_total Total number of messages processed\n\
     # TYPE constellation_messages_total counter\n\
     constellation_messages_total{type=\"sent\"} 0\n\
     constellation_messages_total{type=\"received\"} 0\n"
}

// List agents endpoint
async fn list_agents(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let agents = state.agents.read().await;
    let mut filtered_agents: Vec<Agent> = agents.values().cloned().collect();
    
    // Apply filters
    if let Some(status) = params.get("status") {
        filtered_agents.retain(|agent| agent.status == *status);
    }
    
    // Apply limit
    let limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(100)
        .min(1000);
    
    filtered_agents.truncate(limit);
    
    Json(filtered_agents)
}

// Register agent endpoint
async fn register_agent(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Agent>, ErrorResponse> {
    let agent_id = Uuid::new_v4().to_string();
    
    let agent = Agent {
        id: agent_id.clone(),
        name: payload.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ErrorResponse {
                code: "BAD_REQUEST".to_string(),
                message: "Missing required field: name".to_string(),
                details: None,
                request_id: None,
            })?
            .to_string(),
        status: "offline".to_string(),
        registered_at: chrono::Utc::now().to_rfc3339(),
        last_seen: None,
        capabilities: payload.get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        metadata: payload.get("metadata")
            .and_then(|v| v.as_object())
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
    };
    
    let mut agents = state.agents.write().await;
    if agents.contains_key(&agent_id) {
        return Err(ErrorResponse {
            code: "CONFLICT".to_string(),
            message: format!("Agent with ID {} already exists", agent_id),
            details: None,
            request_id: None,
        });
    }
    
    agents.insert(agent_id.clone(), agent.clone());
    
    info!("✅ Registered agent: {} ({})", agent.name, agent.id);
    Ok(Json(agent))
}

// Get agent endpoint
async fn get_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<Agent>, ErrorResponse> {
    let agents = state.agents.read().await;
    
    agents.get(&agent_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: format!("Agent with ID {} not found", agent_id),
            details: None,
            request_id: None,
        })
}

// Deregister agent endpoint
async fn deregister_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<StatusCode, ErrorResponse> {
    let mut agents = state.agents.write().await;
    
    if agents.remove(&agent_id).is_some() {
        info!("🗑️  Deregistered agent: {}", agent_id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: format!("Agent with ID {} not found", agent_id),
            details: None,
            request_id: None,
        })
    }
}

// Get agent status endpoint
async fn get_agent_status(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let agents = state.agents.read().await;
    
    if !agents.contains_key(&agent_id) {
        return Err(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: format!("Agent with ID {} not found", agent_id),
            details: None,
            request_id: None,
        });
    }
    
    // In production, this would get real queue stats from the broker
    let status = serde_json::json!({
        "agentId": agent_id,
        "status": "offline",
        "lastActivity": chrono::Utc::now().to_rfc3339(),
        "queueStats": {
            "total": 0,
            "byPriority": {
                "critical": 0,
                "high": 0,
                "normal": 0,
                "low": 0
            },
            "oldestMessage": null
        },
        "sessionId": null
    });
    
    Ok(Json(status))
}

// Get agent messages endpoint
async fn get_agent_messages(
    State(_state): State<AppState>,
    Path(agent_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Message>>, ErrorResponse> {
    // In production, this would get messages from the broker
    // For this example, return empty list
    
    let _limit = params.get("limit")
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(100);
    
    let _since = params.get("since");
    let _priority = params.get("priority");
    
    // Check if agent exists (simulated)
    if agent_id.is_empty() {
        return Err(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: "Agent not found".to_string(),
            details: None,
            request_id: None,
        });
    }
    
    Ok(Json(vec![]))
}

// Send message endpoint
async fn send_message(
    State(state): State<AppState>,
    Path(agent_id): Path<String>,
    Json(mut message): Json<Message>,
) -> Result<(StatusCode, Json<serde_json::Value>), ErrorResponse> {
    // Validate recipient exists
    let agents = state.agents.read().await;
    if !agents.contains_key(&agent_id) {
        return Err(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: format!("Recipient agent {} not found", agent_id),
            details: None,
            request_id: None,
        });
    }
    
    // Set message ID if not provided
    if message.id.is_empty() {
        message.id = Uuid::new_v4().to_string();
    }
    
    // Set timestamp if not provided
    if message.timestamp.is_empty() {
        message.timestamp = chrono::Utc::now().to_rfc3339();
    }
    
    // Set default priority
    if message.priority.is_none() {
        message.priority = Some(5);
    }
    
    // Set default A2A version
    if message.a2a_version.is_none() {
        message.a2a_version = Some("1.0".to_string());
    }
    
    // In production, this would send the message through the broker
    info!("📨 Sent message from {} to {}: {}", 
          message.sender, message.recipient, message.id);
    
    let ack = serde_json::json!({
        "messageId": message.id,
        "status": "queued",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "queuePosition": 1,
        "estimatedDelivery": chrono::Utc::now().to_rfc3339()
    });
    
    Ok((StatusCode::ACCEPTED, Json(ack)))
}

// Get message endpoint
async fn get_message(
    State(_state): State<AppState>,
    Path((agent_id, message_id)): Path<(String, String)>,
) -> Result<Json<Message>, ErrorResponse> {
    // In production, this would get the message from the broker
    // For this example, return a simulated message
    
    if agent_id.is_empty() || message_id.is_empty() {
        return Err(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
            details: None,
            request_id: None,
        });
    }
    
    let message = Message {
        id: message_id,
        sender: "system".to_string(),
        recipient: agent_id,
        message_type: "command".to_string(),
        payload: serde_json::json!({"action": "test"}),
        timestamp: chrono::Utc::now().to_rfc3339(),
        priority: Some(5),
        correlation_id: None,
        a2a_version: Some("1.0".to_string()),
        headers: None,
        ttl: Some(3600),
    };
    
    Ok(Json(message))
}

// Delete message endpoint
async fn delete_message(
    State(_state): State<AppState>,
    Path((agent_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode, ErrorResponse> {
    // In production, this would delete the message from the broker
    
    if agent_id.is_empty() || message_id.is_empty() {
        return Err(ErrorResponse {
            code: "NOT_FOUND".to_string(),
            message: "Message not found".to_string(),
            details: None,
            request_id: None,
        });
    }
    
    info!("🗑️  Deleted message {} for agent {}", message_id, agent_id);
    Ok(StatusCode::NO_CONTENT)
}

// Broadcast message endpoint
async fn broadcast_message(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ErrorResponse> {
    let sender = payload.get("sender")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorResponse {
            code: "BAD_REQUEST".to_string(),
            message: "Missing required field: sender".to_string(),
            details: None,
            request_id: None,
        })?;
    
    let agents = state.agents.read().await;
    let recipient_count = agents.len();
    
    let broadcast_id = Uuid::new_v4().to_string();
    
    info!("📢 Broadcast from {} to {} agents: {}", 
          sender, recipient_count, broadcast_id);
    
    let ack = serde_json::json!({
        "broadcastId": broadcast_id,
        "recipients": recipient_count,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    Ok((StatusCode::ACCEPTED, Json(ack)))
}

// Generate token endpoint
async fn generate_token(
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let agent_id = payload.get("agentId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ErrorResponse {
            code: "BAD_REQUEST".to_string(),
            message: "Missing required field: agentId".to_string(),
            details: None,
            request_id: None,
        })?;
    
    // In production, this would validate the signature
    // For this example, generate a simple token
    
    let token = format!("token-{}-{}", agent_id, Uuid::new_v4());
    
    let response = serde_json::json!({
        "token": token,
        "expiresAt": chrono::Utc::now() + chrono::Duration::hours(24),
        "agentId": agent_id
    });
    
    Ok(Json(response))
}