# Research summary (2026-10-18)

Full report with sources: [reports/Solana Mobile simulator feasibility.md](../reports/Solana%20Mobile%20simulator%20feasibility.md)
Raw notes: [research_notes/Solana Mobile simulator feasibility/](../research_notes/Solana%20Mobile%20simulator%20feasibility/)

## The platform
- **Seeker runs normal Android**, not a custom OS. It launched on Android 15 and moved to **Android 16** with the MR7 update (~March 2026). The latest found is MR8 (~June 2026).
- Solana Mobile adds a thin layer: the **Seed Vault** system service, the built-in **Seed Vault Wallet**, the built-in **dApp Store**, and on-chain identity (**Seeker Genesis Token**, **`.skr` domains**, **SKR** token).
- The device reports `Build.MODEL "Seeker"`, brand `solanamobile`, manufacturer `Solana Mobile Inc.`. The docs say this can be spoofed.
- Saga reached end of support on 2025-10-21. In March 2026, Solana Mobile opened the Solana Mobile Stack to any Android phone maker. **Ledger** was announced as the first partner (2026-10-15).

## Mobile Wallet Adapter (MWA)
- Pure software protocol. The app sends a `solana-wallet:/v1/associate/local?...` intent, then talks to the wallet over an encrypted local WebSocket. **It works on emulators.**
- Spec 2.0 / 2.1. JS packages are at 3.0.1 (2026-10-09). Many tutorials still show 2.x.
- **fakewallet** (official test wallet): real Ed25519 keys, really submits transactions, has buttons to simulate errors. **Needs manual taps**, with no scripting hook.
- **Mock MWA Wallet**: simulates a Seed Vault-style approval. Built from source only and needs a lock screen.

## Seed Vault
- `seed-vault-sdk` includes an **insecure Seed Vault Simulator** that runs on any Android API 31+ device. This is our base.
- No prebuilt simulator APK is published, so we build it ourselves.
- On a real Seeker, keys live in secure hardware and every signature needs fingerprint or PIN.

## Existing tooling (important)
- Solana Mobile ships an **official `solana-mobile` CLI** on npm (`npx solana-mobile`). It handles emulator create/start, `device install fakewallet`, a `localnet` with Surfpool, and an MWA `playground`.
- It gives you a **generic Android emulator**, not a Seeker. **Nothing on the market gives you a virtual Seeker**, and that is our gap.
- No community project like this exists (searched GitHub; Colosseum and Discord not searched).

## Building blocks
- Apple Silicon needs `arm64-v8a` system images. Use `google_apis` (it allows `adb root`; Play Store images don't).
- **CI must run on Linux with KVM.** GitHub's macOS arm runners can't run emulators.
- From the emulator, the host's localhost is `10.0.2.2`, or use `adb reverse`.
- **Surfpool** is the recommended local validator (`surfpool start`, RPC on 8899).
- Public devnet: 100 requests per 10 s per IP, and the faucet is very limited, so default to a local network.
- Rust: `adb_client` crate (mature) or shelling out to `adb`. `solana-rpc-client` is heavy, so consider plain JSON-RPC.
- **Maestro** for automated taps (wallet approvals in tests).
- **scrcpy** (Apache-2.0) for mirroring a real phone's screen.
