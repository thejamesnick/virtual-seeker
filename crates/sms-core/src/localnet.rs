//! Local Solana network via Surfpool (decision: local by default, devnet is rate-limited).

use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};

use crate::{paths, util, SmsPaths};

pub const RPC_PORT: u16 = 8899;
pub const WS_PORT: u16 = 8900;

pub fn rpc_url() -> String {
    format!("http://127.0.0.1:{RPC_PORT}")
}

/// Finds `surfpool` on PATH or in `~/.local/bin`.
pub fn find_surfpool() -> Option<PathBuf> {
    util::which("surfpool").or_else(|| {
        let p = paths::user_home().ok()?.join(".local/bin/surfpool");
        util::is_executable(&p).then_some(p)
    })
}

pub fn is_running() -> bool {
    util::tcp_port_open(RPC_PORT)
}

/// Starts `surfpool start --offline --no-tui` in the background. Returns false if something already
/// listens on the RPC port (we reuse it).
pub fn start(sms: &SmsPaths) -> Result<bool> {
    if is_running() {
        return Ok(false);
    }
    let surfpool = find_surfpool().context("surfpool not found. Install it (see docs/05-dev-setup.md) or run without --localnet")?;
    sms.ensure()?;
    let log_path = sms.logs().join("surfpool.log");
    let log = File::create(&log_path)?;
    let child = Command::new(surfpool)
        .args(["start", "--offline", "--no-tui"])
        .current_dir(sms.run())
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .process_group(0)
        .spawn()
        .context("starting surfpool")?;
    std::fs::write(pid_file(sms), child.id().to_string())?;
    util::wait_for(Duration::from_secs(60), Duration::from_millis(500), || is_running().then_some(()))
        .with_context(|| format!("surfpool did not open port {RPC_PORT}; see {}", log_path.display()))?;
    Ok(true)
}

/// Stops the Surfpool that `sms` started (by pid file). Returns false if there was none.
pub fn stop(sms: &SmsPaths) -> Result<bool> {
    let pid_path = pid_file(sms);
    let Ok(pid) = std::fs::read_to_string(&pid_path) else {
        return Ok(false);
    };
    let _ = Command::new("kill").arg(pid.trim()).status();
    let _ = std::fs::remove_file(&pid_path);
    util::wait_for(Duration::from_secs(15), Duration::from_millis(500), || (!is_running()).then_some(()));
    Ok(true)
}

fn pid_file(sms: &SmsPaths) -> PathBuf {
    sms.run().join("surfpool.pid")
}
