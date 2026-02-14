/*!
 * Network Layer - TCP communication between nodes
 * 
 * Handles:
 * - Connection management
 * - Message serialization
 * - Network failures
 * - Retries
 */

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

use crate::raft::RaftMessage;
use crate::NodeId;

/// NetworkConfig - Configuration for network layer
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// My address
    pub listen_addr: String,
    
    /// Peer addresses
    pub peers: HashMap<NodeId, String>,
    
    /// Connection timeout (ms)
    pub connect_timeout_ms: u64,
    
    /// Retry attempts
    pub retry_attempts: usize,
}

/// NetworkLayer - Manages connections between nodes
pub struct NetworkLayer {
    /// My node ID
    node_id: NodeId,
    
    /// Configuration
    config: NetworkConfig,
    
    /// Active connections
    connections: Arc<RwLock<HashMap<NodeId, Connection>>>,
    
    /// Incoming message queue
    inbox: Arc<RwLock<Vec<RaftMessage>>>,
}

struct Connection {
    stream: TcpStream,
    remote_id: NodeId,
}

impl NetworkLayer {
    pub fn new(node_id: NodeId, config: NetworkConfig) -> Self {
        NetworkLayer {
            node_id,
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            inbox: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Start listening for incoming connections
    pub async fn start(&self) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(&self.config.listen_addr)
            .await
            .map_err(|e| NetworkError::BindError(e.to_string()))?;
        
        println!("[Network] Listening on {}", self.config.listen_addr);
        
        // Accept connections
        let connections = self.connections.clone();
        let inbox = self.inbox.clone();
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        println!("[Network] Accepted connection from {}", addr);
                        
                        // Handle connection
                        let inbox = inbox.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, inbox).await {
                                eprintln!("[Network] Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("[Network] Accept error: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Send a message to a peer
    pub async fn send(&self, msg: RaftMessage) -> Result<(), NetworkError> {
        let peer_addr = self.config.peers.get(&msg.to)
            .ok_or_else(|| NetworkError::UnknownPeer(msg.to))?;
        
        // Try to get existing connection
        let mut conn = {
            let connections = self.connections.read();
            connections.get(&msg.to).map(|c| c.stream.try_clone())
        };
        
        // If no connection, establish one
        if conn.is_none() {
            let stream = TcpStream::connect(peer_addr)
                .await
                .map_err(|e| NetworkError::ConnectError(e.to_string()))?;
            
            self.connections.write().insert(msg.to, Connection {
                stream: stream.try_clone().unwrap(),
                remote_id: msg.to,
            });
            
            conn = Some(Ok(stream));
        }
        
        // Send message
        if let Some(Ok(mut stream)) = conn {
            let bytes = serialize_message(&msg)?;
            
            // Write length prefix
            stream.write_u32(bytes.len() as u32).await
                .map_err(|e| NetworkError::SendError(e.to_string()))?;
            
            // Write message
            stream.write_all(&bytes).await
                .map_err(|e| NetworkError::SendError(e.to_string()))?;
            
            stream.flush().await
                .map_err(|e| NetworkError::SendError(e.to_string()))?;
            
            Ok(())
        } else {
            Err(NetworkError::ConnectError("Failed to establish connection".to_string()))
        }
    }
    
    /// Receive pending messages
    pub fn receive(&self) -> Vec<RaftMessage> {
        let mut inbox = self.inbox.write();
        std::mem::take(&mut *inbox)
    }
}

/// Handle an incoming connection
async fn handle_connection(
    mut stream: TcpStream,
    inbox: Arc<RwLock<Vec<RaftMessage>>>,
) -> Result<(), NetworkError> {
    loop {
        // Read length prefix
        let len = match stream.read_u32().await {
            Ok(len) => len as usize,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Connection closed
                return Ok(());
            }
            Err(e) => {
                return Err(NetworkError::ReceiveError(e.to_string()));
            }
        };
        
        // Read message
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf).await
            .map_err(|e| NetworkError::ReceiveError(e.to_string()))?;
        
        // Deserialize
        let msg = deserialize_message(&buf)?;
        
        // Add to inbox
        inbox.write().push(msg);
    }
}

/// Serialize a message
fn serialize_message(msg: &RaftMessage) -> Result<Vec<u8>, NetworkError> {
    bincode::serialize(msg)
        .map_err(|e| NetworkError::SerializeError(e.to_string()))
}

/// Deserialize a message
fn deserialize_message(bytes: &[u8]) -> Result<RaftMessage, NetworkError> {
    bincode::deserialize(bytes)
        .map_err(|e| NetworkError::DeserializeError(e.to_string()))
}

#[derive(Debug)]
pub enum NetworkError {
    BindError(String),
    ConnectError(String),
    SendError(String),
    ReceiveError(String),
    SerializeError(String),
    DeserializeError(String),
    UnknownPeer(NodeId),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::BindError(e) => write!(f, "Bind error: {}", e),
            NetworkError::ConnectError(e) => write!(f, "Connect error: {}", e),
            NetworkError::SendError(e) => write!(f, "Send error: {}", e),
            NetworkError::ReceiveError(e) => write!(f, "Receive error: {}", e),
            NetworkError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            NetworkError::DeserializeError(e) => write!(f, "Deserialize error: {}", e),
            NetworkError::UnknownPeer(id) => write!(f, "Unknown peer: {}", id),
        }
    }
}

impl std::error::Error for NetworkError {}
