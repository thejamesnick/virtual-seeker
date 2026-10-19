//! Pre-loads the public BIP-39 test seed into the Seed Vault Simulator.
//!
//! The simulator keeps its seeds in an unencrypted protobuf DataStore file
//! (`files/datastore/seed_collection.pb`, schema in seed-vault-sdk `seed_repo.proto`).
//! Writing that file is far more reliable than typing 12 words through the UI.
//!
//! Only the well-known test phrase is supported, on purpose: real recovery phrases must never
//! end up in a plaintext file.

use std::io::Write;

use anyhow::{Context, Result, bail};

use crate::adb::Adb;
use crate::profile::SEED_VAULT_PACKAGE;

/// The standard BIP-39 test vector. Public. Never use it for real funds.
pub const TEST_PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
pub const TEST_SEED_NAME: &str = "SMS-test-seed";
pub const TEST_SEED_PIN: &str = "1234";

/// Word indices of `TEST_PHRASE` in the BIP-39 English list (abandon = 0, about = 3).
const TEST_WORD_INDICES: [u8; 12] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3];

/// PBKDF2-HMAC-SHA512(TEST_PHRASE, "mnemonic", 2048): the published BIP-39 seed for this phrase.
const TEST_BIP39_SEED: [u8; 64] = [
    0x5e, 0xb0, 0x0b, 0xbd, 0xdc, 0xf0, 0x69, 0x08, 0x48, 0x89, 0xa8, 0xab, 0x91, 0x55, 0x56, 0x81,
    0x65, 0xf5, 0xc4, 0x53, 0xcc, 0xb8, 0x5e, 0x70, 0x81, 0x1a, 0xae, 0xd6, 0xf6, 0xda, 0x5f, 0xc1,
    0x9a, 0x5a, 0xc4, 0x0b, 0x38, 0x9c, 0xd3, 0x70, 0xd0, 0x86, 0x20, 0x6d, 0xec, 0x8a, 0xa6, 0xc4,
    0x3d, 0xae, 0xa6, 0x69, 0x0f, 0x20, 0xad, 0x3d, 0x8d, 0x48, 0xb2, 0xd2, 0xce, 0x9e, 0x38, 0xe4,
];

/// The simulator's first seed id (`SeedRepository.FIRST_SEED_ID`).
const FIRST_SEED_ID: u64 = 1000;

const DATASTORE_DIR: &str = "/data/data/com.solanamobile.seedvaultimpl/files/datastore";
const DATASTORE_FILE: &str = "seed_collection.pb";

/// True if the simulator already has a seed collection with at least one seed.
pub fn has_seeds(adb: &Adb) -> bool {
    adb.shell(&format!("[ -s {DATASTORE_DIR}/{DATASTORE_FILE} ] && echo yes"))
        .map(|o| o.trim() == "yes")
        .unwrap_or(false)
}

/// Writes the test seed into the simulator if it has no seeds yet. Returns false if seeds already
/// existed (they are never overwritten). Needs root (`adb root`, available on google_apis images).
pub fn import_test_seed(adb: &Adb) -> Result<bool> {
    if !adb.has_package(SEED_VAULT_PACKAGE) {
        bail!("Seed Vault Simulator is not installed");
    }
    adb.root()?;
    if has_seeds(adb) {
        return Ok(false);
    }

    // The app's data dir exists once the app has run; launch it briefly if needed to get its owner.
    let app_dir = format!("/data/data/{SEED_VAULT_PACKAGE}");
    let mut owner = adb.shell(&format!("stat -c %U {app_dir}")).unwrap_or_default();
    if owner.is_empty() || owner.contains("No such file") {
        adb.shell(&format!("monkey -p {SEED_VAULT_PACKAGE} -c android.intent.category.LAUNCHER 1 >/dev/null 2>&1"))?;
        std::thread::sleep(std::time::Duration::from_secs(4));
        owner = adb.shell(&format!("stat -c %U {app_dir}")).context("Seed Vault data directory not found")?;
    }

    let bytes = test_seed_collection();
    let tmp = std::env::temp_dir().join(format!("sms-seed-{}.pb", std::process::id()));
    std::fs::File::create(&tmp)?.write_all(&bytes)?;
    let staging = "/data/local/tmp/sms_seed_collection.pb";
    let pushed = adb.run(&["push", &tmp.to_string_lossy(), staging]);
    let _ = std::fs::remove_file(&tmp);
    pushed?;

    // Stop the app so its DataStore doesn't overwrite the file, then install it with the app's
    // owner and SELinux label.
    adb.shell(&format!(
        "am force-stop {SEED_VAULT_PACKAGE} && mkdir -p {DATASTORE_DIR} && \
         cp {staging} {DATASTORE_DIR}/{DATASTORE_FILE} && rm {staging} && \
         chown -R {owner}:{owner} {app_dir}/files && chmod 700 {DATASTORE_DIR} && \
         chmod 600 {DATASTORE_DIR}/{DATASTORE_FILE} && restorecon -R {app_dir}/files"
    ))?;
    Ok(true)
}

/// Whether the simulator's seed list shows the test seed (reads the file back).
pub fn has_test_seed(adb: &Adb) -> bool {
    adb.shell(&format!("grep -c {TEST_SEED_NAME} {DATASTORE_DIR}/{DATASTORE_FILE}"))
        .map(|o| o.trim().parse::<u32>().unwrap_or(0) > 0)
        .unwrap_or(false)
}

/// Protobuf bytes of a `SeedCollection` holding just the test seed.
fn test_seed_collection() -> Vec<u8> {
    // SeedEntry
    let mut entry = Vec::new();
    field_bytes(&mut entry, 1, &TEST_BIP39_SEED); // seed
    field_bytes(&mut entry, 2, &TEST_WORD_INDICES); // seed_phrase_word_indices (packed; all < 128)
    field_bytes(&mut entry, 3, TEST_SEED_NAME.as_bytes()); // name
    field_bytes(&mut entry, 4, TEST_SEED_PIN.as_bytes()); // pin
    field_varint(&mut entry, 6, 1); // is_backed_up

    // SeedRecord
    let mut record = Vec::new();
    field_varint(&mut record, 1, FIRST_SEED_ID); // seed_id
    field_bytes(&mut record, 2, &entry); // seed

    // SeedCollection
    let mut collection = Vec::new();
    field_bytes(&mut collection, 1, &record); // seeds
    field_varint(&mut collection, 2, FIRST_SEED_ID + 1); // next_id
    collection
}

fn varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn field_varint(out: &mut Vec<u8>, field: u64, v: u64) {
    varint(out, field << 3);
    varint(out, v);
}

fn field_bytes(out: &mut Vec<u8>, field: u64, bytes: &[u8]) {
    varint(out, (field << 3) | 2);
    varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Same bytes the simulator itself wrote after importing the phrase by hand (pulled from a
    /// device, see docs/09-test-seed.md): record header, seed id 1000, 64-byte seed, indices, name, pin.
    #[test]
    fn encodes_like_the_simulator() {
        let b = test_seed_collection();
        assert_eq!(&b[..2], &[0x0a, b.len() as u8 - 5]); // SeedCollection.seeds, length
        assert_eq!(&b[2..5], &[0x08, 0xe8, 0x07]); // seed_id = 1000
        assert_eq!(&b[7..9], &[0x0a, 0x40]); // SeedEntry.seed, 64 bytes
        assert_eq!(&b[9..13], &[0x5e, 0xb0, 0x0b, 0xbd]);
        assert_eq!(&b[73..75], &[0x12, 0x0c]); // 12 packed word indices
        assert!(b.windows(TEST_SEED_NAME.len()).any(|w| w == TEST_SEED_NAME.as_bytes()));
        assert_eq!(&b[b.len() - 3..], &[0x10, 0xe9, 0x07]); // next_id = 1001
    }

    #[test]
    fn varint_encoding() {
        let mut v = Vec::new();
        varint(&mut v, 1000);
        assert_eq!(v, [0xe8, 0x07]);
    }
}
