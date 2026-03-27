use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;
use tracing;

use crate::db::{run_maintenance, DbPool};
use crate::globals::get_db_pool;

const WAL_CHECKPOINT_INTERVAL: Duration = Duration::from_secs(3600);
const WEEKLY_VACUUM_INTERVAL: Duration = Duration::from_secs(7 * 24 * 3600);

static MAINTENANCE_RUNNING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn start_maintenance_scheduler(_db_pool: DbPool) {
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

    task::spawn(async move {
        let mut last_weekly_maintenance = tokio::time::Instant::now();
        tracing::info!(target: "postail", "Maintenance scheduler started");

        loop {
            // Check flag every 1s for 10s total sleep to respond to shutdown
            for _ in 0..10 {
                if !MAINTENANCE_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            if !MAINTENANCE_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                tracing::info!(target: "postail", "Maintenance scheduler stopping");
                break;
            }

            static LAST_CHECKPOINT: Mutex<Option<tokio::time::Instant>> = Mutex::const_new(None);

            let should_checkpoint = {
                let mut last = LAST_CHECKPOINT.lock().await;
                if last.is_none() || last.unwrap().elapsed() >= WAL_CHECKPOINT_INTERVAL {
                    *last = Some(tokio::time::Instant::now());
                    true
                } else {
                    false
                }
            };

            if should_checkpoint {
                tracing::info!(target: "postail", "[Maintenance] Running WAL checkpoint...");
                let result = {
                    match get_db_pool().await {
                        Ok(pool) => match pool.get() {
                            Ok(conn) => conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
                                let busy: i32 = row.get(0)?;
                                let log: i32 = row.get(1)?;
                                let checkpointed: i32 = row.get(2)?;
                                Ok((busy, log, checkpointed))
                            }),
                            Err(_) => Ok((0, 0, 0)),
                        },
                        Err(_) => Ok((0, 0, 0)),
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
                task::spawn(async move {
                    match get_db_pool().await {
                        Ok(pool) => {
                            if let Ok(conn) = pool.get() {
                                if let Err(e) = run_maintenance(&*conn) {
                                    tracing::error!(target: "postail", "Weekly maintenance failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!(target: "postail", "Weekly maintenance failed: {}", e);
                        }
                    }
                });

                last_weekly_maintenance = tokio::time::Instant::now();
            }
        }
    });
}

pub fn stop_maintenance_scheduler() {
    MAINTENANCE_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
}
