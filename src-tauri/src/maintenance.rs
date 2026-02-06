use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tracing;

use rusqlite::Connection;

use crate::db::run_maintenance;

const WAL_CHECKPOINT_INTERVAL: Duration = Duration::from_secs(3600);
const WEEKLY_VACUUM_INTERVAL: Duration = Duration::from_secs(7 * 24 * 3600);

static MAINTENANCE_RUNNING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn start_maintenance_scheduler(db_conn: Arc<Mutex<Option<Connection>>>) {
    if MAINTENANCE_RUNNING
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return;
    }

    thread::spawn(move || {
        let mut last_weekly_maintenance = std::time::Instant::now();
        tracing::info!(target: "postail", "Maintenance scheduler started");

        loop {
            // Check flag every 1s for 10s total sleep to respond to shutdown
            for _ in 0..10 {
                if !MAINTENANCE_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
            
            if !MAINTENANCE_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::info!(target: "postail", "Maintenance scheduler stopping");
                break;
            }
            
            static LAST_CHECKPOINT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
            
            let should_checkpoint = {
                let mut last = LAST_CHECKPOINT.lock().unwrap();
                if last.is_none() || last.unwrap().elapsed() >= WAL_CHECKPOINT_INTERVAL {
                    *last = Some(std::time::Instant::now());
                    true
                } else {
                    false
                }
            };

            if should_checkpoint {
                tracing::info!(target: "postail", "[Maintenance] Running WAL checkpoint...");
                let result = {
                    let conn_guard = db_conn.lock().unwrap();
                    if let Some(conn) = conn_guard.as_ref() {
                        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
                            let busy: i32 = row.get(0)?;
                            let log: i32 = row.get(1)?;
                            let checkpointed: i32 = row.get(2)?;
                            Ok((busy, log, checkpointed))
                        })
                    } else {
                        Ok((0, 0, 0))
                    }
                };

                match result {
                    Ok((busy, log, cp)) => {
                        tracing::info!(target: "postail", "[Maintenance] WAL checkpoint done: busy={}, log={}, checkpointed={}", busy, log, cp);
                    }
                    Err(e) => {
                        tracing::warn!(target: "postail", "[Maintenance] WAL checkpoint failed: {}", e);
                    }
                }
            }

            if last_weekly_maintenance.elapsed() >= WEEKLY_VACUUM_INTERVAL {
                let db_conn_clone = Arc::clone(&db_conn);
                thread::spawn(move || {
                    let conn_guard = db_conn_clone.lock().unwrap();
                    if let Some(conn) = conn_guard.as_ref() {
                        if let Err(e) = run_maintenance(conn) {
                            tracing::error!(target: "postail", "Weekly maintenance failed: {}", e);
                        }
                    }
                });

                last_weekly_maintenance = std::time::Instant::now();
            }
        }
    });
}

pub fn stop_maintenance_scheduler() {
    MAINTENANCE_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
}
