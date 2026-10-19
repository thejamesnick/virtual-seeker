use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

/// Finds an executable on `PATH`.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(name))
        .find(|candidate| is_executable(candidate))
}

pub fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Runs a command and returns its output, failing with stderr if it exits non-zero.
pub fn run(cmd: &mut Command) -> Result<Output> {
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("failed to run {cmd:?}"))?;
    if !out.status.success() {
        bail!(
            "{:?} failed ({}): {}",
            cmd.get_program(),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(out)
}

/// Runs a command and returns trimmed stdout.
pub fn run_text(cmd: &mut Command) -> Result<String> {
    let out = run(cmd)?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// First line of stdout+stderr of a command (for `--version` style checks). None if it can't run or fails.
pub fn first_line(cmd: &mut Command) -> Option<String> {
    let out = cmd.stdin(Stdio::null()).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.contains("integer expression expected"))
        .map(str::to_string)
}

/// Polls `check` until it returns Some or the timeout passes.
pub fn wait_for<T>(timeout: Duration, interval: Duration, mut check: impl FnMut() -> Option<T>) -> Option<T> {
    let start = Instant::now();
    loop {
        if let Some(v) = check() {
            return Some(v);
        }
        if start.elapsed() >= timeout {
            return None;
        }
        std::thread::sleep(interval);
    }
}

/// Downloads a URL to a file using the system `curl` (present on macOS and most Linux).
pub fn download(url: &str, dest: &Path) -> Result<()> {
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = dest.with_extension("part");
    let status = Command::new("curl")
        .args(["-fL", "--progress-bar", "-o"])
        .arg(&tmp)
        .arg(url)
        .status()
        .context("failed to run curl")?;
    if !status.success() {
        bail!("download failed: {url}");
    }
    std::fs::rename(&tmp, dest)?;
    Ok(())
}

pub fn tcp_port_open(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_millis(300)).is_ok()
}
