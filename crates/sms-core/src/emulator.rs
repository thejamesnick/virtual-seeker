use std::fs::File;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::adb::{self, Adb};
use crate::{paths, profile, util, AndroidSdk, SmsPaths};

pub const BOOT_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug, Clone)]
pub struct StartOptions {
    pub avd: String,
    /// No emulator window (CI / desktop app streaming).
    pub headless: bool,
    /// `host` (Mac/PC GPU, default) or `swiftshader_indirect` (software).
    pub gpu: String,
    /// Factory-reset user data on this boot.
    pub wipe: bool,
}

impl Default for StartOptions {
    fn default() -> Self {
        Self { avd: profile::avd_name(), headless: false, gpu: "host".into(), wipe: false }
    }
}

pub fn avd_dir(avd: &str) -> Result<PathBuf> {
    Ok(paths::avd_home()?.join(format!("{avd}.avd")))
}

pub fn avd_exists(avd: &str) -> bool {
    avd_dir(avd).map(|d| d.join("config.ini").exists()).unwrap_or(false)
}

/// Finds the running emulator for this AVD and returns an `Adb` bound to it.
pub fn find_running(sdk: &AndroidSdk, avd: &str) -> Option<Adb> {
    let list = adb::devices(&sdk.adb()).ok()?;
    list.into_iter()
        .filter(|d| d.serial.starts_with("emulator-") && d.state == "device")
        .map(|d| Adb::new(sdk.adb(), d.serial))
        .find(|a| a.avd_name().as_deref() == Some(avd))
}

/// Starts the emulator in the background (it keeps running after `sms` exits) and waits for boot.
pub fn start(sdk: &AndroidSdk, sms: &SmsPaths, opts: &StartOptions) -> Result<Adb> {
    if !avd_exists(&opts.avd) {
        bail!("virtual device '{}' not found. Run `sms setup` first.", opts.avd);
    }
    if let Some(adb) = find_running(sdk, &opts.avd) {
        return Ok(adb);
    }
    sms.ensure()?;
    // Make sure the adb server is up before the emulator registers with it.
    let _ = Command::new(sdk.adb()).arg("start-server").output();

    let log_path = sms.logs().join("emulator.log");
    let log = File::create(&log_path).with_context(|| format!("creating {}", log_path.display()))?;
    let mut cmd = Command::new(sdk.emulator());
    // -writable-system is required every launch: it loads the Seeker identity + privileged Seed Vault.
    cmd.args(["-avd", &opts.avd, "-writable-system", "-gpu", &opts.gpu, "-no-boot-anim"]);
    if opts.headless {
        cmd.arg("-no-window");
    }
    if opts.wipe {
        cmd.arg("-wipe-data");
    }
    cmd.stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        // Own process group, so Ctrl-C in the terminal doesn't kill the emulator.
        .process_group(0);
    let child = cmd.spawn().context("starting the emulator")?;
    std::fs::write(sms.run().join("emulator.pid"), child.id().to_string())?;

    let adb = util::wait_for(Duration::from_secs(120), Duration::from_secs(2), || find_running(sdk, &opts.avd))
        .with_context(|| format!("emulator did not come online; see {}", log_path.display()))?;
    adb.wait_boot(BOOT_TIMEOUT)?;
    Ok(adb)
}

/// Stops a running emulator (`adb emu kill`) and waits for it to disappear.
pub fn stop(sdk: &AndroidSdk, sms: &SmsPaths, avd: &str) -> Result<bool> {
    let Some(adb) = find_running(sdk, avd) else {
        return Ok(false);
    };
    adb.run(&["emu", "kill"])?;
    // Shutdown includes writing the quick-boot snapshot (RAM image), which can take a while.
    util::wait_for(Duration::from_secs(180), Duration::from_secs(1), || find_running(sdk, avd).is_none().then_some(()))
        .context("emulator did not stop within 3 minutes")?;
    let _ = std::fs::remove_file(sms.run().join("emulator.pid"));
    Ok(true)
}

/// Writes the Seeker hardware profile into the AVD's `config.ini` (adds or replaces keys).
pub fn apply_hardware_profile(avd: &str) -> Result<()> {
    let cfg = avd_dir(avd)?.join("config.ini");
    let text = std::fs::read_to_string(&cfg).with_context(|| format!("reading {}", cfg.display()))?;
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    for (key, value) in profile::hardware_settings() {
        let new_line = format!("{key}={value}");
        match lines.iter_mut().find(|l| l.split_once('=').map(|(k, _)| k.trim()) == Some(key)) {
            Some(line) => *line = new_line,
            None => lines.push(new_line),
        }
    }
    std::fs::write(&cfg, lines.join("\n") + "\n")?;
    Ok(())
}

/// Reads one key from the AVD's `config.ini`.
pub fn avd_config_value(avd: &str, key: &str) -> Option<String> {
    let text = std::fs::read_to_string(avd_dir(avd).ok()?.join("config.ini")).ok()?;
    text.lines()
        .filter_map(|l| l.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim().to_string())
}
