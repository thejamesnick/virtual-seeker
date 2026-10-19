# `sms` CLI (v0.1)

Built 2026-10-19. Automates everything from [06-virtual-seeker.md](06-virtual-seeker.md) and [07-wallet-and-seed-vault.md](07-wallet-and-seed-vault.md).

## Install (from source)
```bash
cargo install --path crates/sms-cli    # puts `sms` in ~/.cargo/bin
```

## Quick start
```bash
sms doctor          # what's installed, what's missing, how to fix it
sms setup           # install tools + build the virtual Seeker (asks about Google's SDK license)
sms start           # open the virtual Seeker (add --localnet for a local Solana network)
sms install app.apk
sms logs            # wallet / Seed Vault / crash logs
sms stop
```

## Commands

| Command | What it does |
|---|---|
| `sms doctor [--json]` | Checks host, Java, Android SDK, adb, emulator, command-line tools, hardware acceleration, Android 16 image, the Seeker AVD (profile applied?), Seed Vault APK, Surfpool, Solana CLI, the running device (identity + privileged Seed Vault) and disk space. Each problem shows a fix. Exit code 1 if anything is ✗. |
| `sms setup` | Safe to re-run; each step is skipped when done: 1) find Java, 2) find or create the SDK (`~/.sms/android-sdk` if none), 3) download Google's command-line tools if missing, 4) **ask the user to accept Google's Android SDK license** (or `--accept-licenses`), 5) install `platform-tools`, `emulator`, `platforms;android-36`, `system-images;android-36;google_apis;<arch>`, 6) create the AVD + apply [hardware.ini](../profiles/seeker/hardware.ini), 7) find **or build** the Seed Vault Simulator APK, 8) boot headless → Seeker identity → Seed Vault Simulator as privileged system app → **load the public test seed** → download + install fakewallet → shut down. Options: `--recreate`, `--seed-vault-apk PATH`, `--no-seed-vault`, `--no-device`. See [09-auto-seed-vault.md](09-auto-seed-vault.md). |
| `sms start` | Boots with `-writable-system -gpu host -no-boot-anim` in the background (keeps running after `sms` exits, Ctrl-C safe), waits for boot, sets `adb reverse` for 8899/8900, prints device info. `--localnet` also starts Surfpool (`--offline --no-tui`), or reuses one already on :8899. `--headless`, `--gpu`, `--wipe`. |
| `sms stop` | `adb emu kill`, waits until gone, and stops the Surfpool that `sms start` launched (`--keep-localnet` to keep it). |
| `sms install APK...` | `adb install -r` on the running virtual Seeker (or `--serial`, or the only connected device). |
| `sms logs` | `adb logcat` filtered to MWA + Seed Vault tags + crashes. `--app PKG` for one app, `--all` for everything, `--clear` to start fresh. |
| `sms devices [--json]` | Lists devices as **virtual Seeker**, **real Seeker**, emulator, or Android device. A first step toward USB support. |
| global `--avd NAME` | Use another virtual device (env `SMS_AVD`, default `Seeker`). |

## Files
```
~/.sms/              (override: SMS_HOME)
  apks/              fakewallet-v1-release.apk (downloaded), SeedVaultSimulator.apk (built or copied here)
  src/seed-vault-sdk/  pinned source used to build the simulator
  logs/              emulator.log, surfpool.log
  run/               emulator.pid, surfpool.pid
  downloads/         command-line tools zip (fresh machines)
  android-sdk/       only if no SDK was found
~/.android/avd/Seeker.avd   the virtual Seeker
```
The Seed Vault Simulator APK is looked up in: `--seed-vault-apk`, `$SMS_SEED_VAULT_APK`, `~/.sms/apks/SeedVaultSimulator.apk`, `./vendor/seed-vault-sdk/.../SeedVaultSimulator-debug.apk`.

## Code layout
```
Cargo.toml                 workspace
crates/sms-core/           engine (reused by the desktop app later)
  src/sdk.rs               find Android SDK, tool paths, packages, accel check
  src/java.rs              find a working Java (JAVA_HOME, Android Studio JBR, PATH)
  src/adb.rs               adb wrapper bound to a serial (shell, getprop, install, reverse, root, remount)
  src/emulator.rs          start/stop/find running AVD, apply hardware profile
  src/identity.rs          writable system, Seeker identity, privileged app install
  src/localnet.rs          Surfpool start/stop
  src/seedvault.rs         writes the public test seed into the simulator's datastore
  src/doctor.rs            checks
  src/setup.rs             setup steps, cmdline-tools download, licenses, AVD create
  src/profile.rs           Seeker constants; includes profiles/seeker/hardware.ini at build time
crates/sms-cli/src/main.rs `sms` command (clap)
```
Dependencies: `anyhow`, `serde_json`, `clap`. Downloads use the system `curl`/`unzip`. We shell out to Google's `adb`/`emulator`/`sdkmanager`/`avdmanager` (decision D13).

## What was tested (2026-10-19, M1 Pro Mac)
| Test | Result |
|---|---|
| `cargo test` | 3 unit tests pass (hardware profile parsing, system image name, cmdline-tools version picking) |
| `sms doctor` (text + `--json`) | all ✓, live device shows model Seeker + privileged Seed Vault |
| `sms devices` | `emulator-5554  virtual Seeker  Seeker  16` |
| `sms setup` on the existing device | everything skipped as done, wallet downloaded, **17 s** |
| **`sms --avd SeekerTest setup` from zero** | created AVD → identity changed → Seed Vault installed privileged → wallet installed, **112 s**. Verified after `start`: model Seeker, Android 16, 1200x2670, Seed Vault privileged, fakewallet present. (Test AVD deleted after.) |
| `sms start --localnet` | ready in **6 s**, reused running Surfpool, `adb reverse` set |
| `sms start --headless` | ready in 6 s |
| `sms install fakedapp-release.apk` | ✓ |
| `sms logs --clear` during an MWA authorize | showed `Encrypted session established` → `Authorized: … fakewallet account 0` → `session closed` |
| `sms stop` | 5–70 s (the emulator saves its quick-boot RAM snapshot on shutdown; `stop` now says so and waits up to 3 min) |

### Not tested yet
- **A truly fresh machine**: command-line tools download, license prompt, package install. This Mac already had them. Needs a clean macOS user or VM.
- Linux (code paths exist for `~/Android/Sdk`, `linux` command-line tools, x86_64 images).
- Real Seeker over USB (`sms devices` would label it "real Seeker").

## Known limits / next
- `sms setup` still needs **Java** (for Google's `sdkmanager`/`avdmanager`). Goal (D4): download packages directly and drop Java.
- ~~Seed Vault APK built by hand~~ → `sms setup` builds it (doc 09). Publishing a prebuilt APK would still save users ~5 min and ~1 GB.
- ~~Test seed typed by hand~~ → loaded automatically (doc 09).
- `--localnet` sign-and-send needs a dapp that requests the `localnet` chain.
