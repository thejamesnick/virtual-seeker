use std::path::PathBuf;

use anyhow::{Context, Result};

/// Where `sms` keeps its own files: `~/.sms` (override with `SMS_HOME`).
///
/// ```text
/// ~/.sms/
///   apks/   downloaded or user-provided APKs (fakewallet, Seed Vault Simulator)
///   logs/   emulator.log, surfpool.log
///   run/    pid files for processes `sms start` launched
/// ```
#[derive(Debug, Clone)]
pub struct SmsPaths {
    pub home: PathBuf,
}

impl SmsPaths {
    pub fn new() -> Result<Self> {
        let home = match std::env::var_os("SMS_HOME") {
            Some(p) => PathBuf::from(p),
            None => user_home()?.join(".sms"),
        };
        Ok(Self { home })
    }

    pub fn apks(&self) -> PathBuf {
        self.home.join("apks")
    }
    pub fn logs(&self) -> PathBuf {
        self.home.join("logs")
    }
    pub fn run(&self) -> PathBuf {
        self.home.join("run")
    }

    pub fn ensure(&self) -> Result<()> {
        for dir in [self.apks(), self.logs(), self.run()] {
            std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        Ok(())
    }
}

pub fn user_home() -> Result<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from).context("HOME is not set")
}

/// `~/.android/avd`, where the emulator keeps virtual devices (override with `ANDROID_AVD_HOME`).
pub fn avd_home() -> Result<PathBuf> {
    if let Some(p) = std::env::var_os("ANDROID_AVD_HOME") {
        return Ok(PathBuf::from(p));
    }
    Ok(user_home()?.join(".android").join("avd"))
}
