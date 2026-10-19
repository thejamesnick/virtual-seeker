# Roadmap

Targets are rough, not promises.

## Step 0: Prove it by hand (now)
- [x] Research the platform ([summary](02-research-summary.md))
- [x] Set up the dev Mac (all tools verified) ([dev setup](05-dev-setup.md))
- [x] Create an Android 16 emulator with a Seeker-like profile by hand ([notes](06-virtual-seeker.md))
- [x] Test whether device identity (`Build.MODEL = Seeker`) can be changed: works, no custom image needed
- [x] Build and install the Seed Vault Simulator (privileged) and set up a test seed ([notes](07-wallet-and-seed-vault.md))
- [x] Install a test wallet and run MWA sign + decline flows (testnet dapp; localnet send still open)
- [x] Sign through Seed Vault (authorize seed, derive keys, sign, decline)
- [x] Build the bridge test wallet: MWA + Seed Vault (D10) → SMS Test Wallet ([doc 10](10-sms-test-wallet.md))
- [x] Full localnet sign-and-send: confirmed on Surfpool ([doc 10](10-sms-test-wallet.md))
- [x] Write down every step. That becomes `sms setup` / `sms start`

## v0.1: `sms` CLI ✅ goal reached (2026-10-19)
- [x] `sms doctor`, `sms setup`, `sms start`, `sms stop`, `sms install`, `sms logs` (+ `sms devices`). See [08-sms-cli.md](08-sms-cli.md)
- [x] Virtual Seeker created automatically (quick boot ~6 s; named clean snapshots still to do)
- [x] Surfpool localnet started and connected (`sms start --localnet`)
- [ ] Test on a fresh Mac (no SDK) and on Linux
- [x] Seed Vault Simulator APK built automatically by `sms setup` (pinned source); a published prebuilt is still nice-to-have
- [x] Auto-import the test seed ([doc 09](09-auto-seed-vault.md))
- [ ] Drop the Java requirement (D4)
- **Done when:** a new user with no Android Studio runs `setup` → `start` → installs an app → signs a transaction on the virtual Seeker.

## v0.2: Desktop app
- Tauri app with the phone screen in a Seeker-style frame
- Solana panel: wallet requests, Seed Vault, network, logs
- Real Seeker over USB: mirror, install, logs

## v0.3: Seeker features + compare
- Mock Genesis Token and `.skr` on localnet
- [x] Seeker skin (to scale, symmetric, real colors) + display overlay (camera cutout, status bar) + theme matched to a real Seeker: light, lavender, navy/teal wallpaper ([doc 11](11-seeker-look.md))
- [x] Seeker-like home screen layout: dApps + Wallet in the real spots, same dock ([doc 11](11-seeker-look.md))
- Compare mode (virtual vs real)

## v0.4: Automation / CI
- Scriptable wallet (auto-approve/reject)
- Maestro flows, GitHub Actions template (Linux + KVM)
- `sms tx` transaction inspection

## Later
- Windows / Linux desktop builds
- More device profiles (other Solana Mobile Stack phones, e.g. Ledger Atom)
- Ask Solana Mobile about partnering (official theme/branding, feedback)
