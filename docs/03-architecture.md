# Architecture (draft)

> v0.1 implemented: `crates/sms-core` (engine) + `crates/sms-cli` (`sms`). See [08-sms-cli.md](08-sms-cli.md).

## Big picture

```
┌──────────────────────────── Desktop app (Tauri) ────────────────────────────┐
│  Phone view (Seeker frame)          │  Solana panel                         │
│  - screen stream from device        │  - wallet requests approve/reject     │
│  - mouse/keyboard → touch/keys      │  - Seed Vault status                  │
│                                     │  - network: localnet/devnet/mainnet   │
│  Devices: ● Virtual  ● Real (USB)   │  - mock Genesis Token, .skr           │
│                                     │  - transactions, logs, compare        │
└──────────────────────────────┬──────────────────────────────────────────────┘
                               │ same Rust core
┌──────────────────────────────▼──────────────────────────────────────────────┐
│  sms core (Rust)                                                            │
│  setup · device manager · emulator lifecycle · adb · logs · network · tests │
└───────┬───────────────────┬────────────────────┬─────────────────┬──────────┘
        │                   │                    │                 │
  Android emulator     adb (USB/Wi-Fi)       Surfpool         Solana RPC
  (headless, engine)   real Seeker           local network    devnet/mainnet
        │
  Virtual Seeker = Android 16 image + Seeker profile + Seed Vault Simulator
                   + test wallet + Seeker theme
```

The **CLI** (`sms`) and the **desktop app** are two front-ends on the same Rust core.

## Pieces

### 1. Setup (`sms setup`)
- Downloads directly from Google into `~/.sms/` (planned path): the emulator, the Android 16 arm64 `google_apis` image and platform-tools. **No Android Studio.**
- Shows Google's Android SDK license for the user to accept.
- Downloads or builds the Seed Vault Simulator and test wallet, plus Surfpool.
- Goal: try to avoid needing Java by fetching packages directly instead of through `sdkmanager`.

### 2. Virtual Seeker device
- A custom AVD hardware profile with Seeker's screen size, density and rounded corners.
- Device identity: `Build.MODEL=Seeker`, `BRAND=solanamobile`, `MANUFACTURER=Solana Mobile Inc.`. **Open question:** these are read-only system properties. They may need a custom/modified system image. Test first.
- Seed Vault Simulator installed and pre-loaded with a **test-only** seed.
- A test wallet connected to Seed Vault (based on fakewallet or mock-mwa-wallet).
- Seeker-style theme: wallpaper, colors, dark mode, and later our own launcher.
- Snapshots, so it boots in seconds and starts from a clean, repeatable state.

### 3. Real Seeker over USB / Wi-Fi
- The user enables USB debugging. We talk to the phone through `adb`.
- Screen mirroring and control via **scrcpy** (Apache-2.0).
- Install, logs and tests work the same as on the virtual Seeker.
- We **never** touch Seed Vault or keys on a real phone. Real approvals are made by the user.
- **Compare mode:** run the same test on both devices and show the differences.

### 4. Solana layer
- Surfpool local network by default; devnet and mainnet are optional (with a clear warning for mainnet).
- Mock Seeker Genesis Token and `.skr` names deployed to localnet.
- Transaction inspection (`sms tx <sig>`).

### 5. Automation / CI
- A scriptable wallet mode (auto-approve or reject by rule) so tests run with no taps.
- Maestro flows.
- CI runs on Linux with KVM (x86_64 images), because macOS CI runners can't run emulators.

## Tech choices (draft)
| Part | Choice |
|---|---|
| Core + CLI | Rust (`clap`, `tokio`, `serde`/`toml`) |
| Desktop app | Tauri |
| Android talk | shell out to `adb` first, maybe the `adb_client` crate later |
| Screen streaming | emulator gRPC/headless stream for the virtual device; scrcpy for real devices |
| Android-side apps | Kotlin (Seed Vault Simulator, wallet, launcher) |
| Local network | Surfpool |
| UI automation | Maestro |

## Security rules
- Test seeds only, clearly labeled. Never production keys.
- Never log private keys.
- Warn loudly before using mainnet.
- Simulated Seed Vault is **never** presented as secure.
