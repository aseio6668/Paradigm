/// WebSocket module for real-time updates
pub mod handler;
pub mod manager;
pub mod subscription;

pub use handler::WebSocketHandler;
pub use manager::WebSocketManager;
pub use subscription::{Subscription, SubscriptionManager};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use std::sync::Arc;
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    models::{WebSocketMessage, SubscriptionRequest},
    services::ApiServices,
};

/// WebSocket connection handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(services): State<Arc<ApiServices>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, services))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, services: Arc<ApiServices>) {
    let connection_id = Uuid::new_v4();
    tracing::info!("New WebSocket connection: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WebSocketMessage>();

    // Spawn task to handle outgoing messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json_msg = match serde_json::to_string(&msg) {
                Ok(json) => json,
                Err(e) => {
                    tracing::error!("Failed to serialize WebSocket message: {}", e);
                    continue;
                }
            };

            if sender.send(Message::Text(json_msg)).await.is_err() {
                tracing::info!("WebSocket connection closed");
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_text_message(&text, &tx, &services).await {
                    tracing::error!("Error handling WebSocket message: {}", e);
                }
            }
            Ok(Message::Binary(_)) => {
                tracing::warn!("Binary messages not supported");
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket connection closed by client");
                break;
            }
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() {
                    break;
                }
            }
            Ok(Message::Pong(_)) => {
                // Handle pong if needed
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Clean up
    send_task.abort();
    tracing::info!("WebSocket connection {} closed", connection_id);
}

/// Handle text message from client
async fn handle_text_message(
    text: &str,
    tx: &mpsc::UnboundedSender<WebSocketMessage>,
    services: &Arc<ApiServices>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let request: SubscriptionRequest = serde_json::from_str(text)?;
    
    // Handle subscription
    services.websocket_service()
        .handle_subscription(request, tx.clone())
        .await?;

    Ok(())
}

/// Broadcast message to all connected clients
pub async fn broadcast_message(
    services: &Arc<ApiServices>,
    message: WebSocketMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    services.websocket_service()
        .broadcast(message)
        .await?;

    Ok(())
}