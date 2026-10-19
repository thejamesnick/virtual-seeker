# Vision

## The problem
Most developers building for Solana Mobile don't own a Seeker. Today they test on a generic Android emulator with a test wallet. That works for basic wallet flows, but it is **not a Seeker**: different Android version, screen, device identity, no Seed Vault, and no Seeker-only features (Genesis Token, `.skr` names).

## The promise
**If your app works on the virtual Seeker, it works on a real Seeker.**

No emulator can guarantee 100%, so we are honest about it:
- We match everything that can be matched in software.
- We list clearly what can't be matched (see below).
- We **prove** our accuracy by comparing the virtual Seeker against real Seekers (USB compare feature) and fixing every difference.

## What the virtual Seeker matches

| Seeker part | Virtual Seeker |
|---|---|
| Android 16 (API 36) | ✅ same Android version |
| Screen size, shape, density | ✅ custom device profile |
| Device identity (`Build.MODEL = "Seeker"`, brand `solanamobile`) | ✅ planned (may need a custom system image; to be tested) |
| Seed Vault | ✅ official Seed Vault Simulator, pre-set with a test seed |
| Wallet (Mobile Wallet Adapter) | ✅ test wallet using Seed Vault |
| Seeker Genesis Token, `.skr` names, SKR | 🟡 mock versions on a local Solana network |
| Look and feel | 🟡 Seeker-*style* theme (wallpaper, colors, launcher). No copied Solana Mobile logos or artwork |

## What it can't match (always test on a real phone for these)
- Secure hardware key storage (TEE / secure element)
- Hardware attestation and TEEPIN
- Fingerprint/PIN system approval screen (simulated instead)
- Solana Mobile's private built-in apps (Seed Vault Wallet, dApp Store)
- Real performance, camera, sensors, battery

## The product

1. **Desktop app (main product)**
   - Virtual Seeker screen inside a Seeker-style phone frame. Click and type like a real phone.
   - Solana side panel: wallet requests (approve / reject), Seed Vault status, network switch (localnet / devnet / mainnet), mock Genesis Token toggle, transactions and logs.
   - Real Seeker over USB or Wi-Fi: mirror and control the real phone, install apps, see logs, and **compare virtual vs real**.
2. **`sms` CLI**: the same features from the terminal, for scripts and CI.
3. **Easy setup**: one install command, then `sms setup`. No Android Studio and no manual SDK work.

## Who it's for
Solana Mobile app developers, wallet developers, hackathon teams, CI pipelines, educators.
