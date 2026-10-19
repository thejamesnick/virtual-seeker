# Work log

## 2026-10-18
- Read About.txt (original concept).
- Researched Solana Mobile: Seeker OS, MWA, Seed Vault, dApp Store, existing tooling, emulator/CI, Rust crates. Report in [reports/](../reports/).
- Key finding: the official `solana-mobile` CLI exists and does a generic emulator, but no "virtual Seeker" exists → new direction (decisions D1–D6).
- Set up the dev Mac: SDK command-line tools, Android 16 image, Solana CLI 4.3.1 (+ libusb fix), Surfpool 1.6.1, wallet APKs, Solana Mobile source repos, Maestro. Details in [05-dev-setup.md](05-dev-setup.md).
- Wrote project docs.
- All tools verified working (see dev-setup "Verified" table).
- Built the first virtual Seeker: `Seeker` AVD (Android 16, 1200x2670, density 480), booted in ~18 s.
- Device identity changed to Seeker / solanamobile / Solana Mobile Inc. via writable system + build.prop edits. Settings shows Model: Seeker. Details in [06-virtual-seeker.md](06-virtual-seeker.md).
- **Next:** build + install the Seed Vault Simulator with a test seed, then a wallet + one MWA sign flow on Surfpool.

## 2026-10-19
- MWA on the virtual Seeker: fakedapp ↔ fakewallet authorize, sign txn (signature verified), decline (`-3`). ✅
- Surfpool 1.6.1 running offline on :8899, forwarded with `adb reverse`.
- Built Seed Vault Simulator + Seed Vault fakewallet from source (Gradle 9.7.1).
- Installed the simulator as a **privileged system app** (`install-seed-vault.sh`). Imported public test seed `SMS-test-seed`.
- Wallet ↔ Seed Vault: authorize, derive keys (match `solana-keygen`), sign, decline. ✅
- Found and fixed: SwiftShader freezes → `-gpu host` (6 s boot); stale-dump bug in `tap.sh`.
- Key finding: no official wallet does MWA + Seed Vault together → we build a bridge wallet (D10).
- Details in [07-wallet-and-seed-vault.md](07-wallet-and-seed-vault.md).
- Step 0 (prove it by hand) done, except the bridge wallet and the localnet send.
- Built the **`sms` Rust CLI v0.1** (`crates/sms-core` + `crates/sms-cli`): doctor, setup, start, stop, install, logs, devices. Installed with `cargo install --path crates/sms-cli`.
- Tested all commands on the virtual Seeker. A fresh device built from zero by `sms setup` in 112 s (identity + privileged Seed Vault + wallet). Start in 6 s. Details in [08-sms-cli.md](08-sms-cli.md).
- **Next:** test on a fresh machine, prebuilt Seed Vault APK, then the bridge wallet (D10) or the desktop app (v0.2).
- **Test seed loaded automatically:** decoded the simulator's `seed_collection.pb`, `sms setup` now writes the public test seed itself (D15).
- **Seed Vault APK built automatically** from a pinned seed-vault-sdk commit when missing (D16).
- Fresh-user run (empty sms home, no vendor/, new device): **fully hands-off, 172 s**. The wallet authorized the auto-loaded seed, got the expected keys, and signed. Details in [09-auto-seed-vault.md](09-auto-seed-vault.md).
- **Next:** test on a truly fresh Mac/Linux, then the bridge wallet (D10) or the desktop app (v0.2).
- **SMS Test Wallet (bridge wallet) built:** `android/sms-wallet`. MWA outside, Seed Vault inside. Full real-Seeker path tested: fakedapp authorize, sign txn (signature verified by the app), sign msg, decline, scripted approve policy. Added to `sms setup`/`doctor`/`logs` (D17). Details in [10-sms-test-wallet.md](10-sms-test-wallet.md).
- **Git:** repo initialised, first commit `1f727bd` (author thejamesnick).
- **Skin:** drew a Seeker frame to scale (official 73.5×154.8 mm + measurements from official renders and a review photo). Made it pixel-perfect symmetric after feedback (23 px metal + 42 px black on every side, verified by `check-skin.swift`). Found that the emulator's live window scales frame and screen by slightly different factors (±1–2 px).
- **Theme:** dark mode, Solana-purple system palette, Seeker-style wallpaper (via `ThemeActivity`). All applied by `sms setup`.
- **Localnet sign-and-send ✅:** airdrop + sign-and-send through SMS Test Wallet → Seed Vault → Surfpool, confirmed `Status: Ok`. **v0.1 goal reached.**
- Details: [10-sms-test-wallet.md](10-sms-test-wallet.md), [11-seeker-look.md](11-seeker-look.md).
- **Fixed "square corners / status bar looks off"** (user report): the display rectangle was painted over the rounded metal corners. Mask now repaints the body, glass is pure black with underlap, corner radius 9.7 mm so frame and display layers only meet black-on-black (D20). Colors re-sampled from official renders (cool slate metal). Verified live in white and dark apps.
- **Status bar fixed** (user report: too close to the edge / too high): Seeker display overlay with the camera-hole cutout, rounded corners, and status bar height centered on the camera. Clock/icon margins tuned by measurement to ~82 px each side (D21). Added to `sms setup` + `doctor`.
- **Theme matched to a real Seeker** from review photos: light mode, lavender system colors (`#7B6EC6` seed), navy/black/teal wallpaper (D22).
- **Colors refined** (user: "not there yet"): wallpaper redrawn with photo values corrected for the camera's color cast (slate-navy middle, gray-teal bottom). System colors now derived from the wallpaper (Material You) instead of a forced lavender (D23).
- **Home screen like a real Seeker:** grid + dock arranged to match the real layout. New "dApps" (dApp Store spot) and "Wallet" icons/screens. Edge-to-edge and status-bar-icon fixes in the wallet screens (D24).
- **Fixed stray black dot (top-left):** an emulator test overlay (left punch hole) re-enabled itself after a reboot. `sms setup` now deletes all `DisplayCutoutEmulation*` overlays; doctor checks they're gone.
