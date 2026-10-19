# Hands-off Seed Vault: auto-build + test seed (v0.1)

Added 2026-10-19 so `sms setup` needs **no manual steps** for Seed Vault.

## 1. The test seed is written directly into Seed Vault

### How the simulator stores seeds
The Seed Vault Simulator keeps all seeds in **one unencrypted protobuf file** (it's a simulator, with no secure hardware):
```
/data/data/com.solanamobile.seedvaultimpl/files/datastore/seed_collection.pb
```
Schema: `SeedVaultSimulator/src/main/proto/seed_repo.proto`
```
SeedCollection { repeated SeedRecord seeds = 1; int64 next_id = 2; int64 next_auth_token = 3; int64 next_account_id = 4; }
SeedRecord     { int64 seed_id = 1; SeedEntry seed = 2; repeated AuthorizationEntry authorizations = 3; repeated KnownAccountEntry known_accounts = 4; }
SeedEntry      { bytes seed = 1; repeated int32 seed_phrase_word_indices = 2; string name = 3; string pin = 4; bool unlock_with_biometrics = 5; bool is_backed_up = 6; }
```
We pulled the file from our virtual Seeker after importing the phrase by hand and decoded it:
- `seed` = the **64-byte BIP-39 seed** (`5eb00bbd…38e4`, the published value for the test phrase), not the entropy
- word indices `0 ×11, 3` (abandon = 0, about = 3), name, PIN
- first `seed_id` = 1000, `next_id` = 1001
- accounts (`known_accounts`) are **not** needed: the simulator derives them when a wallet authorizes the seed (`PrepopulateKnownAccountsUseCase`)
- the `next_*` counters only move forward (`SeedRepository`), so leaving them out is safe

### What `sms setup` does ([seedvault.rs](../crates/sms-core/src/seedvault.rs))
1. Skip if the simulator already has seeds. **Your seeds are never overwritten.**
2. Encode a `SeedCollection` with one seed (hand-written protobuf, no extra crates).
3. `adb root` → push to `/data/local/tmp` → `am force-stop` the simulator → copy into `files/datastore/` → `chown` to the app's user → `chmod 600` → `restorecon` (SELinux label).

| | |
|---|---|
| Name | `SMS-test-seed` |
| PIN | `1234` |
| Phrase | `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about` |
| `m/44'/501'/0'` | `GjJyeC1r2RgkuoCWMyPYkCWSGSGLcz266EaAkLA27AhL` |
| `m/44'/501'/1'` | `ANf3TEKFL6jPWjzkndo4CbnNdUNkBk4KHPggJs2nu8Xi` |

⚠️ **This phrase is public. Never send real funds to these addresses.**
Only this test phrase is supported, **on purpose**: a real recovery phrase must never be written to a plaintext file.

Unit tests check that our bytes match what the simulator writes (`cargo test`).

## 2. The Seed Vault Simulator APK is built automatically
No official prebuilt APK exists. `sms setup` now looks for one, in this order:
1. `--seed-vault-apk PATH`
2. `$SMS_SEED_VAULT_APK`
3. `~/.sms/apks/SeedVaultSimulator.apk` (cache)
4. `./vendor/seed-vault-sdk/.../SeedVaultSimulator-debug.apk`

If none is found it **builds one**:
- fetches `solana-mobile/seed-vault-sdk` at a **pinned commit** `efb0e6f3` (2026-10-14, the one we tested) into `~/.sms/src/seed-vault-sdk`
- `./gradlew :SeedVaultSimulator:assembleDebug` with our Java + SDK
- copies the APK to `~/.sms/apks/SeedVaultSimulator.apk`, so later runs reuse it

`--no-seed-vault` skips Seed Vault entirely.

Needs `git` and the same Java that setup already needs. The first build downloads Gradle and its libraries (~1 GB).

## Test results (2026-10-19)
Fresh-user simulation: empty `SMS_HOME` (no cached APK), run from another folder (no `vendor/`), new device `SeekerFresh`:
```
→ Creating virtual device 'SeekerFresh'
→ Building the Seed Vault Simulator from solana-mobile/seed-vault-sdk …
→ Applying the Seeker identity …  identity changed
→ Installing Seed Vault Simulator as a privileged system app
→ Loaded the public test seed 'SMS-test-seed' into Seed Vault (PIN 1234). Test funds only!
→ Downloading / installing the MWA test wallet
✓ Virtual Seeker ready in 172s.
```
(Gradle's cache in `~/.gradle` was already warm from earlier builds, so a truly first-time build will be slower.)

Then on that device:
| Check | Result |
|---|---|
| `sms doctor` | `✓ Test seed 'SMS-test-seed' (PIN 1234, public test phrase)` |
| Seed Vault app | lists **SMS-test-seed** |
| Seed Vault fakewallet → authorize seed (PIN 1234) | accounts `m/44'/501'/0'` → `GjJyeC1r…AhL`, `/1'` → `ANf3TEKF…8Xi` ✅ (same as `solana-keygen`) |
| Sign txn V0 | `Transaction signed … mResolvedDerivationPaths=[bip32:/m/44'/501'/0']` ✅ |

The test device was deleted afterwards.
