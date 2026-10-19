# SMS — Virtual Seeker

> Working name. Independent community project, not affiliated with or endorsed by Solana Mobile.

**Goal:** a virtual Solana Seeker phone on your computer. If your app works on the virtual Seeker, it should work on a real Seeker.

- **Desktop app** (main product): a Seeker-style phone window with a Solana side panel for wallet requests, Seed Vault, network, mock Genesis Token and transaction logs.
- **`sms` CLI**: the same engine, used for scripts, automation and CI.
- **Real Seeker over USB**: mirror and control a real phone in the same app, and compare it with the virtual one.
- **No Android Studio needed**: `sms setup` downloads everything itself.

## Quick start

```bash
cargo install --path crates/sms-cli   # installs the `sms` command
sms doctor                            # check your machine
sms setup                             # install Android tools + build the virtual Seeker
sms start --localnet                  # open it, with a local Solana network
sms install path/to/your-app.apk
sms logs                              # wallet / Seed Vault / crash logs
sms stop
```

Full CLI docs: [docs/08-sms-cli.md](docs/08-sms-cli.md). Status: **v0.1, works on macOS (Apple Silicon)**, not yet tested on a fresh machine or Linux.

## Docs

| Doc | What's inside |
|---|---|
| [docs/01-vision.md](docs/01-vision.md) | What we're building and why |
| [docs/02-research-summary.md](docs/02-research-summary.md) | Key research findings (full report in [reports/](reports/)) |
| [docs/03-architecture.md](docs/03-architecture.md) | How it works inside |
| [docs/04-roadmap.md](docs/04-roadmap.md) | Build order and milestones |
| [docs/05-dev-setup.md](docs/05-dev-setup.md) | Exactly what's installed on the dev Mac and how |
| [docs/06-virtual-seeker.md](docs/06-virtual-seeker.md) | How the virtual Seeker is built (profile, identity), results and gaps |
| [docs/07-wallet-and-seed-vault.md](docs/07-wallet-and-seed-vault.md) | MWA wallet + Seed Vault tests on the virtual Seeker |
| [docs/08-sms-cli.md](docs/08-sms-cli.md) | The `sms` CLI: commands, files, code layout, test results |
| [docs/09-auto-seed-vault.md](docs/09-auto-seed-vault.md) | Seed Vault built + test seed loaded automatically by `sms setup` |
| [docs/10-sms-test-wallet.md](docs/10-sms-test-wallet.md) | SMS Test Wallet: MWA wallet backed by Seed Vault (the real Seeker path) |
| [docs/decisions.md](docs/decisions.md) | Decision log (what we decided, when and why) |
| [docs/log.md](docs/log.md) | Day-by-day work log |

The original concept doc is [About.txt](About.txt). Some of it has been updated by the decisions in [docs/decisions.md](docs/decisions.md).

## Folders

```
SMS/
├── About.txt            original concept
├── README.md            this file
├── Cargo.toml           Rust workspace
├── crates/sms-core/     engine (SDK, emulator, adb, Seeker identity, setup, doctor)
├── crates/sms-cli/      the `sms` command
├── android/sms-wallet/  SMS Test Wallet (Kotlin): MWA → Seed Vault
├── docs/                project documentation
├── profiles/seeker/     Seeker profile + scripts (create AVD, identity, Seed Vault install, tap helper)
├── reports/             research reports
├── research_notes/      raw research notes with sources
└── vendor/              third-party code and APKs (not ours)
    ├── apks/            fakewallet + fakedapp (MWA 3.0.1 release)
    ├── mobile-wallet-adapter/
    ├── mock-mwa-wallet/
    └── seed-vault-sdk/
```
