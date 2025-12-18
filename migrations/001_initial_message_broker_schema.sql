-- Initial message broker schema for Constellation A2A protocol
-- Migration: 001_initial_message_broker_schema.sql

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Message priority levels
CREATE TYPE message_priority AS ENUM (
    'low',
    'normal', 
    'high',
    'critical'
);

-- Message delivery status
CREATE TYPE delivery_status AS ENUM (
    'pending',
    'queued',
    'delivering',
    'delivered',
    'failed',
    'dead_letter'
);

-- Agent session status
CREATE TYPE session_status AS ENUM (
    'connected',
    'disconnected',
    'idle',
    'busy'
);

-- Messages table - core message storage
CREATE TABLE messages (
    -- Primary identifier
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Message metadata
    message_id VARCHAR(255) NOT NULL UNIQUE, -- External message ID from A2A protocol
    correlation_id VARCHAR(255), -- For request-response correlation
    conversation_id VARCHAR(255), -- For multi-message conversations
    
    -- Sender and recipient
    sender_id VARCHAR(255) NOT NULL,
    recipient_id VARCHAR(255) NOT NULL,
    
    -- Message content
    message_type VARCHAR(100) NOT NULL, -- e.g., 'request', 'response', 'notification'
    protocol_version VARCHAR(20) NOT NULL DEFAULT '1.0',
    content_type VARCHAR(100) NOT NULL DEFAULT 'application/json',
    payload TEXT NOT NULL, -- JSON message body
    metadata JSONB, -- Additional metadata
    
    -- Delivery properties
    priority message_priority NOT NULL DEFAULT 'normal',
    delivery_guarantee VARCHAR(50) NOT NULL DEFAULT 'at-least-once',
    ttl_seconds INTEGER, -- Time-to-live in seconds
    max_retries INTEGER NOT NULL DEFAULT 3,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    scheduled_for TIMESTAMP WITH TIME ZONE, -- For delayed messages
    expires_at TIMESTAMP WITH TIME ZONE, -- Calculated from TTL
    
    -- Indexes
    INDEX idx_messages_sender (sender_id),
    INDEX idx_messages_recipient (recipient_id),
    INDEX idx_messages_correlation (correlation_id),
    INDEX idx_messages_conversation (conversation_id),
    INDEX idx_messages_created (created_at),
    INDEX idx_messages_scheduled (scheduled_for),
    INDEX idx_messages_expires (expires_at)
);

-- Queues table - for priority-based message queuing
CREATE TABLE queues (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    
    -- Queue properties
    queue_name VARCHAR(100) NOT NULL DEFAULT 'default',
    priority message_priority NOT NULL DEFAULT 'normal',
    sequence_number BIGSERIAL, -- For ordering within same priority
    
    -- Queue state
    enqueued_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    dequeued_at TIMESTAMP WITH TIME ZONE,
    
    -- Indexes for efficient queue operations
    INDEX idx_queues_message (message_id),
    INDEX idx_queues_priority (queue_name, priority, sequence_number),
    INDEX idx_queues_pending (queue_name, priority, sequence_number) 
        WHERE dequeued_at IS NULL,
    UNIQUE (message_id, queue_name)
);

-- Delivery status tracking
CREATE TABLE delivery_status (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    
    -- Delivery state
    status delivery_status NOT NULL DEFAULT 'pending',
    current_retry INTEGER NOT NULL DEFAULT 0,
    last_delivery_attempt TIMESTAMP WITH TIME ZONE,
    next_retry_at TIMESTAMP WITH TIME ZONE,
    
    -- Delivery results
    delivered_at TIMESTAMP WITH TIME ZONE,
    failed_at TIMESTAMP WITH TIME ZONE,
    failure_reason TEXT,
    
    -- Recipient acknowledgment
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    acknowledgment_payload JSONB,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_delivery_message (message_id),
    INDEX idx_delivery_status (status),
    INDEX idx_delivery_next_retry (next_retry_at),
    INDEX idx_delivery_updated (updated_at),
    UNIQUE (message_id) -- One delivery status per message
);

-- Agent sessions for connection management
CREATE TABLE agent_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Agent identification
    agent_id VARCHAR(255) NOT NULL,
    session_token VARCHAR(512) NOT NULL UNIQUE, -- JWT or session token
    
    -- Connection information
    connection_id VARCHAR(255), -- WebSocket or connection ID
    protocol_binding VARCHAR(50) NOT NULL, -- 'http+json', 'websocket', etc.
    client_ip INET,
    user_agent TEXT,
    
    -- Session state
    status session_status NOT NULL DEFAULT 'connected',
    capabilities JSONB, -- Agent capabilities from A2A agent card
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP + INTERVAL '1 hour',
    
    -- Indexes
    INDEX idx_sessions_agent (agent_id),
    INDEX idx_sessions_token (session_token),
    INDEX idx_sessions_status (status),
    INDEX idx_sessions_expires (expires_at),
    INDEX idx_sessions_activity (last_activity_at)
);

-- Dead letter queue for failed messages
CREATE TABLE dead_letter_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    
    -- Failure details
    original_queue VARCHAR(100) NOT NULL,
    failure_reason TEXT NOT NULL,
    failure_details JSONB,
    failed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Recovery information
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_retry_attempt TIMESTAMP WITH TIME ZONE,
    
    -- Indexes
    INDEX idx_dlq_message (message_id),
    INDEX idx_dlq_failed (failed_at)
);

-- Message routing rules (for future advanced routing)
CREATE TABLE routing_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Rule criteria
    rule_name VARCHAR(255) NOT NULL UNIQUE,
    match_pattern JSONB NOT NULL, -- JSON pattern to match messages
    priority INTEGER NOT NULL DEFAULT 0,
    
    -- Routing action
    target_queue VARCHAR(100) NOT NULL,
    transform_script TEXT, -- Optional transformation
    
    -- Rule state
    enabled BOOLEAN NOT NULL DEFAULT true,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_routing_enabled (enabled, priority)
);

-- Create function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for tables with updated_at
CREATE TRIGGER update_delivery_status_updated_at 
    BEFORE UPDATE ON delivery_status 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_routing_rules_updated_at 
    BEFORE UPDATE ON routing_rules 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create function to handle message expiration
CREATE OR REPLACE FUNCTION expire_old_messages()
RETURNS void AS $$
BEGIN
    -- Move expired messages to dead letter queue
    INSERT INTO dead_letter_queue (
        message_id, 
        original_queue, 
        failure_reason, 
        failure_details,
        failed_at
    )
    SELECT 
        m.id,
        'default',
        'Message expired',
        jsonb_build_object(
            'expired_at', CURRENT_TIMESTAMP,
            'original_ttl', m.ttl_seconds
        ),
        CURRENT_TIMESTAMP
    FROM messages m
    WHERE m.expires_at IS NOT NULL 
        AND m.expires_at < CURRENT_TIMESTAMP
        AND NOT EXISTS (
            SELECT 1 FROM dead_letter_queue dlq 
            WHERE dlq.message_id = m.id
        );
    
    -- Update delivery status for expired messages
    UPDATE delivery_status ds
    SET 
        status = 'dead_letter',
        updated_at = CURRENT_TIMESTAMP,
        failure_reason = 'Message expired'
    FROM messages m
    WHERE ds.message_id = m.id
        AND m.expires_at IS NOT NULL 
        AND m.expires_at < CURRENT_TIMESTAMP
        AND ds.status NOT IN ('delivered', 'dead_letter');
END;
$$ language 'plpgsql';

-- Create function to get next message from queue
CREATE OR REPLACE FUNCTION get_next_queue_message(
    p_queue_name VARCHAR DEFAULT 'default',
    p_limit INTEGER DEFAULT 1
)
RETURNS TABLE (
    message_id UUID,
    queue_id UUID,
    priority message_priority,
    sequence_number BIGINT,
    payload TEXT,
    content_type VARCHAR,
    sender_id VARCHAR,
    recipient_id VARCHAR
) AS $$
BEGIN
    RETURN QUERY
    WITH next_messages AS (
        SELECT 
            q.id as queue_id,
            q.message_id,
            q.priority,
            q.sequence_number,
            m.payload,
            m.content_type,
            m.sender_id,
            m.recipient_id
        FROM queues q
        JOIN messages m ON q.message_id = m.id
        WHERE q.queue_name = p_queue_name
            AND q.dequeued_at IS NULL
            AND (m.expires_at IS NULL OR m.expires_at > CURRENT_TIMESTAMP)
        ORDER BY 
            CASE q.priority
                WHEN 'critical' THEN 1
                WHEN 'high' THEN 2
                WHEN 'normal' THEN 3
                WHEN 'low' THEN 4
            END,
            q.sequence_number
        LIMIT p_limit
        FOR UPDATE SKIP LOCKED
    )
    UPDATE queues q
    SET dequeued_at = CURRENT_TIMESTAMP
    FROM next_messages nm
    WHERE q.id = nm.queue_id
    RETURNING 
        nm.message_id,
        nm.queue_id,
        nm.priority,
        nm.sequence_number,
        nm.payload,
        nm.content_type,
        nm.sender_id,
        nm.recipient_id;
END;
$$ language 'plpgsql';

-- Create indexes for performance
CREATE INDEX idx_messages_priority_ttl ON messages(priority, expires_at) 
    WHERE expires_at IS NOT NULL;

CREATE INDEX idx_queues_performance ON queues(queue_name, priority, sequence_number, dequeued_at);

CREATE INDEX idx_delivery_retry ON delivery_status(status, next_retry_at) 
    WHERE status IN ('pending', 'failed');

-- Create view for message dashboard
CREATE VIEW message_dashboard AS
SELECT 
    m.id,
    m.message_id,
    m.sender_id,
    m.recipient_id,
    m.message_type,
    m.priority,
    m.created_at,
    ds.status as delivery_status,
    ds.current_retry,
    ds.last_delivery_attempt,
    q.queue_name,
    q.enqueued_at,
    CASE 
        WHEN m.expires_at < CURRENT_TIMESTAMP THEN 'expired'
        WHEN ds.status = 'delivered' THEN 'delivered'
        WHEN ds.status = 'failed' AND ds.current_retry >= m.max_retries THEN 'dead_letter'
        WHEN ds.status = 'failed' THEN 'retrying'
        ELSE 'active'
    END as message_state
FROM messages m
LEFT JOIN delivery_status ds ON m.id = ds.message_id
LEFT JOIN queues q ON m.id = q.message_id AND q.dequeued_at IS NULL;

-- Create view for agent activity
CREATE VIEW agent_activity AS
SELECT 
    agent_id,
    COUNT(DISTINCT session_token) as active_sessions,
    MIN(created_at) as first_session,
    MAX(last_activity_at) as last_activity,
    COUNT(DISTINCT CASE WHEN status = 'connected' THEN session_token END) as connected_sessions,
    COUNT(DISTINCT CASE WHEN status = 'busy' THEN session_token END) as busy_sessions
FROM agent_sessions
WHERE expires_at > CURRENT_TIMESTAMP
GROUP BY agent_id;

-- Comments for documentation
COMMENT ON TABLE messages IS 'Stores all A2A protocol messages with metadata and delivery properties';
COMMENT ON TABLE queues IS 'Manages priority-based message queuing with ordering guarantees';
COMMENT ON TABLE delivery_status IS 'Tracks message delivery state with retry logic and acknowledgments';
COMMENT ON TABLE agent_sessions IS 'Manages agent connection state and session information';
COMMENT ON TABLE dead_letter_queue IS 'Stores messages that failed delivery after max retries';
COMMENT ON TABLE routing_rules IS 'Defines message routing rules for advanced routing scenarios';

COMMENT ON COLUMN messages.message_id IS 'External message ID from A2A protocol (unique)';
COMMENT ON COLUMN messages.correlation_id IS 'For correlating request-response messages';
COMMENT ON COLUMN messages.conversation_id IS 'For grouping messages in multi-message conversations';
COMMENT ON COLUMN messages.ttl_seconds IS 'Time-to-live in seconds (NULL means no expiration)';
COMMENT ON COLUMN messages.expires_at IS 'Calculated expiration timestamp (created_at + ttl_seconds)';

COMMENT ON COLUMN queues.sequence_number IS 'Auto-incrementing sequence for ordering within same priority';
COMMENT ON COLUMN queues.dequeued_at IS 'Timestamp when message was dequeued (NULL means still in queue)';

COMMENT ON COLUMN delivery_status.next_retry_at IS 'When to retry failed delivery (exponential backoff)';
COMMENT ON COLUMN delivery_status.acknowledgment_payload IS 'Optional acknowledgment data from recipient';

COMMENT ON COLUMN agent_sessions.session_token IS 'JWT or session token for authentication';
COMMENT ON COLUMN agent_sessions.capabilities IS 'JSON representation of agent capabilities from A2A agent card';