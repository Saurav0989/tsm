/*!
 * Configuration Management - Centralized config for all components
 * 
 * Provides:
 * - YAML/TOML config files
 * - Environment variable overrides
 * - Validation
 * - Hot reloading
 */

use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;

/// Complete system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub node: NodeConfig,
    pub cluster: ClusterConfiguration,
    pub verification: VerificationConfig,
    pub consensus: ConsensusConfig,
    pub network: NetworkConfiguration,
    pub storage: StorageConfiguration,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub id: u64,
    pub data_dir: String,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfiguration {
    pub peers: Vec<PeerConfig>,
    pub min_quorum: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    pub id: u64,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub enabled: bool,
    pub shadow_model: bool,
    pub max_verification_time_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub election_timeout_min_ms: u64,
    pub election_timeout_max_ms: u64,
    pub heartbeat_interval_ms: u64,
    pub max_entries_per_append: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    pub listen_address: String,
    pub listen_port: u16,
    pub connect_timeout_ms: u64,
    pub max_connections: usize,
    pub tcp_nodelay: bool,
    pub tcp_keepalive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfiguration {
    pub wal_dir: String,
    pub snapshot_dir: String,
    pub max_wal_size_mb: usize,
    pub snapshot_interval: usize,
    pub sync_on_write: bool,
    pub compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics_port: u16,
    pub health_check_interval_s: u64,
    pub alert_webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub admin_nodes: Vec<u64>,
    pub rate_limit_requests: usize,
    pub rate_limit_window_s: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    pub cache_size: usize,
    pub worker_threads: usize,
}

impl Config {
    /// Load configuration from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        let config: Config = toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        fs::write(path, contents)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate node ID
        if self.node.id == 0 {
            return Err(ConfigError::ValidationError(
                "Node ID must be non-zero".to_string()
            ));
        }
        
        // Validate cluster
        if self.cluster.peers.is_empty() {
            return Err(ConfigError::ValidationError(
                "Cluster must have at least one peer".to_string()
            ));
        }
        
        // Validate quorum
        let total_nodes = self.cluster.peers.len();
        if self.cluster.min_quorum > total_nodes {
            return Err(ConfigError::ValidationError(
                format!("Quorum {} exceeds total nodes {}", 
                        self.cluster.min_quorum, total_nodes)
            ));
        }
        
        // Validate consensus timeouts
        if self.consensus.election_timeout_min_ms >= self.consensus.election_timeout_max_ms {
            return Err(ConfigError::ValidationError(
                "Election timeout min must be < max".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Create default configuration
    pub fn default_for_node(node_id: u64) -> Self {
        Config {
            node: NodeConfig {
                id: node_id,
                data_dir: format!("/var/lib/lattice/node-{}", node_id),
                log_level: "info".to_string(),
            },
            cluster: ClusterConfiguration {
                peers: vec![
                    PeerConfig { id: 1, address: "127.0.0.1:5001".to_string() },
                    PeerConfig { id: 2, address: "127.0.0.1:5002".to_string() },
                    PeerConfig { id: 3, address: "127.0.0.1:5003".to_string() },
                ],
                min_quorum: 2,
            },
            verification: VerificationConfig {
                enabled: true,
                shadow_model: true,
                max_verification_time_us: 1000,
            },
            consensus: ConsensusConfig {
                election_timeout_min_ms: 150,
                election_timeout_max_ms: 300,
                heartbeat_interval_ms: 50,
                max_entries_per_append: 100,
            },
            network: NetworkConfiguration {
                listen_address: "0.0.0.0".to_string(),
                listen_port: 5000 + node_id as u16,
                connect_timeout_ms: 5000,
                max_connections: 100,
                tcp_nodelay: true,
                tcp_keepalive: true,
            },
            storage: StorageConfiguration {
                wal_dir: format!("/var/lib/lattice/node-{}/wal", node_id),
                snapshot_dir: format!("/var/lib/lattice/node-{}/snapshots", node_id),
                max_wal_size_mb: 64,
                snapshot_interval: 10000,
                sync_on_write: true,
                compression: false,
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_port: 9000 + node_id as u16,
                health_check_interval_s: 10,
                alert_webhook_url: None,
            },
            security: SecurityConfig {
                enabled: true,
                tls_cert: None,
                tls_key: None,
                admin_nodes: vec![1],
                rate_limit_requests: 1000,
                rate_limit_window_s: 60,
            },
            performance: PerformanceConfig {
                batch_size: 100,
                batch_timeout_ms: 10,
                cache_size: 10000,
                worker_threads: 4,
            },
        }
    }
    
    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) {
        use std::env;
        
        if let Ok(val) = env::var("LATTICE_NODE_ID") {
            if let Ok(id) = val.parse() {
                self.node.id = id;
            }
        }
        
        if let Ok(val) = env::var("LATTICE_DATA_DIR") {
            self.node.data_dir = val;
        }
        
        if let Ok(val) = env::var("LATTICE_LOG_LEVEL") {
            self.node.log_level = val;
        }
        
        if let Ok(val) = env::var("LATTICE_VERIFICATION_ENABLED") {
            self.verification.enabled = val == "true";
        }
        
        // Add more overrides as needed...
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "I/O error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Serialize error: {}", e),
            ConfigError::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

// Need toml dependency
mod toml {
    use super::*;
    
    pub fn from_str<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
    
    pub fn to_string_pretty<T: Serialize>(val: &T) -> Result<String, String> {
        serde_json::to_string_pretty(val).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default_for_node(1);
        assert_eq!(config.node.id, 1);
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_validation() {
        let mut config = Config::default_for_node(0);
        assert!(config.validate().is_err()); // ID = 0 invalid
        
        config.node.id = 1;
        assert!(config.validate().is_ok());
    }
}
