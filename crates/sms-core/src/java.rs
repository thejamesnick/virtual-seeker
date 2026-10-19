use std::path::{Path, PathBuf};
use std::process::Command;

use crate::util;

/// A Java runtime. Only needed by Google's `sdkmanager` / `avdmanager` (used by `sms setup`).
#[derive(Debug, Clone)]
pub struct Java {
    /// Value to export as `JAVA_HOME`.
    pub home: PathBuf,
    pub version: String,
}

/// Finds a working Java: `JAVA_HOME`, then Android Studio's bundled JBR, then `java` on PATH.
/// macOS ships a `/usr/bin/java` stub that fails without a JDK, so every candidate is actually run.
pub fn find() -> Option<Java> {
    let mut homes: Vec<PathBuf> = Vec::new();
    if let Some(h) = std::env::var_os("JAVA_HOME") {
        homes.push(PathBuf::from(h));
    }
    homes.push(PathBuf::from("/Applications/Android Studio.app/Contents/jbr/Contents/Home"));
    if let Ok(home) = crate::paths::user_home() {
        homes.push(home.join("Applications/Android Studio.app/Contents/jbr/Contents/Home"));
        homes.push(home.join("android-studio/jbr"));
    }
    homes.push(PathBuf::from("/opt/android-studio/jbr"));

    for home in homes {
        if let Some(version) = version_of(&home.join("bin/java")) {
            return Some(Java { home, version });
        }
    }

    let java = util::which("java")?;
    let version = version_of(&java)?;
    // bin/java -> JAVA_HOME
    let real = std::fs::canonicalize(&java).unwrap_or(java);
    let home = real.parent()?.parent()?.to_path_buf();
    Some(Java { home, version })
}

fn version_of(java: &Path) -> Option<String> {
    if !java.exists() {
        return None;
    }
    util::first_line(Command::new(java).arg("-version"))
}
