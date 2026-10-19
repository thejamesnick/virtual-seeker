use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::util;

/// `adb` bound to one device serial (e.g. `emulator-5554` or a USB serial).
#[derive(Debug, Clone)]
pub struct Adb {
    pub bin: PathBuf,
    pub serial: String,
}

/// A device line from `adb devices`.
#[derive(Debug, Clone)]
pub struct DeviceEntry {
    pub serial: String,
    /// `device`, `offline`, `unauthorized`, ...
    pub state: String,
}

pub fn devices(adb_bin: &Path) -> Result<Vec<DeviceEntry>> {
    let text = util::run_text(Command::new(adb_bin).arg("devices"))?;
    Ok(text
        .lines()
        .skip(1)
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            Some(DeviceEntry { serial: parts.next()?.to_string(), state: parts.next()?.to_string() })
        })
        .collect())
}

impl Adb {
    pub fn new(bin: impl Into<PathBuf>, serial: impl Into<String>) -> Self {
        Self { bin: bin.into(), serial: serial.into() }
    }

    pub fn cmd(&self) -> Command {
        let mut c = Command::new(&self.bin);
        c.arg("-s").arg(&self.serial);
        c
    }

    pub fn run(&self, args: &[&str]) -> Result<String> {
        util::run_text(self.cmd().args(args))
    }

    pub fn shell(&self, script: &str) -> Result<String> {
        self.run(&["shell", script])
    }

    pub fn getprop(&self, name: &str) -> String {
        self.shell(&format!("getprop {name}")).unwrap_or_default()
    }

    pub fn boot_completed(&self) -> bool {
        self.getprop("sys.boot_completed") == "1"
    }

    /// Waits until the device is online and Android has finished booting.
    pub fn wait_boot(&self, timeout: Duration) -> Result<()> {
        util::wait_for(timeout, Duration::from_secs(2), || self.boot_completed().then_some(()))
            .with_context(|| format!("{} did not finish booting within {}s", self.serial, timeout.as_secs()))
    }

    pub fn is_emulator(&self) -> bool {
        self.getprop("ro.kernel.qemu") == "1" || self.serial.starts_with("emulator-")
    }

    /// For an emulator, the name of the AVD it runs (`adb emu avd name`).
    pub fn avd_name(&self) -> Option<String> {
        let text = self.run(&["emu", "avd", "name"]).ok()?;
        text.lines().next().map(|l| l.trim().to_string()).filter(|s| !s.is_empty())
    }

    pub fn install(&self, apk: &Path) -> Result<()> {
        let out = util::run_text(self.cmd().arg("install").arg("-r").arg(apk))?;
        if !out.contains("Success") {
            bail!("install of {} failed: {out}", apk.display());
        }
        Ok(())
    }

    pub fn has_package(&self, package: &str) -> bool {
        self.shell(&format!("pm path {package}")).map(|o| o.starts_with("package:")).unwrap_or(false)
    }

    /// Makes the device's `localhost:port` reach the host's `localhost:port`.
    pub fn reverse(&self, port: u16) -> Result<()> {
        self.run(&["reverse", &format!("tcp:{port}"), &format!("tcp:{port}")])?;
        Ok(())
    }

    /// `adb root`, then wait for the device to come back.
    pub fn root(&self) -> Result<()> {
        self.run(&["root"])?;
        std::thread::sleep(Duration::from_secs(2));
        self.run(&["wait-for-device"])?;
        Ok(())
    }

    /// `adb remount`. Returns true if Android says a reboot is needed first (first-time overlay setup).
    pub fn remount(&self) -> Result<bool> {
        let out = self.cmd().arg("remount").output().context("adb remount")?;
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        if text.to_lowercase().contains("reboot") {
            return Ok(true);
        }
        if !out.status.success() || !text.contains("Remount succeeded") {
            bail!("adb remount failed: {}", text.trim());
        }
        Ok(false)
    }

    pub fn reboot_and_wait(&self, timeout: Duration) -> Result<()> {
        self.run(&["reboot"])?;
        std::thread::sleep(Duration::from_secs(3));
        self.run(&["wait-for-device"])?;
        self.wait_boot(timeout)
    }
}
