//! `sms doctor`: checks everything the virtual Seeker needs and says how to fix what's missing.

use std::process::Command;

use serde_json::{Value, json};

use crate::{emulator, identity, java, localnet, profile, seedvault, setup, theme, util, AndroidSdk, SmsPaths};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    /// Works, but something is missing or not ideal.
    Warn,
    /// `sms start` can't work until this is fixed.
    Fail,
}

#[derive(Debug, Clone)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
    pub fix: Option<String>,
}

impl Check {
    fn ok(name: &'static str, detail: impl Into<String>) -> Self {
        Self { name, status: Status::Ok, detail: detail.into(), fix: None }
    }
    fn warn(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Self { name, status: Status::Warn, detail: detail.into(), fix: Some(fix.into()) }
    }
    fn fail(name: &'static str, detail: impl Into<String>, fix: impl Into<String>) -> Self {
        Self { name, status: Status::Fail, detail: detail.into(), fix: Some(fix.into()) }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "status": match self.status { Status::Ok => "ok", Status::Warn => "warn", Status::Fail => "fail" },
            "detail": self.detail,
            "fix": self.fix,
        })
    }
}

pub fn run(sms: &SmsPaths, avd: &str) -> Vec<Check> {
    let mut checks = Vec::new();
    let sdk = AndroidSdk::find(sms);

    checks.push(Check::ok("Host", format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)));

    // Java is only needed by sdkmanager/avdmanager during `sms setup`.
    checks.push(match java::find() {
        Some(j) => Check::ok("Java", format!("{} ({})", j.version, j.home.display())),
        None => Check::warn("Java", "not found (only needed by `sms setup`)", "Install a JDK 17+ (e.g. Temurin) or Android Studio"),
    });

    let Some(sdk) = sdk else {
        checks.push(Check::fail("Android SDK", "not found", "Run `sms setup`"));
        return checks;
    };
    checks.push(Check::ok("Android SDK", sdk.root.display().to_string()));

    checks.push(match sdk.adb_version() {
        Some(v) => Check::ok("adb", v),
        None => Check::fail("adb", "missing", "Run `sms setup`"),
    });
    checks.push(match sdk.emulator_version() {
        Some(v) => Check::ok("Emulator", v),
        None => Check::fail("Emulator", "missing", "Run `sms setup`"),
    });
    checks.push(if util::is_executable(&sdk.sdkmanager()) {
        Check::ok("SDK command-line tools", sdk.sdkmanager().display().to_string())
    } else {
        Check::warn("SDK command-line tools", "missing (needed by `sms setup`)", "Run `sms setup`")
    });

    if util::is_executable(&sdk.emulator()) {
        checks.push(match sdk.accel_check() {
            Ok(text) if !text.is_empty() => Check::ok("Hardware acceleration", text),
            _ => Check::warn("Hardware acceleration", "not confirmed", "Enable virtualization (Hypervisor.Framework on macOS, KVM on Linux)"),
        });
    }

    let image = profile::system_image();
    checks.push(if sdk.has_package(&image) {
        Check::ok("Android 16 image", image)
    } else {
        Check::fail("Android 16 image", format!("{image} not installed"), "Run `sms setup`")
    });

    checks.push(if emulator::avd_exists(avd) {
        let width = emulator::avd_config_value(avd, "hw.lcd.width").unwrap_or_default();
        let height = emulator::avd_config_value(avd, "hw.lcd.height").unwrap_or_default();
        let gpu = emulator::avd_config_value(avd, "hw.gpu.mode").unwrap_or_default();
        let skin = if emulator::avd_config_value(avd, "skin.name").as_deref() == Some("seeker") { "Seeker skin" } else { "no skin" };
        if width == "1200" && height == "2670" {
            Check::ok("Virtual Seeker", format!("AVD '{avd}' ({width}x{height}, gpu={gpu}, {skin})"))
        } else {
            Check::warn("Virtual Seeker", format!("AVD '{avd}' exists but has no Seeker profile ({width}x{height})"), "Run `sms setup`")
        }
    } else {
        Check::fail("Virtual Seeker", format!("AVD '{avd}' not created"), "Run `sms setup`")
    });

    checks.push(match setup::find_seed_vault_apk(sms) {
        Some(p) => Check::ok("Seed Vault Simulator APK", p.display().to_string()),
        None => Check::warn(
            "Seed Vault Simulator APK",
            "not found",
            format!("Build it from solana-mobile/seed-vault-sdk and copy it to {}", sms.apks().join(profile::SEED_VAULT_APK_NAME).display()),
        ),
    });

    checks.push(match localnet::find_surfpool() {
        Some(p) => Check::ok("Surfpool (localnet)", util::first_line(Command::new(&p).arg("--version")).unwrap_or_else(|| p.display().to_string())),
        None => Check::warn("Surfpool (localnet)", "not installed (needed for `sms start --localnet`)", "Install from https://github.com/solana-foundation/surfpool/releases"),
    });

    let solana = util::which("solana").or_else(|| {
        let p = crate::paths::user_home().ok()?.join(".local/share/solana/install/active_release/bin/solana");
        util::is_executable(&p).then_some(p)
    });
    checks.push(match solana {
        None => Check::warn("Solana CLI", "not installed (optional)", "sh -c \"$(curl -sSfL https://release.anza.xyz/stable/install)\""),
        Some(p) => match util::first_line(Command::new(&p).arg("--version")) {
            Some(v) => Check::ok("Solana CLI", v),
            None => Check::warn("Solana CLI", "installed but fails to run", "On macOS without Homebrew it needs libusb; see docs/05-dev-setup.md"),
        },
    });

    // Live state of the virtual Seeker, if it's running.
    match emulator::find_running(&sdk, avd) {
        Some(adb) => {
            let identity = identity::has_seeker_identity(&adb);
            checks.push(if identity {
                Check::ok("Running", format!("{} · Android {} · model {}", adb.serial, adb.getprop("ro.build.version.release"), adb.getprop("ro.product.model")))
            } else {
                Check::warn("Running", format!("{} reports model '{}'", adb.serial, adb.getprop("ro.product.model")), "Run `sms setup` to apply the Seeker identity")
            });
            checks.push(if identity::is_privileged(&adb, profile::SEED_VAULT_PACKAGE) {
                Check::ok("Seed Vault on device", "privileged system app")
            } else {
                Check::warn("Seed Vault on device", "not installed as privileged app", "Run `sms setup`")
            });
            checks.push(if identity::has_display_overlay(&adb) {
                Check::ok("Display", "camera-hole cutout + rounded corners")
            } else {
                Check::warn("Display", "no cutout/corner overlay (status bar not aligned)", "Run `sms setup`")
            });
            checks.push(if theme::is_themed(&adb) {
                Check::ok("Theme", "Seeker style (dark, Solana purple)")
            } else {
                Check::warn("Theme", "stock Android look", "Run `sms setup`")
            });
            checks.push(if adb.has_package(profile::SMS_WALLET_PACKAGE) {
                Check::ok("Wallet on device", "SMS Test Wallet (MWA → Seed Vault)")
            } else if adb.has_package(profile::FAKEWALLET_PACKAGE) {
                Check::warn("Wallet on device", "generic fakewallet (keys not in Seed Vault)", "Run `sms setup` from the repo to install the SMS Test Wallet")
            } else {
                Check::warn("Wallet on device", "no MWA wallet installed", "Run `sms setup`")
            });
            if adb.has_package(profile::SEED_VAULT_PACKAGE) {
                checks.push(if seedvault::has_test_seed(&adb) {
                    Check::ok("Test seed", format!("'{}' (PIN {}, public test phrase)", seedvault::TEST_SEED_NAME, seedvault::TEST_SEED_PIN))
                } else if seedvault::has_seeds(&adb) {
                    Check::ok("Test seed", "Seed Vault has your own seeds")
                } else {
                    Check::warn("Test seed", "Seed Vault has no seeds", "Run `sms setup`")
                });
            }
        }
        None => checks.push(Check::ok("Running", "not running (start with `sms start`)")),
    }

    checks.push(disk_check(sms));
    checks
}

fn disk_check(sms: &SmsPaths) -> Check {
    let dir = if sms.home.exists() { sms.home.clone() } else { sms.home.parent().map(|p| p.to_path_buf()).unwrap_or_default() };
    let free_gb = util::run_text(Command::new("df").arg("-k").arg(&dir))
        .ok()
        .and_then(|t| t.lines().nth(1).and_then(|l| l.split_whitespace().nth(3)).and_then(|kb| kb.parse::<u64>().ok()))
        .map(|kb| kb / 1024 / 1024);
    match free_gb {
        Some(gb) if gb >= 10 => Check::ok("Disk space", format!("{gb} GB free")),
        Some(gb) => Check::warn("Disk space", format!("{gb} GB free"), "Free up at least 10 GB (the Android image is ~2 GB, the device ~6 GB)"),
        None => Check::warn("Disk space", "unknown", "Check free space manually"),
    }
}
