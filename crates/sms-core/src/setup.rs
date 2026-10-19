//! `sms setup`: installs everything and builds the virtual Seeker. Safe to run again (each step
//! is skipped when already done).

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::emulator::{self, StartOptions};
use crate::{identity, java, profile, sdk, seedvault, theme, util, AndroidSdk, SmsPaths};

pub const REPOSITORY_XML: &str = "https://dl.google.com/android/repository/repository2-3.xml";
pub const REPOSITORY_BASE: &str = "https://dl.google.com/android/repository/";

#[derive(Debug, Clone)]
pub struct SetupOptions {
    pub avd: String,
    /// The user agreed to Google's Android SDK license terms (the CLI asks first).
    pub accept_licenses: bool,
    /// Delete and recreate the AVD (factory-fresh virtual Seeker).
    pub recreate_avd: bool,
    pub seed_vault_apk: Option<PathBuf>,
    /// Don't install (or build) the Seed Vault Simulator.
    pub skip_seed_vault: bool,
    /// Only install tools and create the AVD; don't boot it to apply identity/apps.
    pub skip_device: bool,
}

/// Where we look for the Seed Vault Simulator APK (no official prebuilt exists).
pub fn find_seed_vault_apk(sms: &SmsPaths) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(p) = std::env::var_os("SMS_SEED_VAULT_APK") {
        candidates.push(PathBuf::from(p));
    }
    candidates.push(sms.apks().join(profile::SEED_VAULT_APK_NAME));
    candidates.push(PathBuf::from("vendor/seed-vault-sdk/SeedVaultSimulator/build/outputs/apk/debug/SeedVaultSimulator-debug.apk"));
    candidates.into_iter().find(|p| p.is_file())
}

/// Where we look for the SMS Test Wallet APK.
pub fn find_sms_wallet_apk(sms: &SmsPaths) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(p) = std::env::var_os("SMS_WALLET_APK") {
        candidates.push(PathBuf::from(p));
    }
    candidates.push(sms.apks().join(profile::SMS_WALLET_APK_NAME));
    candidates.push(PathBuf::from("android/sms-wallet/app/build/outputs/apk/release/app-release.apk"));
    candidates.into_iter().find(|p| p.is_file())
}

/// Builds the SMS Test Wallet from `android/sms-wallet` (when run from the repo) and caches the APK.
fn build_sms_wallet_apk(sms: &SmsPaths, sdk: &AndroidSdk, java: &java::Java) -> Result<Option<PathBuf>> {
    let project = PathBuf::from("android/sms-wallet");
    if !project.join("gradlew").is_file() {
        return Ok(None);
    }
    std::fs::write(project.join("local.properties"), format!("sdk.dir={}\n", sdk.root.display()))?;
    let status = Command::new(project.join("gradlew"))
        .args(["--no-daemon", "-q", ":app:assembleRelease"])
        .current_dir(&project)
        .env("JAVA_HOME", &java.home)
        .env("ANDROID_HOME", &sdk.root)
        .stdin(Stdio::null())
        .status()
        .context("running gradle for the SMS Test Wallet")?;
    if !status.success() {
        bail!("building the SMS Test Wallet failed (see Gradle output above)");
    }
    let built = project.join("app/build/outputs/apk/release/app-release.apk");
    let cached = sms.apks().join(profile::SMS_WALLET_APK_NAME);
    std::fs::copy(&built, &cached)?;
    Ok(Some(cached))
}

pub fn run(sms: &SmsPaths, opts: &SetupOptions, log: &mut dyn FnMut(&str)) -> Result<()> {
    sms.ensure()?;

    log("Checking Java (needed by Google's sdkmanager/avdmanager)");
    let java = java::find().context("Java not found. Install a JDK 17+ (e.g. Temurin) or Android Studio, then run `sms setup` again")?;

    let sdk = AndroidSdk::find_or_default(sms);
    std::fs::create_dir_all(&sdk.root)?;
    log(&format!("Android SDK: {}", sdk.root.display()));

    if !util::is_executable(&sdk.sdkmanager()) {
        log("Installing Android SDK command-line tools");
        install_cmdline_tools(&sdk, sms)?;
    }

    if !sdk.licenses_accepted() {
        if !opts.accept_licenses {
            bail!("Google's Android SDK license must be accepted first");
        }
        log("Recording Android SDK license acceptance");
        accept_licenses(&sdk, &java)?;
    }

    let missing: Vec<String> = profile::required_packages().into_iter().filter(|p| !sdk.has_package(p)).collect();
    if missing.is_empty() {
        log("Android packages already installed");
    } else {
        log(&format!("Installing {} (the system image is ~2 GB)", missing.join(", ")));
        let status = sdk
            .java_tool(&sdk.sdkmanager(), &java)
            .arg("--install")
            .args(&missing)
            .stdin(Stdio::null())
            .status()
            .context("running sdkmanager")?;
        if !status.success() {
            bail!("sdkmanager --install failed");
        }
    }

    if opts.recreate_avd || !emulator::avd_exists(&opts.avd) {
        log(&format!("Creating virtual device '{}'", opts.avd));
        if emulator::find_running(&sdk, &opts.avd).is_some() {
            bail!("'{}' is running; stop it with `sms stop` before recreating it", opts.avd);
        }
        create_avd(&sdk, &java, &opts.avd)?;
    }
    log("Applying the Seeker hardware profile (1200x2670, density 480, host GPU)");
    emulator::apply_hardware_profile(&opts.avd)?;
    log("Installing the Seeker-style phone frame (skin)");
    theme::install_skin(sms, &opts.avd)?;

    let seed_vault_apk = if opts.skip_seed_vault {
        None
    } else {
        Some(match opts.seed_vault_apk.clone().or_else(|| find_seed_vault_apk(sms)) {
            Some(apk) => cache_seed_vault_apk(sms, &apk)?,
            None => {
                log("Building the Seed Vault Simulator from solana-mobile/seed-vault-sdk (first time: ~5 min, downloads ~1 GB)");
                build_seed_vault_apk(sms, &sdk, &java)?
            }
        })
    };

    let sms_wallet_apk = match find_sms_wallet_apk(sms) {
        Some(apk) => {
            let cached = sms.apks().join(profile::SMS_WALLET_APK_NAME);
            if apk != cached {
                std::fs::copy(&apk, &cached)?;
            }
            Some(cached)
        }
        None => {
            log("Building the SMS Test Wallet (android/sms-wallet)");
            build_sms_wallet_apk(sms, &sdk, &java)?
        }
    };

    if opts.skip_device {
        return Ok(());
    }

    let was_running = emulator::find_running(&sdk, &opts.avd).is_some();
    if !was_running {
        log("Booting the virtual Seeker in the background");
    }
    let adb = emulator::start(&sdk, sms, &StartOptions { avd: opts.avd.clone(), headless: true, ..Default::default() })?;

    log("Applying the Seeker identity (model/brand/manufacturer)");
    if identity::apply_seeker_identity(&adb)? {
        log("  identity changed to Seeker / solanamobile / Solana Mobile Inc.");
    } else {
        log("  already a Seeker");
    }

    log("Applying the Seeker display (camera-hole cutout, rounded corners, status bar)");
    if !identity::install_display_overlay(&adb)? {
        log("  already applied");
    }

    match seed_vault_apk {
        Some(apk) if !identity::is_privileged(&adb, profile::SEED_VAULT_PACKAGE) => {
            log("Installing Seed Vault Simulator as a privileged system app");
            identity::install_privileged_app(&adb, &apk, "SeedVaultSimulator")?;
        }
        Some(_) => log("Seed Vault Simulator already installed (privileged)"),
        None => log("Skipping Seed Vault Simulator (--no-seed-vault)"),
    }

    if adb.has_package(profile::SEED_VAULT_PACKAGE) {
        if seedvault::import_test_seed(&adb)? {
            log(&format!(
                "Loaded the public test seed '{}' into Seed Vault (PIN {}). Test funds only!",
                seedvault::TEST_SEED_NAME,
                seedvault::TEST_SEED_PIN
            ));
        } else {
            log("Seed Vault already has seeds; left them unchanged");
        }
    }

    match sms_wallet_apk {
        Some(apk) => {
            log("Installing the SMS Test Wallet (MWA wallet backed by Seed Vault)");
            adb.install(&apk)?;
            adb.shell(&format!("pm grant {} com.solanamobile.seedvault.ACCESS_SEED_VAULT", profile::SMS_WALLET_PACKAGE))?;
            // One MWA wallet, like a real Seeker: no "choose a wallet" dialog.
            if adb.has_package(profile::FAKEWALLET_PACKAGE) {
                adb.shell(&format!("pm disable-user --user 0 {}", profile::FAKEWALLET_PACKAGE))?;
                log("  disabled the generic fakewallet so apps use the SMS Test Wallet");
            }
        }
        None => {
            log("SMS Test Wallet not available (run setup from the repo to build it); using the generic fakewallet");
            let wallet_apk = sms.apks().join("fakewallet-v1-release.apk");
            if !wallet_apk.is_file() {
                log("Downloading the MWA test wallet (fakewallet)");
                util::download(profile::FAKEWALLET_URL, &wallet_apk)?;
            }
            if adb.has_package(profile::FAKEWALLET_PACKAGE) {
                log("MWA test wallet already installed");
            } else {
                log("Installing the MWA test wallet");
                adb.install(&wallet_apk)?;
            }
        }
    }

    log("Applying the Seeker theme (light mode, Seeker wallpaper, colors from the wallpaper)");
    if !theme::apply_device_theme(&adb)? {
        log("  wallpaper skipped (needs the SMS Test Wallet app)");
    }

    if !was_running {
        log("Shutting the virtual Seeker down");
        emulator::stop(&sdk, sms, &opts.avd)?;
    }
    Ok(())
}

/// seed-vault-sdk commit we build and tested (2026-10-14). Pinned so builds are reproducible.
pub const SEED_VAULT_SDK_REPO: &str = "https://github.com/solana-mobile/seed-vault-sdk.git";
pub const SEED_VAULT_SDK_COMMIT: &str = "efb0e6f37779fc46d3ceb7895462e865d4303425";

/// Copies an APK into `~/.sms/apks/SeedVaultSimulator.apk` so later runs find it from any folder.
fn cache_seed_vault_apk(sms: &SmsPaths, apk: &std::path::Path) -> Result<PathBuf> {
    let cached = sms.apks().join(profile::SEED_VAULT_APK_NAME);
    if apk != cached {
        std::fs::copy(apk, &cached).with_context(|| format!("copying {}", apk.display()))?;
    }
    Ok(cached)
}

/// Fetches seed-vault-sdk at the pinned commit into `~/.sms/src` and builds the simulator with Gradle.
fn build_seed_vault_apk(sms: &SmsPaths, sdk: &AndroidSdk, java: &java::Java) -> Result<PathBuf> {
    util::which("git").context("git is needed to fetch seed-vault-sdk")?;
    let src = sms.home.join("src/seed-vault-sdk");
    if !src.join("gradlew").exists() {
        std::fs::create_dir_all(&src)?;
        let git = |args: &[&str]| util::run(Command::new("git").arg("-C").arg(&src).args(args));
        git(&["init", "-q"])?;
        git(&["fetch", "-q", "--depth", "1", SEED_VAULT_SDK_REPO, SEED_VAULT_SDK_COMMIT])?;
        git(&["checkout", "-q", "FETCH_HEAD"])?;
    }
    std::fs::write(src.join("local.properties"), format!("sdk.dir={}\n", sdk.root.display()))?;
    let status = Command::new(src.join("gradlew"))
        .args(["--no-daemon", "-q", ":SeedVaultSimulator:assembleDebug"])
        .current_dir(&src)
        .env("JAVA_HOME", &java.home)
        .env("ANDROID_HOME", &sdk.root)
        .stdin(Stdio::null())
        .status()
        .context("running gradle")?;
    if !status.success() {
        bail!("building the Seed Vault Simulator failed (see Gradle output above)");
    }
    let built = src.join("SeedVaultSimulator/build/outputs/apk/debug/SeedVaultSimulator-debug.apk");
    cache_seed_vault_apk(sms, &built)
}

/// Downloads Google's latest command-line tools for this OS into `<sdk>/cmdline-tools/latest`.
fn install_cmdline_tools(sdk: &AndroidSdk, sms: &SmsPaths) -> Result<()> {
    let xml = util::run_text(Command::new("curl").args(["-fsSL", REPOSITORY_XML])).context("fetching Android repository index")?;
    let prefix = format!("commandlinetools-{}-", sdk::cmdline_tools_os());
    let build = latest_build(&xml, &prefix).context("no command-line tools found in the Android repository index")?;
    let file = format!("{prefix}{build}_latest.zip");

    let work = sms.home.join("downloads");
    let zip = work.join(&file);
    if !zip.is_file() {
        util::download(&format!("{REPOSITORY_BASE}{file}"), &zip)?;
    }
    let unpack = work.join("cmdline-tools-unpack");
    let _ = std::fs::remove_dir_all(&unpack);
    std::fs::create_dir_all(&unpack)?;
    util::run(Command::new("unzip").arg("-q").arg(&zip).arg("-d").arg(&unpack))?;

    let dest = sdk.root.join("cmdline-tools/latest");
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(dest.parent().unwrap())?;
    std::fs::rename(unpack.join("cmdline-tools"), &dest)?;
    let _ = std::fs::remove_dir_all(&unpack);
    Ok(())
}

/// Highest build number in names like `commandlinetools-mac-15641748_latest.zip`.
fn latest_build(xml: &str, prefix: &str) -> Option<u64> {
    xml.match_indices(prefix)
        .filter_map(|(i, _)| {
            let rest = &xml[i + prefix.len()..];
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            rest[digits.len()..].starts_with("_latest.zip").then(|| digits.parse().ok()).flatten()
        })
        .max()
}

/// Feeds "y" to `sdkmanager --licenses`. Only called after the user agreed.
fn accept_licenses(sdk: &AndroidSdk, java: &java::Java) -> Result<()> {
    let mut child = sdk
        .java_tool(&sdk.sdkmanager(), java)
        .arg("--licenses")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("running sdkmanager --licenses")?;
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all("y\n".repeat(30).as_bytes());
    }
    child.wait()?;
    if !sdk.licenses_accepted() {
        bail!("license acceptance was not recorded");
    }
    Ok(())
}

fn create_avd(sdk: &AndroidSdk, java: &java::Java, avd: &str) -> Result<()> {
    let mut child = sdk
        .java_tool(&sdk.avdmanager(), java)
        .args(["create", "avd", "-n", avd, "-k", &profile::system_image(), "--force"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("running avdmanager")?;
    // "Do you wish to create a custom hardware profile? [no]"
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(b"no\n");
    }
    let out = child.wait_with_output()?;
    if !out.status.success() || !emulator::avd_exists(avd) {
        bail!("avdmanager create avd failed: {}", String::from_utf8_lossy(&out.stderr).trim());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_highest_cmdline_tools_build() {
        let xml = "<url>commandlinetools-mac-9123335_latest.zip</url>
                   <url>commandlinetools-mac-15641748_latest.zip</url>
                   <url>commandlinetools-linux-99999999_latest.zip</url>
                   <url>commandlinetools-mac-123.zip</url>";
        assert_eq!(latest_build(xml, "commandlinetools-mac-"), Some(15641748));
    }
}
