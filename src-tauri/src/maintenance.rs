use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

        loop {
            thread::sleep(WAL_CHECKPOINT_INTERVAL);

            let result = {
                let conn_guard = db_conn.lock().unwrap();
                if let Some(conn) = conn_guard.as_ref() {
                    conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", [])
                } else {
                    // DB not ready
                    Ok(0) // Dummy result
                }
            };

            if let Err(e) = result {
                eprintln!("WAL checkpoint failed: {}", e);
            }

            if last_weekly_maintenance.elapsed() >= WEEKLY_VACUUM_INTERVAL {
                let db_conn_clone = Arc::clone(&db_conn);
                thread::spawn(move || {
                    let conn_guard = db_conn_clone.lock().unwrap();
                    if let Some(conn) = conn_guard.as_ref() {
                        if let Err(e) = run_maintenance(conn) {
                            eprintln!("Weekly maintenance failed: {}", e);
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
