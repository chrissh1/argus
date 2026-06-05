//! Manage the bundled Screenpipe binary as a per-session child process.
//!
//! Lifecycle:
//!   * `spawn` — start the binary writing into the session's raw.db
//!   * `pause`/`resume` — SIGSTOP/SIGCONT to suspend capture without dropping state
//!   * `stop` — SIGTERM, escalate to SIGKILL after a 2s grace period
//!
//! Idle CPU is 0% because we kill the process completely at session end.

use crate::{ArgusError, ArgusResult};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct Screenpipe {
    child: Mutex<Option<Child>>,
}

impl Screenpipe {
    pub const fn new() -> Self {
        Screenpipe { child: Mutex::new(None) }
    }

    pub fn spawn(&self, binary: &Path, raw_db: &Path) -> ArgusResult<()> {
        let mut guard = self.child.lock().unwrap();
        if guard.is_some() {
            return Err(ArgusError::InvalidState(
                "screenpipe already running".into(),
            ));
        }
        let parent = raw_db.parent().ok_or_else(|| {
            ArgusError::Other(format!("raw_db has no parent: {}", raw_db.display()))
        })?;
        std::fs::create_dir_all(parent)?;

        let child = Command::new(binary)
            .arg("--start-audio")
            .arg("--ocr")
            .arg("--db")
            .arg(raw_db)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        tracing::info!(pid = child.id(), "screenpipe spawned");
        *guard = Some(child);
        Ok(())
    }

    pub fn pause(&self) -> ArgusResult<()> {
        self.signal(Signal::SIGSTOP)
    }

    pub fn resume(&self) -> ArgusResult<()> {
        self.signal(Signal::SIGCONT)
    }

    fn signal(&self, sig: Signal) -> ArgusResult<()> {
        let guard = self.child.lock().unwrap();
        let child = guard
            .as_ref()
            .ok_or_else(|| ArgusError::InvalidState("screenpipe not running".into()))?;
        kill(Pid::from_raw(child.id() as i32), sig)
            .map_err(|e| ArgusError::Other(format!("kill({sig:?}): {e}")))?;
        Ok(())
    }

    pub fn stop(&self) -> ArgusResult<()> {
        let mut guard = self.child.lock().unwrap();
        let Some(mut child) = guard.take() else {
            return Ok(());
        };

        // SIGCONT first in case we were paused (otherwise SIGTERM is queued but
        // never delivered until resumed).
        let pid = Pid::from_raw(child.id() as i32);
        let _ = kill(pid, Signal::SIGCONT);
        let _ = kill(pid, Signal::SIGTERM);

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match child.try_wait()? {
                Some(_) => return Ok(()),
                None if Instant::now() >= deadline => break,
                None => std::thread::sleep(Duration::from_millis(100)),
            }
        }
        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().unwrap().is_some()
    }
}
