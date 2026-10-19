# Decision log

Newest at the bottom. Each entry says what was decided, and why.

### D1 (2026-10-18): Build a *virtual Seeker*, not a generic dev CLI
- **Decided:** the product is a Seeker-like emulator: Android 16, Seeker screen and identity, Seed Vault Simulator, wallet, mock Seeker features, and a Seeker-style theme.
- **Why:** research found the official `solana-mobile` CLI already covers a generic emulator + test wallet. Nobody offers a virtual *Seeker*.
- **Changes vs About.txt:** the focus moves from "orchestrate existing tools" to "faithful Seeker device".

### D2 (2026-10-18): Promise = "works on virtual Seeker → works on real Seeker", proven by comparison
- **Decided:** we don't claim 100%. We list the limits and prove accuracy by comparing against real Seekers.
- **Why:** no emulator can copy secure hardware, attestation or private system apps.

### D3 (2026-10-18): Desktop app is the main product, CLI for automation
- **Decided:** a Tauri desktop app with the phone in a Seeker-style frame and a Solana side panel. The `sms` CLI shares the same Rust core.
- **Why:** much better experience for most developers, and the CLI still serves CI.

### D4 (2026-10-18): No Android Studio required
- **Decided:** `sms setup` downloads the emulator, system image and platform-tools directly from Google.
- **Why:** Android Studio is a big download that only some users need. Our tool just needs the emulator engine and an `.apk`.

### D5 (2026-10-18): Real Seeker over USB / Wi-Fi in the same app
- **Decided:** support real devices via `adb`, with screen mirroring via scrcpy (Apache-2.0) and a compare mode.
- **Why:** lets developers check on real hardware and gives us the data to prove D2.

### D6 (2026-10-18): Seeker-*style* look, not a copy
- **Decided:** our own theme inspired by Seeker. No copied Solana Mobile logos or artwork unless they agree.
- **Why:** trademark/copyright, and the project must not look official.

### D7 (2026-10-18): Dev Mac without Homebrew or Docker
- **Decided:** built libusb from source for the Solana CLI, and used native Surfpool instead of Docker.
- **Why:** fewer big installs. See [05-dev-setup.md](05-dev-setup.md).

### D8 (2026-10-18): Seeker identity via writable-system overlay, not a custom system image
- **Decided:** use Google's stock `google_apis` arm64 image, boot with `-writable-system`, and edit `ro.product.*` in each partition's `build.prop`.
- **Why:** tested and it works (Settings shows Model: Seeker). Building a custom AOSP image would be far more work. Trade-off: the emulator must always launch with `-writable-system`, and low-level emulator props (`ro.hardware=ranchu`, `ro.kernel.qemu`) stay.

### D9 (2026-10-18): Lower RAM/CPU than the real Seeker by default
- **Decided:** 4 GB RAM and 4 cores (real: 8 GB, 8 cores), configurable.
- **Why:** keeps a 16 GB Mac usable. Apps rarely depend on exact RAM.

### D10 (2026-10-19): Build our own "bridge" test wallet (MWA outside, Seed Vault inside)
- **Decided:** the virtual Seeker needs a test wallet that talks MWA to apps **and** keeps keys in Seed Vault, like the real Seeker's Seed Vault Wallet.
- **Why:** none of the official test wallets do both (MWA fakewallet = own keys; Seed Vault fakewallet = no MWA; mock-mwa-wallet = fake Seed Vault UI). Without it, the real Seeker path (app → MWA → wallet → Seed Vault) can't be tested. It can also add the scriptable approve/reject mode CI needs.

### D11 (2026-10-19): Use the host GPU by default
- **Decided:** `hw.gpu.mode=host` / `-gpu host`.
- **Why:** software rendering (SwiftShader) made the whole system so slow that the Seed Vault Simulator froze (ANR). With the host GPU: 6 s boot, no freezes. `sms doctor` should warn if the GPU falls back to software.

### D12 (2026-10-19): Seed Vault Simulator installed as a privileged system app
- **Decided:** push it to `/system/priv-app` instead of a normal `adb install`.
- **Why:** that's how Seed Vault is on a real Seeker (privileged, priority-100 intents). Possible thanks to the writable system from D8.

### D13 (2026-10-19): Rust workspace `sms-core` + `sms-cli`, shelling out to Google's tools
- **Decided:** a Cargo workspace with an engine crate (`sms-core`, reused by the desktop app) and a thin CLI crate (`sms-cli`, binary `sms`). Run `adb`, `emulator`, `sdkmanager`, `avdmanager`, `curl` and `unzip` as processes instead of reimplementing them. Minimal dependencies (`anyhow`, `serde_json`, `clap`).
- **Why:** the official tools are the most reliable and stay up to date. A small dependency tree builds fast. The `adb_client` crate can replace shelling out later if needed.
- **Trade-off:** `sms setup` still needs Java for `sdkmanager`/`avdmanager` (D4's "no Java" goal is open).

### D14 (2026-10-19): `sms setup` asks the user to accept Google's SDK license
- **Decided:** if no license is recorded, `sms setup` shows the link and asks y/N. `--accept-licenses` is for scripts and CI. We never accept silently.
- **Why:** it's the user's agreement with Google, not ours.

### D15 (2026-10-19): Load the test seed by writing the simulator's datastore file
- **Decided:** `sms setup` writes a protobuf `seed_collection.pb` holding the public BIP-39 test phrase, instead of typing it through the UI.
- **Why:** UI typing was unreliable (keyboard autocomplete corrupted words). The file format is simple and documented by the SDK's `.proto`. Only the public test phrase is supported, so a real phrase never lands in a plaintext file. Existing seeds are never overwritten.

### D16 (2026-10-19): Build the Seed Vault Simulator from a pinned commit when no APK is available
- **Decided:** fetch `solana-mobile/seed-vault-sdk` at commit `efb0e6f3` into `~/.sms/src` and build it with Gradle, then cache the APK in `~/.sms/apks`.
- **Why:** there is no official prebuilt APK. Pinning keeps builds reproducible and matches what we tested. Later: publish our own prebuilt (Apache-2.0 allows it) to skip the ~5 min build.

### D17 (2026-10-19): SMS Test Wallet is the default wallet on the virtual Seeker
- **Decided:** our own Kotlin app (`android/sms-wallet`, package `dev.sms.wallet`) built on the official `walletlib` + `seedvault-wallet-sdk` from Maven. `sms setup` installs it and disables the generic fakewallet.
- **Why:** it's the only way to test the real Seeker path (app → MWA → wallet → Seed Vault). One wallet avoids the "choose a wallet" popup a real Seeker never shows.
- **Design:** the wallet sends each transaction's *message bytes* to Seed Vault and inserts the signature itself. A policy (ask/approve/reject, settable over adb) automates the wallet prompt for CI, while Seed Vault's PIN stays manual to match a real Seeker.

### D18 (2026-10-19): Skin drawn to scale from public measurements, perfectly symmetric
- **Decided:** draw the frame in code (Swift/CoreGraphics) from the official 73.5 × 154.8 mm spec and measurements taken from Solana Mobile's official renders (button positions, corner radius) and review photos (camera). No photos or logos are copied. Every side gets exactly the same border (body height rounded 0.1 mm to make that true) and symmetric shading. `check-skin.swift` verifies it.
- **Why:** the user wants it "pixel perfect" and recognisably a Seeker, while we can't ship Solana Mobile's artwork (D6).

### D19 (2026-10-19): Theme via Android settings + a wallpaper activity in the SMS Test Wallet
- **Decided:** dark mode and Material You palette (`#9945FF`) through `cmd uimode` / `theme_customization_overlay_packages`. The wallpaper is set by `ThemeActivity` in the SMS Test Wallet, because Android has no shell command for it.
- **Why:** no custom system image needed. Everything is reversible, and `sms setup` applies it.

### D20 (2026-10-19): Corner radius 9.7 mm so the emulator's layers can't misalign
- **Decided:** body corner 9.7 mm (render ~10.8), metal band 0.9 mm. The mask repaints the body in its corners, the glass is pure black and runs under the display edge, and the colors are sampled from the official renders.
- **Why:** the emulator draws the display rectangle over the frame and scales those two layers differently. Any metal inside the display rectangle showed as square notches or steps. At 9.7 mm the metal never enters it, which is the only geometry that looks right at every window size.

### D21 (2026-10-19): Display overlay for camera cutout, corners and status bar
- **Decided:** our own static framework overlay (cutout path, status bar height 134 px, corner radii) in `/product/overlay`, prebuilt APK embedded in `sms`.
- **Why:** that's how real devices tell Android about their hardware, so apps see the same `DisplayCutout` and rounded-corner insets as on a Seeker, and the status bar lines up with the camera. Bonus: apps that handle cutouts can now be tested properly.
