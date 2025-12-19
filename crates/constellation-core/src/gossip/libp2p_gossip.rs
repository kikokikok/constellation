//! libp2p-based gossip protocol implementation
//!
//! This module integrates libp2p for peer discovery and gossip communication.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    gossipsub::{self, Gossipsub, GossipsubConfig, GossipsubEvent, IdentTopic, MessageAuthenticity},
    identity::Keypair,
    kad::{self, Kademlia, KademliaConfig, KademliaEvent, QueryId, Record, store::MemoryStore},
    mdns::{self, tokio::Behaviour as Mdns, MdnsEvent},
    noise,
    ping::{self, Ping, PingConfig, PingEvent},
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp::tokio::{Transport as TcpTransport, Config as TcpConfig},
    websocket::tokio::WsConfig,
    yamux, Multiaddr, PeerId, Transport,
};
use libp2p::identify::{Identify, IdentifyConfig, IdentifyEvent};

use crate::models::message_broker::A2AMessage;
use crate::communication::MessageBroker;

use super::models::{
    GossipConfig, GossipMessage, Membership, Node, NodeState, SerializationFormat, ServiceInfo,
};

/// libp2p network behaviour combining all protocols
#[derive(NetworkBehaviour)]
struct ConstellationBehaviour {
    /// Identify protocol for peer information
    identify: Identify,
    /// Ping protocol for latency measurement
    ping: Ping,
    /// Kademlia DHT for peer discovery
    kademlia: Kademlia<kad::store::MemoryStore>,
    /// mDNS for local peer discovery
    mdns: Mdns,
    /// GossipSub for message dissemination
    gossipsub: Gossipsub,
}

/// libp2p-based gossip protocol implementation
pub struct Libp2pGossipProtocol {
    /// Protocol configuration
    config: GossipConfig,
    /// Local node information
    local_node: Arc<RwLock<Node>>,
    /// Membership information
    membership: Arc<RwLock<Membership>>,
    /// Known services
    services: Arc<RwLock<HashMap<Uuid, ServiceInfo>>>,
    /// Message broker for communication
    message_broker: Arc<dyn MessageBroker + Send + Sync>,
    /// libp2p swarm
    swarm: Arc<Mutex<Swarm<ConstellationBehaviour>>>,
    /// Protocol statistics
    stats: Arc<Mutex<ProtocolStats>>,
}

/// Protocol statistics
#[derive(Debug, Clone, Default)]
struct ProtocolStats {
    peers_discovered: u64,
    peers_connected: u64,
    messages_sent: u64,
    messages_received: u64,
    gossip_messages_sent: u64,
    gossip_messages_received: u64,
    ping_sent: u64,
    ping_received: u64,
    bytes_sent: u64,
    bytes_received: u64,
}

impl Libp2pGossipProtocol {
    /// Create a new libp2p-based gossip protocol instance
    pub async fn new(
        config: GossipConfig,
        local_node: Node,
        message_broker: Arc<dyn MessageBroker + Send + Sync>,
    ) -> anyhow::Result<Self> {
        // Generate keypair for libp2p
        let keypair = Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        info!("Creating libp2p gossip protocol with peer ID: {}", peer_id);

        // Create transport
        let transport = Self::create_transport(keypair)?;

        // Create behaviours
        let identify = Identify::new(IdentifyConfig::new(
            "/constellation/1.0.0".to_string(),
            keypair.public(),
        ));

        let ping = Ping::new(PingConfig::new().with_keep_alive(true));

        let mut kademlia_config = KademliaConfig::default();
        kademlia_config.set_query_timeout(Duration::from_secs(5 * 60));
        let kademlia_store = kad::store::MemoryStore::new(peer_id);
        let kademlia = Kademlia::with_config(peer_id, kademlia_store, kademlia_config);

        let mdns = Mdns::new(Default::default())?;

        let gossipsub_config = GossipsubConfig::default();
        let gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(keypair),
            gossipsub_config,
        )?;

        // Create behaviour
        let behaviour = ConstellationBehaviour {
            identify,
            ping,
            kademlia,
            mdns,
            gossipsub,
        };

        // Create swarm
        let swarm = Swarm::new(transport, behaviour, peer_id);
        
        let membership = Membership::new(local_node.clone());

        Ok(Self {
            config,
            local_node: Arc::new(RwLock::new(local_node)),
            membership: Arc::new(RwLock::new(membership)),
            services: Arc::new(RwLock::new(HashMap::new())),
            message_broker,
            swarm: Arc::new(Mutex::new(swarm)),
            stats: Arc::new(Mutex::new(ProtocolStats::default())),
        })
    }

    /// Create libp2p transport
    fn create_transport(keypair: Keypair) -> anyhow::Result<Boxed<(PeerId, StreamMuxerBox)>> {
        let noise_config = noise::Config::new(&keypair)?;
        let yamux_config = yamux::Config::default();

        let transport = TcpTransport::new(TcpConfig::default().nodelay(true))
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise_config)
            .multiplex(yamux_config)
            .timeout(Duration::from_secs(20))
            .boxed();

        let websocket_transport = libp2p::websocket::WsConfig::new(transport.clone())
            .or_transport(transport);

        Ok(websocket_transport.boxed())
    }

    /// Start the protocol
    pub async fn start(&self, listen_addr: Multiaddr) -> anyhow::Result<()> {
        info!("Starting libp2p gossip protocol on {}", listen_addr);

        let mut swarm = self.swarm.lock().await;
        
        // Listen on address
        swarm.listen_on(listen_addr)?;

        // Bootstrap Kademlia with known peers
        if let Some(bootstrap_peers) = &self.config.bootstrap_peers {
            for peer_addr in bootstrap_peers {
                if let Ok(addr) = peer_addr.parse() {
                    swarm.behaviour_mut().kademlia.add_address(&PeerId::random(), addr);
                }
            }
        }

        // Start Kademlia bootstrap
        swarm.behaviour_mut().kademlia.bootstrap()?;

        info!("libp2p gossip protocol started");
        Ok(())
    }

    /// Stop the protocol
    pub async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping libp2p gossip protocol");
        Ok(())
    }

    /// Run network event loop (to be called periodically)
    pub async fn run_event_loop(&self) -> anyhow::Result<()> {
        let mut swarm = self.swarm.lock().await;

        // Process swarm events
        while let Some(event) = swarm.select_next_some().await {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {}", address);
                }
                SwarmEvent::Behaviour(ConstellationBehaviourEvent::Identify(event)) => {
                    self.handle_identify_event(event).await;
                }
                SwarmEvent::Behaviour(ConstellationBehaviourEvent::Ping(event)) => {
                    self.handle_ping_event(event).await;
                }
                SwarmEvent::Behaviour(ConstellationBehaviourEvent::Kademlia(event)) => {
                    self.handle_kademlia_event(event).await;
                }
                SwarmEvent::Behaviour(ConstellationBehaviourEvent::Mdns(event)) => {
                    self.handle_mdns_event(event).await;
                }
                SwarmEvent::Behaviour(ConstellationBehaviourEvent::Gossipsub(event)) => {
                    self.handle_gossipsub_event(event).await;
                }
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    info!("Connected to peer: {}", peer_id);
                    self.stats.lock().await.peers_connected += 1;
                    
                    // Add peer to Kademlia
                    swarm.behaviour_mut().kademlia.add_peer(&peer_id);
                }
                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    info!("Disconnected from peer: {}", peer_id);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle Identify events
    async fn handle_identify_event(&self, event: IdentifyEvent) {
        match event {
            IdentifyEvent::Received { peer_id, info, .. } => {
                debug!("Received identify info from {}: {:?}", peer_id, info);
                
                // Update membership with peer information
                let mut membership_guard = self.membership.write().await;
                // TODO: Convert libp2p peer info to Constellation Node
            }
            _ => {}
        }
    }

    /// Handle Ping events
    async fn handle_ping_event(&self, event: PingEvent) {
        match event {
            PingEvent { peer, result, .. } => {
                match result {
                    Ok(rtt) => {
                        debug!("Ping to {}: {:?}", peer, rtt);
                        self.stats.lock().await.ping_received += 1;
                    }
                    Err(e) => {
                        warn!("Ping to {} failed: {:?}", peer, e);
                    }
                }
            }
        }
    }

    /// Handle Kademlia events
    async fn handle_kademlia_event(&self, event: KademliaEvent) {
        match event {
            KademliaEvent::OutboundQueryCompleted { id, result, .. } => {
                match result {
                    kad::QueryResult::Bootstrap(Ok(kad::BootstrapResult { .. })) => {
                        info!("Kademlia bootstrap completed for query {:?}", id);
                    }
                    kad::QueryResult::GetProviders(Ok(kad::GetProvidersResult::FoundProviders { providers, .. })) => {
                        info!("Found {} providers for query {:?}", providers.len(), id);
                        self.stats.lock().await.peers_discovered += providers.len() as u64;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Handle mDNS events
    async fn handle_mdns_event(&self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(list) => {
                for (peer_id, addr) in list {
                    info!("mDNS discovered peer: {} at {}", peer_id, addr);
                    self.stats.lock().await.peers_discovered += 1;
                    
                    // Connect to discovered peer
                    let mut swarm = self.swarm.lock().await;
                    swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                }
            }
            MdnsEvent::Expired(list) => {
                for (peer_id, addr) in list {
                    info!("mDNS expired peer: {} at {}", peer_id, addr);
                }
            }
        }
    }

    /// Handle GossipSub events
    async fn handle_gossipsub_event(&self, event: GossipsubEvent) {
        match event {
            GossipsubEvent::Message {
                propagation_source: peer_id,
                message_id: _,
                message,
            } => {
                debug!("Received gossip message from {}: {} bytes", peer_id, message.data.len());
                
                self.stats.lock().await.gossip_messages_received += 1;
                self.stats.lock().await.bytes_received += message.data.len() as u64;

                // Parse and handle the message
                if let Ok(gossip_message) = serde_json::from_slice::<GossipMessage>(&message.data) {
                    self.handle_gossip_message(gossip_message).await;
                }
            }
            _ => {}
        }
    }

    /// Handle parsed gossip message
    async fn handle_gossip_message(&self, message: GossipMessage) {
        // Convert to A2A message and forward to message broker
        let a2a_message = A2AMessage::new(
            Uuid::new_v4().to_string(),
            "gossip".to_string(),
            "broadcast".to_string(),
            "gossip".to_string(),
            serde_json::to_string(&message).unwrap_or_default(),
        );

        if let Err(e) = self.message_broker.send_message(a2a_message.to_message()).await {
            error!("Failed to forward gossip message to broker: {}", e);
        }
    }

    /// Publish a message to gossip network
    pub async fn publish(&self, topic: String, message: GossipMessage) -> anyhow::Result<()> {
        let mut swarm = self.swarm.lock().await;
        
        // Ensure we're subscribed to the topic
        let topic = gossipsub::IdentTopic::new(topic);
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

        // Serialize message
        let data = serde_json::to_vec(&message)?;

        // Publish message
        swarm.behaviour_mut().gossipsub.publish(topic, data)?;

        self.stats.lock().await.gossip_messages_sent += 1;
        self.stats.lock().await.bytes_sent += data.len() as u64;

        Ok(())
    }

    /// Broadcast service state to gossip network
    pub async fn broadcast_service_state(&self, service_info: ServiceInfo) -> anyhow::Result<()> {
        // Add to local services
        {
            let mut services_guard = self.services.write().await;
            services_guard.insert(service_info.id, service_info.clone());
        }

        // Create gossip message
        let message = GossipMessage::StateSync {
            sender_id: self.local_node.read().await.id,
            state_type: "service".to_string(),
            state_data: serde_json::to_value(&service_info)?,
            version: 1,
            timestamp: SystemTime::now(),
        };

        // Publish to service discovery topic
        self.publish("constellation/services".to_string(), message).await
    }

    /// Get protocol statistics
    pub async fn get_stats(&self) -> ProtocolStats {
        self.stats.lock().await.clone()
    }

    /// Get current membership
    pub async fn get_membership(&self) -> Membership {
        self.membership.read().await.clone()
    }

    /// Get known services
    pub async fn get_services(&self) -> Vec<ServiceInfo> {
        let services_guard = self.services.read().await;
        services_guard.values().cloned().collect()
    }

    /// Connect to a peer
    pub async fn connect(&self, addr: Multiaddr) -> anyhow::Result<()> {
        let mut swarm = self.swarm.lock().await;
        swarm.dial(addr)?;
        Ok(())
    }

    /// Get listening addresses
    pub async fn listen_addrs(&self) -> Vec<Multiaddr> {
        let swarm = self.swarm.lock().await;
        swarm.listeners().cloned().collect()
    }

    /// Get connected peers
    pub async fn connected_peers(&self) -> Vec<PeerId> {
        let swarm = self.swarm.lock().await;
        swarm.connected_peers().cloned().collect()
    }
}