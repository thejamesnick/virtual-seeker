//! `sms`: the virtual Seeker from the command line.

use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use serde_json::json;

use sms_core::adb::{self, Adb};
use sms_core::doctor::{self, Status};
use sms_core::emulator::{self, StartOptions};
use sms_core::setup::{self, SetupOptions};
use sms_core::{identity, localnet, profile, AndroidSdk, SmsPaths};

/// Logcat tags that show the Mobile Wallet Adapter and Seed Vault flow (see docs/07).
const SOLANA_LOG_TAGS: &[&str] = &[
    "SmsWallet",
    "MobileWalletAdapterSessionCommon",
    "MobileWalletAdapterWebSocket",
    "JsonRpc20Client",
    "JsonRpc20Server",
    "LocalWebSocketServer",
    "LocalAssociationScenario",
    "LocalScenario",
    "AuthRepositoryImpl",
    "AuthorizeCommonViewModel",
    "MainViewModel",
    "MainActivity",
    "MemoTransactionUseCase",
];

#[derive(Parser)]
#[command(name = "sms", version, about = "Virtual Seeker: a Solana Seeker-like Android emulator for developers")]
#[command(after_help = "Community project, not affiliated with or endorsed by Solana Mobile.")]
struct Cli {
    /// Virtual device name [env: SMS_AVD] [default: Seeker]
    #[arg(long, global = true)]
    avd: Option<String>,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check that everything the virtual Seeker needs is installed
    Doctor {
        /// Machine-readable output
        #[arg(long)]
        json: bool,
    },
    /// Install the Android tools and build the virtual Seeker (safe to re-run)
    Setup {
        /// Accept Google's Android SDK license without asking
        #[arg(long)]
        accept_licenses: bool,
        /// Delete and recreate the virtual device
        #[arg(long)]
        recreate: bool,
        /// Seed Vault Simulator APK to install as a system app (default: find it, or build it from source)
        #[arg(long, value_name = "APK")]
        seed_vault_apk: Option<PathBuf>,
        /// Don't install the Seed Vault Simulator
        #[arg(long)]
        no_seed_vault: bool,
        /// Only install tools and create the device; don't boot it
        #[arg(long)]
        no_device: bool,
    },
    /// Start the virtual Seeker
    Start {
        /// No emulator window
        #[arg(long)]
        headless: bool,
        /// Also start a local Solana network (Surfpool on :8899) and connect the device to it
        #[arg(long)]
        localnet: bool,
        /// GPU mode: host (fast, default) or swiftshader_indirect (software)
        #[arg(long, default_value = "host")]
        gpu: String,
        /// Factory-reset the device's user data on this boot
        #[arg(long)]
        wipe: bool,
    },
    /// Stop the virtual Seeker (and the local network `sms start` launched)
    Stop {
        /// Keep the local Solana network running
        #[arg(long)]
        keep_localnet: bool,
    },
    /// Install one or more APKs on the virtual Seeker (or --serial device)
    Install {
        #[arg(required = true, value_name = "APK")]
        apks: Vec<PathBuf>,
        /// Target device (default: the running virtual Seeker)
        #[arg(long)]
        serial: Option<String>,
    },
    /// Stream device logs (by default: wallet / Seed Vault / crash logs only)
    Logs {
        /// Only logs from this app package
        #[arg(long, value_name = "PACKAGE")]
        app: Option<String>,
        /// Everything (no filter)
        #[arg(long)]
        all: bool,
        /// Clear the log buffer first
        #[arg(long)]
        clear: bool,
        /// Target device (default: the running virtual Seeker)
        #[arg(long)]
        serial: Option<String>,
    },
    /// List connected devices (virtual and real Seekers)
    Devices {
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("✗ {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    let sms = SmsPaths::new()?;
    let avd = cli.avd.unwrap_or_else(profile::avd_name);

    match cli.command {
        Cmd::Doctor { json } => doctor_cmd(&sms, &avd, json),
        Cmd::Setup { accept_licenses, recreate, seed_vault_apk, no_seed_vault, no_device } => {
            setup_cmd(&sms, &avd, accept_licenses, recreate, seed_vault_apk, no_seed_vault, no_device)
        }
        Cmd::Start { headless, localnet, gpu, wipe } => start_cmd(&sms, &avd, headless, localnet, gpu, wipe),
        Cmd::Stop { keep_localnet } => stop_cmd(&sms, &avd, keep_localnet),
        Cmd::Install { apks, serial } => install_cmd(&sms, &avd, apks, serial),
        Cmd::Logs { app, all, clear, serial } => logs_cmd(&sms, &avd, app, all, clear, serial),
        Cmd::Devices { json } => devices_cmd(&sms, json),
    }
}

fn require_sdk(sms: &SmsPaths) -> Result<AndroidSdk> {
    AndroidSdk::find(sms).context("Android SDK not found. Run `sms setup` first")
}

fn doctor_cmd(sms: &SmsPaths, avd: &str, json: bool) -> Result<ExitCode> {
    let checks = doctor::run(sms, avd);
    let failed = checks.iter().any(|c| c.status == Status::Fail);
    if json {
        let list: Vec<_> = checks.iter().map(|c| c.to_json()).collect();
        println!("{}", serde_json::to_string_pretty(&json!({ "ok": !failed, "checks": list }))?);
    } else {
        println!("sms doctor\n");
        for c in &checks {
            let icon = match c.status {
                Status::Ok => "✓",
                Status::Warn => "!",
                Status::Fail => "✗",
            };
            println!("  {icon} {:<26} {}", c.name, c.detail);
            if let Some(fix) = &c.fix {
                println!("    {:<26} → {fix}", "");
            }
        }
        println!();
        println!("{}", if failed { "Some checks failed. Fix them, then run `sms doctor` again." } else { "Ready." });
    }
    Ok(if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS })
}

fn setup_cmd(
    sms: &SmsPaths,
    avd: &str,
    mut accept_licenses: bool,
    recreate: bool,
    seed_vault_apk: Option<PathBuf>,
    skip_seed_vault: bool,
    no_device: bool,
) -> Result<ExitCode> {
    let sdk = AndroidSdk::find_or_default(sms);
    if !sdk.licenses_accepted() && !accept_licenses {
        accept_licenses = ask_license()?;
        if !accept_licenses {
            bail!("setup cancelled: the Android SDK license was not accepted");
        }
    }
    let started = Instant::now();
    let opts = SetupOptions { avd: avd.to_string(), accept_licenses, recreate_avd: recreate, seed_vault_apk, skip_seed_vault, skip_device: no_device };
    setup::run(sms, &opts, &mut |msg| println!("→ {msg}"))?;
    println!("\n✓ Virtual Seeker ready in {}s. Start it with `sms start`.", started.elapsed().as_secs());
    Ok(ExitCode::SUCCESS)
}

fn ask_license() -> Result<bool> {
    println!("sms setup downloads Android SDK packages from Google.");
    println!("To use them you must accept the Android SDK License Agreement:");
    println!("  https://developer.android.com/studio/terms\n");
    if !std::io::stdin().is_terminal() {
        bail!("no terminal to ask for license acceptance; re-run with --accept-licenses if you accept it");
    }
    print!("Do you accept the Android SDK license? [y/N] ");
    std::io::stdout().flush()?;
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    Ok(matches!(answer.trim().to_lowercase().as_str(), "y" | "yes"))
}

fn start_cmd(sms: &SmsPaths, avd: &str, headless: bool, localnet: bool, gpu: String, wipe: bool) -> Result<ExitCode> {
    let sdk = require_sdk(sms)?;
    let started = Instant::now();
    let already = emulator::find_running(&sdk, avd).is_some();
    if !already {
        println!("→ Starting virtual Seeker '{avd}'{}", if headless { " (headless)" } else { "" });
    }
    let adb = emulator::start(&sdk, sms, &StartOptions { avd: avd.to_string(), headless, gpu, wipe })?;

    if localnet {
        if localnet::start(sms)? {
            println!("→ Started local Solana network (Surfpool) at {}", localnet::rpc_url());
        } else {
            println!("→ Using the local Solana network already running at {}", localnet::rpc_url());
        }
    }
    // Device localhost:8899/8900 → host, for wallets that use "localnet" (e.g. fakewallet).
    for port in [localnet::RPC_PORT, localnet::WS_PORT] {
        adb.reverse(port)?;
    }

    println!();
    println!("✓ Virtual Seeker {}", if already { "already running" } else { "running" });
    println!("  device     {}", adb.serial);
    println!("  model      {} ({})", adb.getprop("ro.product.model"), adb.getprop("ro.product.manufacturer"));
    println!("  android    {} (API {})", adb.getprop("ro.build.version.release"), adb.getprop("ro.build.version.sdk"));
    println!("  screen     {}", adb.shell("wm size").unwrap_or_default().replace("Physical size: ", ""));
    println!("  seed vault {}", if identity::is_privileged(&adb, profile::SEED_VAULT_PACKAGE) { "installed (privileged)" } else { "not installed" });
    println!("  localnet   {}", if localnet::is_running() { format!("{} (device: http://localhost:{})", localnet::rpc_url(), localnet::RPC_PORT) } else { "off (use --localnet)".into() });
    if !identity::has_seeker_identity(&adb) {
        println!("\n! The device doesn't report itself as a Seeker yet. Run `sms setup`.");
    }
    if !already {
        println!("\nReady in {}s.", started.elapsed().as_secs());
    }
    Ok(ExitCode::SUCCESS)
}

fn stop_cmd(sms: &SmsPaths, avd: &str, keep_localnet: bool) -> Result<ExitCode> {
    let sdk = require_sdk(sms)?;
    if emulator::find_running(&sdk, avd).is_some() {
        println!("→ Stopping '{avd}' (saving its quick-boot snapshot can take up to a minute)");
    }
    if emulator::stop(&sdk, sms, avd)? {
        println!("✓ Stopped virtual Seeker '{avd}'");
    } else {
        println!("Virtual Seeker '{avd}' was not running");
    }
    if !keep_localnet && localnet::stop(sms)? {
        println!("✓ Stopped local Solana network");
    }
    Ok(ExitCode::SUCCESS)
}

/// The device to talk to: --serial, else the running virtual Seeker, else the only connected device.
fn target_device(sms: &SmsPaths, avd: &str, serial: Option<String>) -> Result<Adb> {
    let sdk = require_sdk(sms)?;
    if let Some(serial) = serial {
        return Ok(Adb::new(sdk.adb(), serial));
    }
    if let Some(adb) = emulator::find_running(&sdk, avd) {
        return Ok(adb);
    }
    let online: Vec<_> = adb::devices(&sdk.adb())?.into_iter().filter(|d| d.state == "device").collect();
    match online.as_slice() {
        [one] => Ok(Adb::new(sdk.adb(), one.serial.clone())),
        [] => bail!("no device running. Start the virtual Seeker with `sms start`"),
        _ => bail!("several devices connected; pick one with --serial (see `sms devices`)"),
    }
}

fn install_cmd(sms: &SmsPaths, avd: &str, apks: Vec<PathBuf>, serial: Option<String>) -> Result<ExitCode> {
    let adb = target_device(sms, avd, serial)?;
    for apk in &apks {
        if !apk.is_file() {
            bail!("file not found: {}", apk.display());
        }
        print!("→ Installing {} on {} ... ", apk.display(), adb.serial);
        std::io::stdout().flush()?;
        adb.install(apk)?;
        println!("done");
    }
    Ok(ExitCode::SUCCESS)
}

fn logs_cmd(sms: &SmsPaths, avd: &str, app: Option<String>, all: bool, clear: bool, serial: Option<String>) -> Result<ExitCode> {
    let adb = target_device(sms, avd, serial)?;
    if clear {
        adb.run(&["logcat", "-c"])?;
    }
    let mut cmd: Command = adb.cmd();
    cmd.args(["logcat", "-v", "time"]);
    if let Some(pkg) = &app {
        let pid = adb.shell(&format!("pidof {pkg}")).unwrap_or_default();
        if pid.is_empty() {
            bail!("{pkg} is not running");
        }
        cmd.arg(format!("--pid={}", pid.split_whitespace().next().unwrap_or_default()));
    } else if !all {
        cmd.args(SOLANA_LOG_TAGS.iter().map(|t| format!("{t}:V")));
        cmd.arg("AndroidRuntime:E"); // crashes
        cmd.arg("*:S");
    }
    eprintln!("Streaming logs from {} (Ctrl-C to stop)…", adb.serial);
    let status = cmd.status().context("running adb logcat")?;
    Ok(if status.success() { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}

fn devices_cmd(sms: &SmsPaths, json: bool) -> Result<ExitCode> {
    let sdk = require_sdk(sms)?;
    let mut rows = Vec::new();
    for d in adb::devices(&sdk.adb())? {
        let adb = Adb::new(sdk.adb(), d.serial.clone());
        let online = d.state == "device";
        let model = if online { adb.getprop("ro.product.model") } else { String::new() };
        let kind = match (online, adb.is_emulator(), model == profile::MODEL) {
            (false, _, _) => "offline",
            (true, true, true) => "virtual Seeker",
            (true, true, false) => "emulator",
            (true, false, true) => "real Seeker",
            (true, false, false) => "Android device",
        };
        rows.push(json!({
            "serial": d.serial,
            "state": d.state,
            "kind": kind,
            "model": model,
            "android": if online { adb.getprop("ro.build.version.release") } else { String::new() },
            "avd": if online && adb.is_emulator() { adb.avd_name() } else { None },
        }));
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
    } else if rows.is_empty() {
        println!("No devices. Start the virtual Seeker with `sms start`, or plug in a Seeker with USB debugging on.");
    } else {
        println!("{:<16} {:<16} {:<10} {:<8} {}", "SERIAL", "KIND", "MODEL", "ANDROID", "STATE");
        for r in &rows {
            println!(
                "{:<16} {:<16} {:<10} {:<8} {}",
                r["serial"].as_str().unwrap_or_default(),
                r["kind"].as_str().unwrap_or_default(),
                r["model"].as_str().unwrap_or_default(),
                r["android"].as_str().unwrap_or_default(),
                r["state"].as_str().unwrap_or_default()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}
