use crate::error::{CacheError, NetworkError};
use crate::globals::SECURITY;
use crate::network::fetcher::{ResourceFetcher, ResourceResponse};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime};
use tracing::{info, warn};

const RAM_LIMIT: usize = 32 * 1024 * 1024; // 32 MB
const DISK_LIMIT: u64 = 512 * 1024 * 1024; // 512 MB
const RAM_TIER_MAX: usize = 512 * 1024; // files > 512 KB go disk-only
const METADATA_FILE: &str = "resource_cache_meta.json";

pub static RESOURCE_CACHE: OnceLock<ResourceCache> = OnceLock::new();

struct RamEntry {
    data: Arc<Vec<u8>>,
    mime: String,
    size: usize,
    last_used: Instant,
}

#[derive(Serialize, Deserialize, Clone)]
struct DiskEntry {
    hash: String,
    mime: String,
    size: u64,
    last_used: SystemTime,
}

pub struct CacheStats {
    pub ram_entries: usize,
    pub ram_bytes: usize,
    pub disk_entries: usize,
    pub disk_bytes: u64,
}

pub struct ResourceCache {
    ram: Mutex<HashMap<String, RamEntry>>,
    ram_total: AtomicUsize,
    disk: Mutex<IndexMap<String, DiskEntry>>,
    disk_total: AtomicU64,
    cache_dir: PathBuf,
    fetcher: ResourceFetcher,
}

impl ResourceCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();

        let (disk_entries, disk_total) = Self::load_metadata(&cache_dir);

        Self {
            ram: Mutex::new(HashMap::new()),
            ram_total: AtomicUsize::new(0),
            disk: Mutex::new(disk_entries),
            disk_total: AtomicU64::new(disk_total),
            cache_dir,
            fetcher: ResourceFetcher::new(),
        }
    }

    fn url_hash(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn metadata_path(cache_dir: &PathBuf) -> PathBuf {
        cache_dir.join(METADATA_FILE)
    }

    fn load_metadata(cache_dir: &PathBuf) -> (IndexMap<String, DiskEntry>, u64) {
        let path = Self::metadata_path(cache_dir);
        let Ok(data) = std::fs::read(&path) else {
            return (IndexMap::new(), 0);
        };
        let Ok(map) = serde_json::from_slice::<IndexMap<String, DiskEntry>>(&data) else {
            warn!("resource cache: failed to parse metadata, starting fresh");
            return (IndexMap::new(), 0);
        };

        let total = map.values().map(|e| e.size).sum();
        info!(
            "resource cache: loaded {} disk entries ({} bytes) from metadata",
            map.len(),
            total
        );
        (map, total)
    }

    fn save_metadata(&self) {
        let disk = self.disk.lock().unwrap();
        let path = Self::metadata_path(&self.cache_dir);
        match serde_json::to_vec(&*disk) {
            Ok(data) => {
                if let Err(e) = std::fs::write(&path, data) {
                    warn!("resource cache: failed to save metadata: {e}");
                }
            }
            Err(e) => warn!("resource cache: failed to serialize metadata: {e}"),
        }
    }

    fn evict_ram_if_needed(&self, incoming: usize) {
        let mut ram = self.ram.lock().unwrap();
        while self.ram_total.load(Ordering::Relaxed) + incoming > RAM_LIMIT && !ram.is_empty() {
            let oldest_key = ram
                .iter()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                if let Some(entry) = ram.remove(&key) {
                    self.ram_total.fetch_sub(entry.size, Ordering::Relaxed);
                    info!(
                        "resource cache: evicted from RAM url_hash={key} size={}",
                        entry.size
                    );
                }
            } else {
                break;
            }
        }
    }

    fn evict_disk_if_needed(&self, incoming: u64) {
        let mut disk = self.disk.lock().unwrap();
        while self.disk_total.load(Ordering::Relaxed) + incoming > DISK_LIMIT && !disk.is_empty() {
            let oldest_idx = disk
                .values()
                .enumerate()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(i, _)| i);

            if let Some(idx) = oldest_idx {
                let (url, entry) = disk.swap_remove_index(idx).unwrap();
                let file_path = self.cache_dir.join(&entry.hash);
                if let Err(e) = std::fs::remove_file(&file_path) {
                    warn!(
                        "resource cache: failed to remove disk entry {}: {e}",
                        entry.hash
                    );
                }
                self.disk_total.fetch_sub(entry.size, Ordering::Relaxed);
                info!(
                    "resource cache: evicted from disk url_hash={url} size={}",
                    entry.size
                );
            } else {
                break;
            }
        }
    }

    fn insert_ram(&self, url: &str, data: Arc<Vec<u8>>, mime: String) {
        let size = data.len();
        if size > RAM_TIER_MAX {
            return;
        }
        self.evict_ram_if_needed(size);
        let mut ram = self.ram.lock().unwrap();
        ram.insert(
            url.to_string(),
            RamEntry {
                data,
                mime,
                size,
                last_used: Instant::now(),
            },
        );
        self.ram_total.fetch_add(size, Ordering::Relaxed);
    }

    fn insert_disk(&self, url: &str, data: &[u8], mime: &str) {
        if let Err(e) = self.try_insert_disk(url, data, mime) {
            warn!("resource cache: disk write failed for {url}: {e}");
        }
    }

    fn try_insert_disk(&self, url: &str, data: &[u8], mime: &str) -> Result<(), CacheError> {
        let hash = Self::url_hash(url);
        let size = data.len() as u64;

        self.evict_disk_if_needed(size);

        let security = SECURITY
            .try_lock()
            .map_err(|e| CacheError::SecurityUnavailable(e.to_string()))?;
        let encrypted = security
            .encrypt(data)
            .map_err(|e| CacheError::Encryption(e.to_string()))?;
        drop(security);

        let file_path = self.cache_dir.join(&hash);
        std::fs::write(&file_path, &encrypted)?;

        let entry = DiskEntry {
            hash,
            mime: mime.to_string(),
            size,
            last_used: SystemTime::now(),
        };

        let mut disk = self.disk.lock().unwrap();
        disk.insert(url.to_string(), entry);
        self.disk_total.fetch_add(size, Ordering::Relaxed);
        drop(disk);

        self.save_metadata();
        Ok(())
    }

    fn read_disk(&self, url: &str) -> Option<(Vec<u8>, String)> {
        match self.try_read_disk(url) {
            Ok(result) => result,
            Err(e) => {
                warn!("resource cache: disk read failed for {url}: {e}");
                None
            }
        }
    }

    fn try_read_disk(&self, url: &str) -> Result<Option<(Vec<u8>, String)>, CacheError> {
        let mut disk = self.disk.lock().unwrap();
        let Some(entry) = disk.get_mut(url) else {
            return Ok(None);
        };
        entry.last_used = SystemTime::now();
        let hash = entry.hash.clone();
        let mime = entry.mime.clone();
        drop(disk);

        let file_path = self.cache_dir.join(&hash);
        let encrypted = std::fs::read(&file_path)?;

        let security = SECURITY
            .try_lock()
            .map_err(|e| CacheError::SecurityUnavailable(e.to_string()))?;
        let decrypted = security
            .decrypt(&encrypted)
            .map_err(|e| CacheError::Decryption(e.to_string()))?;
        drop(security);

        self.save_metadata();
        Ok(Some((decrypted, mime)))
    }

    pub async fn get_or_fetch(&self, url: &str) -> Result<(Arc<Vec<u8>>, String), NetworkError> {
        // 1. RAM hit
        {
            let mut ram = self.ram.lock().unwrap();
            if let Some(entry) = ram.get_mut(url) {
                entry.last_used = Instant::now();
                info!("resource cache: RAM hit url={url}");
                return Ok((Arc::clone(&entry.data), entry.mime.clone()));
            }
        }

        // 2. Disk hit
        if let Some((bytes, mime)) = self.read_disk(url) {
            info!("resource cache: disk hit url={url}");
            let data = Arc::new(bytes);
            self.insert_ram(url, Arc::clone(&data), mime.clone());
            return Ok((data, mime));
        }

        // 3. Fetch
        info!("resource cache: cache miss, fetching url={url}");
        let ResourceResponse { bytes, mime_type } = self.fetcher.fetch(url).await?;

        let data = Arc::new(bytes);
        self.insert_disk(url, &data, &mime_type);
        self.insert_ram(url, Arc::clone(&data), mime_type.clone());

        Ok((data, mime_type))
    }

    pub fn clear(&self) {
        let mut ram = self.ram.lock().unwrap();
        ram.clear();
        self.ram_total.store(0, Ordering::Relaxed);
        drop(ram);

        let mut disk = self.disk.lock().unwrap();
        for entry in disk.values() {
            let path = self.cache_dir.join(&entry.hash);
            std::fs::remove_file(&path).ok();
        }
        disk.clear();
        self.disk_total.store(0, Ordering::Relaxed);
        drop(disk);

        self.save_metadata();
        info!("resource cache: cleared");
    }

    pub fn stats(&self) -> CacheStats {
        let ram = self.ram.lock().unwrap();
        let disk = self.disk.lock().unwrap();
        CacheStats {
            ram_entries: ram.len(),
            ram_bytes: self.ram_total.load(Ordering::Relaxed),
            disk_entries: disk.len(),
            disk_bytes: self.disk_total.load(Ordering::Relaxed),
        }
    }

    pub fn b64_data_url(data: &[u8], mime: &str) -> String {
        format!("data:{};base64,{}", mime, STANDARD.encode(data))
    }
}
