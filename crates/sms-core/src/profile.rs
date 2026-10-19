//! The Seeker device profile.
//!
//! Hardware values live in `profiles/seeker/hardware.ini` (one source of truth for the
//! manual scripts and this code). See docs/06-virtual-seeker.md for sources.

use crate::sdk;

/// Emulator hardware settings applied to the AVD's `config.ini`.
pub const HARDWARE_INI: &str = include_str!("../../../profiles/seeker/hardware.ini");

/// Default AVD name. Override with `--avd` or `SMS_AVD`.
pub const DEFAULT_AVD: &str = "Seeker";

/// Seeker runs Android 16 = API 36.
pub const API_LEVEL: u32 = 36;

/// Software identity of a real Seeker (from Solana Mobile's device-detection docs).
pub const MODEL: &str = "Seeker";
pub const BRAND: &str = "solanamobile";
pub const MANUFACTURER: &str = "Solana Mobile Inc.";

/// `google_apis` (not `playstore`): needed for `adb root` / writable system.
pub fn system_image() -> String {
    format!("system-images;android-{API_LEVEL};google_apis;{}", sdk::host_abi())
}

pub fn platform_package() -> String {
    format!("platforms;android-{API_LEVEL}")
}

/// SDK packages `sms setup` installs.
pub fn required_packages() -> Vec<String> {
    vec!["platform-tools".into(), "emulator".into(), platform_package(), system_image()]
}

/// `key=value` pairs from `HARDWARE_INI`, without comments.
pub fn hardware_settings() -> Vec<(&'static str, &'static str)> {
    HARDWARE_INI
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim(), v.trim()))
        .collect()
}

pub fn avd_name() -> String {
    std::env::var("SMS_AVD").unwrap_or_else(|_| DEFAULT_AVD.to_string())
}

/// Official MWA test wallet from the mobile-wallet-adapter GitHub releases.
pub const FAKEWALLET_URL: &str = "https://github.com/solana-mobile/mobile-wallet-adapter/releases/download/%40solana-mobile/mobile-wallet-adapter-protocol%403.0.1/fakewallet-v1-release.apk";
pub const FAKEWALLET_PACKAGE: &str = "com.solana.mobilewalletadapter.fakewallet";

/// Our bridge test wallet (android/sms-wallet): MWA outside, Seed Vault inside (decision D10).
pub const SMS_WALLET_PACKAGE: &str = "dev.sms.wallet";
pub const SMS_WALLET_APK_NAME: &str = "SmsTestWallet.apk";

/// Seed Vault Simulator (built from solana-mobile/seed-vault-sdk; no official prebuilt APK).
pub const SEED_VAULT_PACKAGE: &str = "com.solanamobile.seedvaultimpl";
pub const SEED_VAULT_APK_NAME: &str = "SeedVaultSimulator.apk";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_profile_has_seeker_screen() {
        let s = hardware_settings();
        let get = |k| s.iter().find(|(key, _)| *key == k).map(|(_, v)| *v);
        assert_eq!(get("hw.lcd.width"), Some("1200"));
        assert_eq!(get("hw.lcd.height"), Some("2670"));
        assert_eq!(get("hw.gpu.mode"), Some("host"));
        assert!(s.iter().all(|(k, _)| !k.starts_with('#')));
    }

    #[test]
    fn system_image_is_android_16_google_apis() {
        assert!(system_image().starts_with("system-images;android-36;google_apis;"));
    }
}
