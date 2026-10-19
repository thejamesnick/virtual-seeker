//! Core engine for the virtual Seeker.
//!
//! Everything here is shared by the `sms` CLI and (later) the desktop app.
//! We shell out to Google's tools (`adb`, `emulator`, `sdkmanager`, `avdmanager`)
//! instead of reimplementing them.

pub mod adb;
pub mod doctor;
pub mod emulator;
pub mod home;
pub mod identity;
pub mod java;
pub mod localnet;
pub mod paths;
pub mod profile;
pub mod sdk;
pub mod seedvault;
pub mod setup;
pub mod theme;
pub mod util;

pub use paths::SmsPaths;
pub use sdk::AndroidSdk;
