use email_network::network::cache::RESOURCE_CACHE;
use serde::Serialize;
use tauri::command;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStats {
    pub ram_entries: usize,
    pub ram_bytes: usize,
    pub disk_entries: usize,
    pub disk_bytes: u64,
}

#[command]
pub fn clear_resource_cache() -> Result<(), String> {
    let cache = RESOURCE_CACHE
        .get()
        .ok_or("resource cache not initialized")?;
    cache.clear();
    Ok(())
}

#[command]
pub fn get_resource_cache_stats() -> Result<CacheStats, String> {
    let cache = RESOURCE_CACHE
        .get()
        .ok_or("resource cache not initialized")?;
    let stats = cache.stats();
    Ok(CacheStats {
        ram_entries: stats.ram_entries,
        ram_bytes: stats.ram_bytes,
        disk_entries: stats.disk_entries,
        disk_bytes: stats.disk_bytes,
    })
}
