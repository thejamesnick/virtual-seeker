//! Changes that need a writable `/system`: the Seeker device identity and
//! installing Seed Vault as a privileged system app (decisions D8, D12).

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail};

use crate::adb::Adb;
use crate::emulator::BOOT_TIMEOUT;
use crate::profile;

/// Partitions whose `build.prop` holds `ro.product.<partition>.model/brand/manufacturer`.
const BUILD_PROPS: &[&str] = &[
    "/system/build.prop",
    "/vendor/build.prop",
    "/product/etc/build.prop",
    "/odm/etc/build.prop",
    "/system_ext/etc/build.prop",
    "/vendor_dlkm/etc/build.prop",
    "/system_dlkm/etc/build.prop",
];

pub fn has_seeker_identity(adb: &Adb) -> bool {
    adb.getprop("ro.product.model") == profile::MODEL
        && adb.getprop("ro.product.brand") == profile::BRAND
        && adb.getprop("ro.product.manufacturer") == profile::MANUFACTURER
}

/// `adb root` + `adb remount`. The first time, Android enables an overlay and needs one reboot.
/// Only works when the emulator was started with `-writable-system`.
pub fn enable_writable_system(adb: &Adb) -> Result<()> {
    adb.root()?;
    if adb.remount()? {
        adb.reboot_and_wait(BOOT_TIMEOUT)?;
        adb.root()?;
        if adb.remount()? {
            bail!("system still not writable after reboot (was the emulator started with -writable-system?)");
        }
    }
    Ok(())
}

/// Makes the device report itself as a Seeker. Returns false if it already did.
pub fn apply_seeker_identity(adb: &Adb) -> Result<bool> {
    if has_seeker_identity(adb) {
        set_device_name(adb)?;
        return Ok(false);
    }
    enable_writable_system(adb)?;
    let (model, brand, manufacturer) = (profile::MODEL, profile::BRAND, profile::MANUFACTURER);
    for file in BUILD_PROPS {
        adb.shell(&format!(
            "[ -f {file} ] || exit 0; sed -i -E \
             -e 's/^(ro\\.product\\.([a-z_]+\\.)?model)=.*/\\1={model}/' \
             -e 's/^(ro\\.product\\.([a-z_]+\\.)?brand)=.*/\\1={brand}/' \
             -e 's/^(ro\\.product\\.([a-z_]+\\.)?manufacturer)=.*/\\1={manufacturer}/' {file}"
        ))
        .with_context(|| format!("editing {file}"))?;
    }
    adb.reboot_and_wait(BOOT_TIMEOUT)?;
    set_device_name(adb)?;
    if !has_seeker_identity(adb) {
        bail!("identity change did not stick (model is '{}')", adb.getprop("ro.product.model"));
    }
    Ok(true)
}

fn set_device_name(adb: &Adb) -> Result<()> {
    adb.shell(&format!("settings put global device_name {}", profile::MODEL))?;
    Ok(())
}

pub fn is_privileged(adb: &Adb, package: &str) -> bool {
    adb.shell(&format!("dumpsys package {package}"))
        .map(|o| o.contains("PRIVILEGED") && o.contains("codePath=/system/priv-app/"))
        .unwrap_or(false)
}

/// Installs an APK into `/system/priv-app/<name>/` and reboots so Android registers it.
pub fn install_privileged_app(adb: &Adb, apk: &Path, name: &str) -> Result<()> {
    enable_writable_system(adb)?;
    let dir = format!("/system/priv-app/{name}");
    let dest = format!("{dir}/{name}.apk");
    adb.shell(&format!("mkdir -p {dir}"))?;
    adb.run(&["push", &apk.to_string_lossy(), &dest])?;
    adb.shell(&format!("chmod 644 {dest}"))?;
    adb.reboot_and_wait(BOOT_TIMEOUT)?;
    std::thread::sleep(Duration::from_secs(2));
    Ok(())
}

/// Seeker display overlay (android/seeker-overlay, prebuilt): camera-hole cutout, rounded corners and a
/// status bar centered on the camera. Static system overlay in /product/overlay.
const DISPLAY_OVERLAY_APK: &[u8] = include_bytes!("../../../profiles/seeker/overlay/SeekerDisplayOverlay.apk");
const DISPLAY_OVERLAY_PACKAGE: &str = "dev.sms.overlay.seekerdisplay";
const DISPLAY_OVERLAY_PATH: &str = "/product/overlay/SeekerDisplayOverlay/SeekerDisplayOverlay.apk";

pub fn has_display_overlay(adb: &Adb) -> bool {
    let enabled = adb.shell("cmd overlay list android").map(|o| o.contains(&format!("[x] {DISPLAY_OVERLAY_PACKAGE}"))).unwrap_or(false);
    let size = adb.shell(&format!("stat -c %s {DISPLAY_OVERLAY_PATH}")).unwrap_or_default();
    enabled && size.trim() == DISPLAY_OVERLAY_APK.len().to_string()
}

/// Installs (or updates) the display overlay and reboots. Returns false if it was already current.
pub fn install_display_overlay(adb: &Adb) -> Result<bool> {
    if has_display_overlay(adb) {
        return Ok(false);
    }
    enable_writable_system(adb)?;
    let tmp = std::env::temp_dir().join(format!("sms-overlay-{}.apk", std::process::id()));
    std::fs::write(&tmp, DISPLAY_OVERLAY_APK)?;
    adb.shell("mkdir -p /product/overlay/SeekerDisplayOverlay")?;
    let pushed = adb.run(&["push", &tmp.to_string_lossy(), DISPLAY_OVERLAY_PATH]);
    let _ = std::fs::remove_file(&tmp);
    pushed?;
    adb.shell(&format!("chmod 644 {DISPLAY_OVERLAY_PATH}"))?;
    // Android's own cutout emulations would override ours.
    for emu in ["hole", "corner", "double", "tall", "waterfall", "emu01"] {
        let _ = adb.shell(&format!("cmd overlay disable com.android.internal.display.cutout.emulation.{emu}"));
    }
    adb.reboot_and_wait(BOOT_TIMEOUT)?;
    Ok(true)
}
