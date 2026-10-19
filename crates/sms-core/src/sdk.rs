use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use crate::java::Java;
use crate::{paths, util};

/// An Android SDK folder (`ANDROID_HOME`).
#[derive(Debug, Clone)]
pub struct AndroidSdk {
    pub root: PathBuf,
}

impl AndroidSdk {
    /// Finds an existing SDK: `ANDROID_HOME`, `ANDROID_SDK_ROOT`, the Android Studio default,
    /// then `~/.sms/android-sdk` (where `sms setup` installs one).
    pub fn find(sms: &paths::SmsPaths) -> Option<Self> {
        Self::candidates(sms)
            .into_iter()
            .find(|p| p.join("platform-tools").exists() || p.join("cmdline-tools").exists() || p.join("emulator").exists())
            .map(|root| Self { root })
    }

    /// The SDK to use, existing or not: the first found, else `~/.sms/android-sdk`.
    pub fn find_or_default(sms: &paths::SmsPaths) -> Self {
        Self::find(sms).unwrap_or_else(|| Self { root: sms.home.join("android-sdk") })
    }

    fn candidates(sms: &paths::SmsPaths) -> Vec<PathBuf> {
        let mut c = Vec::new();
        for var in ["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
            if let Some(p) = std::env::var_os(var) {
                c.push(PathBuf::from(p));
            }
        }
        if let Ok(home) = paths::user_home() {
            c.push(home.join("Library/Android/sdk")); // macOS (Android Studio default)
            c.push(home.join("Android/Sdk")); // Linux (Android Studio default)
        }
        c.push(sms.home.join("android-sdk"));
        c
    }

    pub fn adb(&self) -> PathBuf {
        self.root.join("platform-tools/adb")
    }
    pub fn emulator(&self) -> PathBuf {
        self.root.join("emulator/emulator")
    }
    pub fn sdkmanager(&self) -> PathBuf {
        self.root.join("cmdline-tools/latest/bin/sdkmanager")
    }
    pub fn avdmanager(&self) -> PathBuf {
        self.root.join("cmdline-tools/latest/bin/avdmanager")
    }

    /// Path of an SDK package, e.g. `system-images;android-36;google_apis;arm64-v8a`.
    pub fn package_dir(&self, package: &str) -> PathBuf {
        package.split(';').fold(self.root.clone(), |p, part| p.join(part))
    }

    pub fn has_package(&self, package: &str) -> bool {
        self.package_dir(package).join("package.xml").exists()
    }

    pub fn licenses_accepted(&self) -> bool {
        self.root.join("licenses/android-sdk-license").exists()
    }

    /// `sdkmanager` / `avdmanager` with `JAVA_HOME` and the SDK root set.
    pub fn java_tool(&self, tool: &Path, java: &Java) -> Command {
        let mut cmd = Command::new(tool);
        cmd.env("JAVA_HOME", &java.home)
            .env("ANDROID_HOME", &self.root)
            .env("ANDROID_SDK_ROOT", &self.root);
        cmd
    }

    pub fn emulator_version(&self) -> Option<String> {
        util::first_line(Command::new(self.emulator()).arg("-version"))
    }

    pub fn adb_version(&self) -> Option<String> {
        util::first_line(Command::new(self.adb()).arg("version"))
    }

    /// Asks the emulator whether hardware acceleration (Hypervisor.Framework / KVM) works.
    pub fn accel_check(&self) -> Result<String> {
        let out = Command::new(self.emulator())
            .arg("-accel-check")
            .output()
            .context("running emulator -accel-check")?;
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        Ok(text.lines().map(str::trim).filter(|l| !l.is_empty() && *l != "accel:" && *l != "accel" && *l != "0").collect::<Vec<_>>().join(" "))
    }
}

/// CPU ABI of the host, which decides the system image to use.
pub fn host_abi() -> &'static str {
    if cfg!(target_arch = "aarch64") { "arm64-v8a" } else { "x86_64" }
}

/// Name of the OS part of Google's command-line tools download.
pub fn cmdline_tools_os() -> &'static str {
    if cfg!(target_os = "macos") { "mac" } else { "linux" }
}
