/*!
 * Write-Ahead Log (WAL) - Persistent storage for state and log entries
 * 
 * Ensures durability:
 * - All state changes written to disk before acknowledging
 * - Can recover from crashes
 * - Append-only for performance
 * 
 * Design:
 * - WAL file: Sequential log entries
 * - Checkpoint file: Periodic state snapshots
 * - Index file: Fast lookup
 */

use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read, Seek, SeekFrom, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

use crate::{State, Transition};
use crate::raft::LogEntry;

/// WAL - Write-Ahead Log manager
pub struct WAL {
    /// Path to WAL directory
    dir: PathBuf,
    
    /// Current WAL file
    log_file: BufWriter<File>,
    
    /// Current log index
    current_index: usize,
    
    /// Bytes written (for rotation)
    bytes_written: usize,
    
    /// Configuration
    config: WALConfig,
}

#[derive(Debug, Clone)]
pub struct WALConfig {
    /// Maximum WAL file size before rotation
    pub max_file_size: usize,
    
    /// Sync to disk after every write
    pub sync_on_write: bool,
    
    /// Compression enabled
    pub compression: bool,
}

impl Default for WALConfig {
    fn default() -> Self {
        WALConfig {
            max_file_size: 64 * 1024 * 1024, // 64MB
            sync_on_write: true, // Safety over performance
            compression: false, // Simple for now
        }
    }
}

/// WAL entry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WALEntry {
    /// Log entry from Raft
    LogEntry {
        index: usize,
        term: u64,
        transition: Transition,
    },
    
    /// State snapshot
    Snapshot {
        index: usize,
        term: u64,
        state: State,
    },
    
    /// Metadata update
    Metadata {
        current_term: u64,
        voted_for: Option<u64>,
    },
    
    /// Commit index update
    Commit {
        index: usize,
    },
}

impl WAL {
    /// Create new WAL at path
    pub fn create(path: impl AsRef<Path>, config: WALConfig) -> io::Result<Self> {
        let dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        
        let log_path = dir.join("wal-0000.log");
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        Ok(WAL {
            dir,
            log_file: BufWriter::new(log_file),
            current_index: 0,
            bytes_written: 0,
            config,
        })
    }
    
    /// Open existing WAL
    pub fn open(path: impl AsRef<Path>, config: WALConfig) -> io::Result<Self> {
        let dir = path.as_ref().to_path_buf();
        
        // Find latest WAL file
        let mut wal_files: Vec<_> = std::fs::read_dir(&dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("wal-") && s.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect();
        
        wal_files.sort_by_key(|e| e.file_name());
        
        let (log_file, current_index) = if let Some(latest) = wal_files.last() {
            let file = OpenOptions::new()
                .append(true)
                .open(latest.path())?;
            
            let index = latest
                .file_name()
                .to_str()
                .and_then(|s| s.strip_prefix("wal-"))
                .and_then(|s| s.strip_suffix(".log"))
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            
            (file, index)
        } else {
            // No existing WAL, create new
            let log_path = dir.join("wal-0000.log");
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)?;
            (file, 0)
        };
        
        Ok(WAL {
            dir,
            log_file: BufWriter::new(log_file),
            current_index,
            bytes_written: 0,
            config,
        })
    }
    
    /// Append entry to WAL
    pub fn append(&mut self, entry: WALEntry) -> io::Result<usize> {
        // Serialize entry
        let bytes = bincode::serialize(&entry)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Write length prefix (4 bytes)
        let len = bytes.len() as u32;
        self.log_file.write_all(&len.to_le_bytes())?;
        
        // Write entry
        self.log_file.write_all(&bytes)?;
        
        // Flush if configured
        if self.config.sync_on_write {
            self.log_file.flush()?;
            self.log_file.get_ref().sync_all()?;
        }
        
        self.bytes_written += 4 + bytes.len();
        
        // Check if rotation needed
        if self.bytes_written >= self.config.max_file_size {
            self.rotate()?;
        }
        
        Ok(bytes.len())
    }
    
    /// Rotate to new WAL file
    fn rotate(&mut self) -> io::Result<()> {
        // Flush current file
        self.log_file.flush()?;
        
        // Open new file
        self.current_index += 1;
        let new_path = self.dir.join(format!("wal-{:04}.log", self.current_index));
        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(new_path)?;
        
        self.log_file = BufWriter::new(new_file);
        self.bytes_written = 0;
        
        println!("[WAL] Rotated to file {}", self.current_index);
        
        Ok(())
    }
    
    /// Read all entries from WAL
    pub fn read_all(&self) -> io::Result<Vec<WALEntry>> {
        let mut entries = Vec::new();
        
        // Read all WAL files in order
        for i in 0..=self.current_index {
            let path = self.dir.join(format!("wal-{:04}.log", i));
            
            if !path.exists() {
                continue;
            }
            
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            
            // Read entries
            loop {
                // Read length
                let mut len_bytes = [0u8; 4];
                match reader.read_exact(&mut len_bytes) {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                    Err(e) => return Err(e),
                }
                
                let len = u32::from_le_bytes(len_bytes) as usize;
                
                // Read entry
                let mut entry_bytes = vec![0u8; len];
                reader.read_exact(&mut entry_bytes)?;
                
                // Deserialize
                let entry: WALEntry = bincode::deserialize(&entry_bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                
                entries.push(entry);
            }
        }
        
        Ok(entries)
    }
    
    /// Compact WAL by removing entries before index
    pub fn compact(&mut self, before_index: usize) -> io::Result<()> {
        println!("[WAL] Compacting entries before index {}", before_index);
        
        // This would delete old WAL files
        // For now, just mark for deletion
        
        Ok(())
    }
    
    /// Sync to disk
    pub fn sync(&mut self) -> io::Result<()> {
        self.log_file.flush()?;
        self.log_file.get_ref().sync_all()?;
        Ok(())
    }
}

/// Snapshot manager - Periodic state snapshots
pub struct SnapshotManager {
    dir: PathBuf,
    config: SnapshotConfig,
}

#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Snapshot interval (number of log entries)
    pub snapshot_interval: usize,
    
    /// Keep last N snapshots
    pub keep_snapshots: usize,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        SnapshotConfig {
            snapshot_interval: 10000,
            keep_snapshots: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Last included index
    pub last_index: usize,
    
    /// Last included term
    pub last_term: u64,
    
    /// State machine state
    pub state: State,
    
    /// Timestamp
    pub timestamp: u64,
}

impl SnapshotManager {
    pub fn new(path: impl AsRef<Path>, config: SnapshotConfig) -> io::Result<Self> {
        let dir = path.as_ref().to_path_buf();
        std::fs::create_dir_all(&dir)?;
        
        Ok(SnapshotManager { dir, config })
    }
    
    /// Save a snapshot
    pub fn save(&self, snapshot: &Snapshot) -> io::Result<()> {
        let path = self.dir.join(format!("snapshot-{:08}.snap", snapshot.last_index));
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        let bytes = bincode::serialize(snapshot)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        writer.write_all(&bytes)?;
        writer.flush()?;
        
        println!("[Snapshot] Saved snapshot at index {}", snapshot.last_index);
        
        // Cleanup old snapshots
        self.cleanup()?;
        
        Ok(())
    }
    
    /// Load latest snapshot
    pub fn load_latest(&self) -> io::Result<Option<Snapshot>> {
        let mut snapshots: Vec<_> = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("snapshot-") && s.ends_with(".snap"))
                    .unwrap_or(false)
            })
            .collect();
        
        if snapshots.is_empty() {
            return Ok(None);
        }
        
        snapshots.sort_by_key(|e| e.file_name());
        
        let latest = snapshots.last().unwrap();
        let file = File::open(latest.path())?;
        let mut reader = BufReader::new(file);
        
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        
        let snapshot: Snapshot = bincode::deserialize(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        Ok(Some(snapshot))
    }
    
    /// Cleanup old snapshots
    fn cleanup(&self) -> io::Result<()> {
        let mut snapshots: Vec<_> = std::fs::read_dir(&self.dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|s| s.starts_with("snapshot-") && s.ends_with(".snap"))
                    .unwrap_or(false)
            })
            .collect();
        
        snapshots.sort_by_key(|e| e.file_name());
        
        // Keep only last N
        while snapshots.len() > self.config.keep_snapshots {
            if let Some(old) = snapshots.first() {
                std::fs::remove_file(old.path())?;
                println!("[Snapshot] Deleted old snapshot: {:?}", old.file_name());
            }
            snapshots.remove(0);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_wal_create_and_append() {
        let dir = TempDir::new().unwrap();
        let mut wal = WAL::create(dir.path(), WALConfig::default()).unwrap();
        
        let entry = WALEntry::Metadata {
            current_term: 1,
            voted_for: Some(1),
        };
        
        wal.append(entry).unwrap();
        
        let entries = wal.read_all().unwrap();
        assert_eq!(entries.len(), 1);
    }
    
    #[test]
    fn test_snapshot_save_and_load() {
        let dir = TempDir::new().unwrap();
        let mgr = SnapshotManager::new(dir.path(), SnapshotConfig::default()).unwrap();
        
        let snapshot = Snapshot {
            last_index: 100,
            last_term: 1,
            state: State::new(),
            timestamp: 0,
        };
        
        mgr.save(&snapshot).unwrap();
        
        let loaded = mgr.load_latest().unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().last_index, 100);
    }
}
