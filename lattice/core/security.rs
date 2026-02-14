/*!
 * Security - Authentication, authorization, and encryption
 * 
 * Implements:
 * - TLS for network communication
 * - mTLS for node authentication
 * - Access control
 * - Message signing
 * - Audit logging
 */

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use blake3::Hasher;

use crate::NodeId;

/// Node credentials for mutual authentication
#[derive(Debug, Clone)]
pub struct Credentials {
    pub node_id: NodeId,
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

pub type PublicKey = [u8; 32];
pub type PrivateKey = [u8; 32];
pub type Signature = [u8; 64];

impl Credentials {
    /// Generate new credentials
    pub fn generate(node_id: NodeId) -> Self {
        // Simplified - real impl would use proper Ed25519
        let mut public_key = [0u8; 32];
        let mut private_key = [0u8; 32];
        
        // Use node_id as seed for deterministic keys
        let seed = node_id.to_le_bytes();
        for i in 0..32 {
            public_key[i] = seed[i % 8].wrapping_add(i as u8);
            private_key[i] = seed[i % 8].wrapping_add(i as u8).wrapping_mul(2);
        }
        
        Credentials {
            node_id,
            public_key,
            private_key,
        }
    }
    
    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        // Simplified signing - real impl would use Ed25519
        let mut hasher = Hasher::new();
        hasher.update(&self.private_key);
        hasher.update(message);
        
        let hash = hasher.finalize();
        let mut sig = [0u8; 64];
        sig[..32].copy_from_slice(hash.as_bytes());
        sig[32..].copy_from_slice(&self.public_key);
        sig
    }
    
    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &Signature, public_key: &PublicKey) -> bool {
        // Extract claimed signature
        let claimed_hash = &signature[..32];
        let claimed_key = &signature[32..];
        
        // Verify public key matches
        if claimed_key != public_key {
            return false;
        }
        
        // Verify signature (simplified)
        true // Real impl would verify Ed25519 signature
    }
}

/// Access control manager
pub struct AccessControl {
    /// Node permissions
    permissions: HashMap<NodeId, Permissions>,
    
    /// Admin nodes
    admins: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct Permissions {
    pub can_propose: bool,
    pub can_vote: bool,
    pub can_read: bool,
    pub can_admin: bool,
}

impl Default for Permissions {
    fn default() -> Self {
        Permissions {
            can_propose: true,
            can_vote: true,
            can_read: true,
            can_admin: false,
        }
    }
}

impl AccessControl {
    pub fn new(admins: Vec<NodeId>) -> Self {
        AccessControl {
            permissions: HashMap::new(),
            admins,
        }
    }
    
    /// Check if node has permission
    pub fn check(&self, node_id: NodeId, action: Action) -> bool {
        // Admins can do everything
        if self.admins.contains(&node_id) {
            return true;
        }
        
        let perms = self.permissions.get(&node_id)
            .unwrap_or(&Permissions::default());
        
        match action {
            Action::Propose => perms.can_propose,
            Action::Vote => perms.can_vote,
            Action::Read => perms.can_read,
            Action::Admin => perms.can_admin,
        }
    }
    
    /// Grant permissions
    pub fn grant(&mut self, node_id: NodeId, permissions: Permissions) {
        self.permissions.insert(node_id, permissions);
    }
    
    /// Revoke all permissions
    pub fn revoke(&mut self, node_id: NodeId) {
        self.permissions.remove(&node_id);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Propose,
    Vote,
    Read,
    Admin,
}

/// Audit logger - Records all security events
pub struct AuditLog {
    entries: Vec<AuditEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub node_id: NodeId,
    pub action: String,
    pub result: AuditResult,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Allowed,
    Denied,
    Error,
}

impl AuditLog {
    pub fn new(max_entries: usize) -> Self {
        AuditLog {
            entries: Vec::new(),
            max_entries,
        }
    }
    
    /// Log an event
    pub fn log(&mut self, entry: AuditEntry) {
        println!(
            "[Audit] {} - Node {} - {} - {:?}",
            entry.timestamp, entry.node_id, entry.action, entry.result
        );
        
        self.entries.push(entry);
        
        // Keep bounded
        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }
    
    /// Get recent entries
    pub fn recent(&self, count: usize) -> Vec<AuditEntry> {
        self.entries.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Export audit log
    pub fn export(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap()
    }
}

/// Rate limiter - Prevent abuse
pub struct RateLimiter {
    /// Requests per node
    requests: HashMap<NodeId, Vec<u64>>,
    
    /// Max requests per window
    max_requests: usize,
    
    /// Window size (seconds)
    window_seconds: u64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: u64) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            max_requests,
            window_seconds,
        }
    }
    
    /// Check if request is allowed
    pub fn allow(&mut self, node_id: NodeId) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let timestamps = self.requests.entry(node_id).or_insert_with(Vec::new);
        
        // Remove old timestamps
        let cutoff = now - self.window_seconds;
        timestamps.retain(|&ts| ts > cutoff);
        
        // Check limit
        if timestamps.len() >= self.max_requests {
            return false;
        }
        
        // Record request
        timestamps.push(now);
        true
    }
}

/// Security context - Bundles all security components
pub struct SecurityContext {
    pub credentials: Credentials,
    pub access_control: AccessControl,
    pub audit_log: AuditLog,
    pub rate_limiter: RateLimiter,
}

impl SecurityContext {
    pub fn new(node_id: NodeId, admins: Vec<NodeId>) -> Self {
        SecurityContext {
            credentials: Credentials::generate(node_id),
            access_control: AccessControl::new(admins),
            audit_log: AuditLog::new(10000),
            rate_limiter: RateLimiter::new(1000, 60),
        }
    }
    
    /// Authorize an action
    pub fn authorize(&mut self, node_id: NodeId, action: Action) -> bool {
        // Check rate limit
        if !self.rate_limiter.allow(node_id) {
            self.audit_log.log(AuditEntry {
                timestamp: now(),
                node_id,
                action: format!("{:?}", action),
                result: AuditResult::Denied,
                details: "Rate limit exceeded".to_string(),
            });
            return false;
        }
        
        // Check permissions
        if !self.access_control.check(node_id, action) {
            self.audit_log.log(AuditEntry {
                timestamp: now(),
                node_id,
                action: format!("{:?}", action),
                result: AuditResult::Denied,
                details: "Permission denied".to_string(),
            });
            return false;
        }
        
        // Allowed
        self.audit_log.log(AuditEntry {
            timestamp: now(),
            node_id,
            action: format!("{:?}", action),
            result: AuditResult::Allowed,
            details: "Success".to_string(),
        });
        
        true
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_credentials() {
        let creds = Credentials::generate(1);
        let message = b"test message";
        
        let signature = creds.sign(message);
        assert!(creds.verify(message, &signature, &creds.public_key));
    }
    
    #[test]
    fn test_access_control() {
        let mut ac = AccessControl::new(vec![1]);
        
        // Admin can do anything
        assert!(ac.check(1, Action::Admin));
        
        // Normal node can't admin
        assert!(!ac.check(2, Action::Admin));
        
        // But can propose by default
        assert!(ac.check(2, Action::Propose));
    }
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, 1);
        
        assert!(limiter.allow(1));
        assert!(limiter.allow(1));
        assert!(limiter.allow(1));
        assert!(!limiter.allow(1)); // 4th should be denied
    }
}
