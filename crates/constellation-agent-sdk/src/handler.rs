//! Message and request handlers for agents

use async_trait::async_trait;
use constellation_core::models::communication::{RequestMessage, ResponseMessage};
use constellation_core::models::message_broker::Message;

/// Trait for handling incoming messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message
    async fn handle_message(&self, message: Message) -> Option<Message>;
}

/// Trait for handling incoming requests
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle an incoming request and produce a response
    async fn handle_request(&self, request: RequestMessage) -> ResponseMessage;
}

/// Default message handler that logs messages
#[allow(dead_code)]
pub struct DefaultMessageHandler;

#[async_trait]
impl MessageHandler for DefaultMessageHandler {
    async fn handle_message(&self, message: Message) -> Option<Message> {
        tracing::debug!("Received message: {:?}", message);
        None // Don't send a response
    }
}

/// Default request handler that echoes requests
#[allow(dead_code)]
pub struct DefaultRequestHandler;

#[async_trait]
impl RequestHandler for DefaultRequestHandler {
    async fn handle_request(&self, request: RequestMessage) -> ResponseMessage {
        tracing::info!("Handling request: {}", request.request_id);

        // Clone fields to avoid partial move
        let correlation_id = request.correlation_id.clone();
        let recipient = request.recipient.clone();
        let sender_id = request.sender_id.clone();
        let payload = request.payload.clone();

        // Echo the request payload
        ResponseMessage::success(
            correlation_id,
            recipient,
            sender_id,
            format!("Echo: {}", payload),
            Some(request),
        )
    }
}

/// Composite handler that delegates to multiple handlers
#[allow(dead_code)]
pub struct CompositeMessageHandler {
    handlers: Vec<Box<dyn MessageHandler>>,
}

#[allow(dead_code)]
impl CompositeMessageHandler {
    /// Create a new composite handler
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a handler
    pub fn add_handler<H: MessageHandler + 'static>(mut self, handler: H) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }
}

#[async_trait]
impl MessageHandler for CompositeMessageHandler {
    async fn handle_message(&self, message: Message) -> Option<Message> {
        for handler in &self.handlers {
            if let Some(response) = handler.handle_message(message.clone()).await {
                return Some(response);
            }
        }
        None
    }
}

/// Composite request handler that delegates to multiple handlers
#[allow(dead_code)]
pub struct CompositeRequestHandler {
    handlers: Vec<Box<dyn RequestHandler>>,
}

#[allow(dead_code)]
impl CompositeRequestHandler {
    /// Create a new composite request handler
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a handler
    pub fn add_handler<H: RequestHandler + 'static>(mut self, handler: H) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }
}

#[async_trait]
impl RequestHandler for CompositeRequestHandler {
    async fn handle_request(&self, request: RequestMessage) -> ResponseMessage {
        for handler in &self.handlers {
            let response = handler.handle_request(request.clone()).await;
            if response.status == constellation_core::models::communication::ResponseStatus::Success
            {
                return response;
            }
        }

        // Default error response if no handler succeeds
        // Clone fields to avoid partial move
        let correlation_id = request.correlation_id.clone();
        let recipient = request.recipient.clone();
        let sender_id = request.sender_id.clone();

        ResponseMessage::error(
            correlation_id,
            recipient,
            sender_id,
            "No handler could process the request".to_string(),
            Some(request),
        )
    }
}
