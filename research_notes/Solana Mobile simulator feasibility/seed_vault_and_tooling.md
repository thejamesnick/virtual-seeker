# Seed Vault SDK/Simulator and Existing Solana Mobile Developer Tooling (as of 2026-10-18)

> **Headline finding for the `sms` proposal:** Solana Mobile now ships an official CLI, `solana-mobile` (npm), that already has `doctor`, `create`, `emulator *`, `device list/install/open/tune`, `localnet start/forward/check/status/logs/stop`, `playground` (streams MWA interactions to the terminal), `release check/build`, and `webshell`. It was first published 2026-05-22. The latest stable is 0.6.0 (2026-10-09), with 0.7.0 canaries released daily through 2026-10-17. Most of the proposed `sms` command surface (doctor/init/dev/devices/install/logs/stop) **duplicates official tooling**. See section 3.

## 1. Seed Vault SDK: contents, simulator, APIs, limitations, maintenance

### Takeaway
The seed-vault-sdk has four parts: a wallet-facing Android library (`com.solanamobile:seedvault-wallet-sdk:0.4.0`), a Seed Vault **simulator** app (an insecure reference implementation of the Wallet API that runs on any Android API 31+ device), a partial "fake wallet", and JS/React Native bindings. It targets *wallet* developers, not dApp developers. It is maintained mostly through dependency bumps. The last tagged release is v0.4.0 (2025-10-06), and commits continue into Oct 2026.

### Cited Findings
- The README describes the repo as "Wallet APIs, a simulator for the Seed Vault, and related documentation". It is "primarily intended for consumption by developers of Android wallet apps" — [seed-vault-sdk README](https://github.com/solana-mobile/seed-vault-sdk)
- Contents:
  - An integration guide.
  - The `seedvault` Wallet API library, with [JavaDoc](https://solana-mobile.github.io/seed-vault-sdk/seedvault/javadoc/index.html).
  - A simulator (`impl`/SeedVaultSimulator). The README warns that it "makes zero guarantees about security… never be used with any Solana accounts other than test accounts".
  - A `fakewallet` that is "only a partial implementation of a wallet".
  - Source: [seed-vault-sdk README](https://raw.githubusercontent.com/solana-mobile/seed-vault-sdk/main/README.md)
- Gradle coordinate: `com.solanamobile:seedvault-wallet-sdk:0.4.0`. Everything builds with Android Studio. No prebuilt simulator APK install path is documented in the README — [README](https://raw.githubusercontent.com/solana-mobile/seed-vault-sdk/main/README.md)
- The simulator "provides a simple UI for creating seeds via BIP-0039 phrases, and provides a reference implementation of the Wallet API… on any Android device with API version 31 or higher". It was originally intended for use before Saga was available — [integration_guide.md](https://github.com/solana-mobile/seed-vault-sdk/blob/main/docs/integration_guide.md)
- Wallet API surface:
  - Intents `ACTION_AUTHORIZE_SEED_ACCESS` (`Wallet.authorizeSeed`), `ACTION_SIGN_TRANSACTION` (`Wallet.signTransaction`) and `ACTION_GET_PUBLIC_KEY` (`Wallet.requestPublicKey`).
  - A `walletprovider` content provider with the tables `authorizedseeds`, `unauthorizedseeds`, `accounts` and `implementationlimits`.
  - Authorization is granted per seed and per app, and every signing request needs user authorization.
  - Source: [integration_guide.md](https://github.com/solana-mobile/seed-vault-sdk/blob/main/docs/integration_guide.md)
- Derivation:
  - Seeds are BIP-0039 (12 or 24 words), and accounts are derived hierarchically (for example SLIP-0010).
  - Signing requests specify a BIP-0032 derivation path. Private keys are never exposed to wallets.
  - Public keys for `m'/44'/501'/X'` and `m/44'/501'/X'/0'` are pre-derived on authorization. Non-standard paths need the password or biometrics on first use.
  - An implementation must store at least 8 seeds. Limits such as transactions per signing intent come from `implementationlimits`.
  - Source: [integration_guide.md](https://github.com/solana-mobile/seed-vault-sdk/blob/main/docs/integration_guide.md)
- The fetched guide content did not cover an explicit create/import-seed API for wallets. Seed creation and import happen in the simulator's own UI (it gained a "paste seed phrase" menu item in v0.3.2-SNAPSHOT) — [integration guide](https://github.com/solana-mobile/seed-vault-sdk/blob/main/docs/integration_guide.md), [releases](https://github.com/solana-mobile/seed-vault-sdk/releases)
- Real Seed Vault security model:
  - Secrets live only in "privileged environments, separated from the Android OS". Password and biometric input is handled in that same environment — [integration_guide.md](https://github.com/solana-mobile/seed-vault-sdk/blob/main/docs/integration_guide.md)
  - On Seeker it uses the Android Keystore backed by the secure execution environment, and keys never leave it — [docs: Seed Vault](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md)
  - The simulator provides none of this.
- Permissions: v0.2.7 fixed "detection of the privileged protection level for the SEED_VAULT_IMPL permission". v0.3.1 added privileged-wallet-only features (deep link to seed settings, querying backup status) — [releases](https://github.com/solana-mobile/seed-vault-sdk/releases). The exact manifest permission names that wallets must request were not confirmed in the pages fetched.
- The docs tell dApp developers they normally interact with Seed Vault only indirectly via MWA: "If you're building a dApp, use the MWA SDK instead". The SDK "also includes a simulator for testing on standard Android devices" — [docs: Seed Vault](https://docs.solanamobile.com/solana-mobile-stack/seed-vault.md)
- Release history (dates from the GitHub API):
  - v0.4.0, 2025-10-06: compileSdk 36, Kotlin 2.2.20, JS v0.3.4
  - v0.3.3, 2025-08-05: workaround for detecting secure Seed Vault on some devices
  - v0.3.2, 2025-07-28: simulator moved to Hilt, device-specific implementation limits, animated auth dialogs
  - v0.2.10, Apr 2024 and v0.2.9, Mar 2024: React Native library
  - Source: [releases](https://github.com/solana-mobile/seed-vault-sdk/releases)
- Latest commits are 2026-10-14, all Dependabot bumps (for example androidx.annotation 1.11.0). The repo has 102 stars — [seed-vault-sdk commits](https://github.com/solana-mobile/seed-vault-sdk/commits/main)

### Inferences
- The simulator is maintained but not actively evolving. There has been **no tagged release in about 12 months**, and recent activity is dependency hygiene.
- It is a wallet-developer tool. For dApp developers, the realistic "device simulation" layer is an MWA test wallet, not Seed Vault.
- An `sms` that wraps building and installing SeedVaultSimulator plus a wallet onto an emulator, and seeding it from a test mnemonic, is a niche but real gap. Nobody distributes a prebuilt simulator APK, and the official CLI catalog only lists `fakewallet`.
- The simulator cannot emulate TEE/secure-element behaviour, attestation, or Seeker-specific system integration (for example, the Seed Vault Wallet's own UI and Seeker Genesis Token). Any "Seeker simulator" claim must be scoped accordingly.

### Gaps
- Exact install steps for SeedVaultSimulator on an emulator, and whether it needs `-writable-system` or a privileged/system install for `SEED_VAULT_IMPL`. These were not in the pages fetched. (The GrapheneOS "Seedvault" emulator steps that appear in search results are an unrelated Android backup app.)
- No first-party statement found on whether the simulator matches the current Seeker Seed Vault behaviour (MR4–MR10 releases).

## 2. Official docs on developing and testing on an Android emulator

### Takeaway
The official position is that no Solana Mobile device is needed: "for 99% of development and testing" any Android device or emulator works. The recommended wallet is the **Mock MWA Wallet**, built from source. Docs cover Expo/React Native (custom dev build, not Expo Go), Kotlin, Flutter, Unity and Unreal.

### Cited Findings
- Development setup by toolchain:
  - Expo/RN needs Node LTS plus an Expo **custom development build**, not Expo Go.
  - Kotlin needs Android Studio.
  - Flutter follows Flutter's Android setup.
  - A standard Android phone or emulator is "typically enough".
  - Source: [docs: Development Setup](https://docs.solanamobile.com/getting-started/development-setup)
- Wallet install: clone mock-mwa-wallet, open it in Android Studio, run the `fakewallet` configuration and build. Solflare and Jupiter Mobile are alternative real MWA wallets. The doc says the mock wallet has no persistent keypair and resets on exit — [docs: Development Setup](https://docs.solanamobile.com/getting-started/development-setup)
  - This conflicts with the mock-mwa-wallet README, which supports a configurable private key via `local.properties` ([repo](https://github.com/solana-mobile/mock-mwa-wallet)). The docs text may be partly outdated.
- The "Test with any Android device" recipe says Seeker "is an Android phone at its core… Your app will behave the same way on Seeker" as on any other Android device. It recommends the Mock MWA Wallet, which "simulates Seed Vault Wallet functionality". **It warns that the wallet requires a secure lock screen (PIN/pattern/biometric) and fails without one** — [docs recipe](https://docs.solanamobile.com/recipes/general/test-with-any-android-device.md)
- Mock MWA Wallet details:
  - Supports `authorize`, `signIn`, `signAndSendTransactions` and `signMessage`.
  - Bottom-sheet approval, biometric auth, optional `privateKey` and `walletName` in `local.properties`.
  - Press "Authenticate" to enable signing for 15 minutes.
  - **No releases or APKs; build from source.**
  - 18 stars, 6 open issues.
  - Last commit 2026-06-01 ("configurable wallet name").
  - Source: [mock-mwa-wallet](https://github.com/solana-mobile/mock-mwa-wallet)
- Framework docs exist for React Native (Wallet UI, `MobileWalletProvider`/`useMobileWallet`), Kotlin, Flutter, Unity and Unreal. There is also a recipe explaining that MWA is unavailable on iOS — [docs index llms.txt](https://docs.solanamobile.com/llms.txt)
- Third-party guide (QuickNode, updated Jan 2026): an Expo 54 + JDK 17 app on an Android 16 (API 36) AVD with Mock Wallet. You must set a screen PIN so the wallet can approve — [QuickNode guide](https://www.quicknode.com/guides/solana-development/dapps/build-a-solana-mobile-app-on-android-with-react-native)

### Inferences
- The manual steps are well documented but still several in number: install the SDK/AVD, set a lock-screen PIN, build the wallet in Android Studio, configure `adb reverse`, and point the RPC at localnet. The official CLI now automates most of these (section 3).
- Setting the lock-screen PIN is not obviously automated by `emulator tune`, which *disables* the lock screen. This tension is worth verifying.

### Gaps
- Whether `solana-mobile emulator tune` and `fakewallet` installs handle the secure-lock-screen requirement automatically was not confirmed.

## 3. Official templates, scaffolds, and the official `solana-mobile` CLI

### Takeaway
Scaffolding moved to `npx solana-mobile@latest create`, with templates in `solana-mobile/templates` built on create-solana-dapp. Older scaffolds are archived or stale. The official CLI already covers environment checks, emulators, devices, APK installs (including `fakewallet`), localnet with device forwarding, an MWA playground, and release APK checks.

### Cited Findings
- **`solana-mobile` CLI** (repo `solana-mobile/solana-mobile-cli`, Apache-2.0, 91 commits, 4 stars):
  - "CLI for Solana Mobile development". It runs via `npx`, `pnpx` or `bun x solana-mobile@latest`, with no global install.
  - Commands: `create`, `doctor`, `emulator create/delete/list/start/status/stop/tune` and `emulator images install/list/delete`, `device list [--json]/open/tune/install`, `localnet [start|forward|check|status|logs|stop]`, `release check/build`, `templates check/generate/sync`, `webshell init/build`.
  - Source: [solana-mobile-cli README](https://github.com/solana-mobile/solana-mobile-cli)
- npm `solana-mobile` release timeline: created 2026-05-22, latest 0.6.0 (2026-10-09), canary 0.7.0-canary-20261017020241 — [npm registry](https://registry.npmjs.org/solana-mobile)
- `doctor` checks the system, JS tooling, JDK, Android SDK, emulators and physical devices. It reports readiness separately for project creation, Android builds, emulator workflows and device workflows. It is diagnostic only, offers `--json`, and exits 1 on missing required dependencies — [README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)
- `device install`:
  - Installs local APKs or catalog APKs by name. The catalog "currently `fakewallet`, the Mobile Wallet Adapter test wallet".
  - Downloads are cached. `--all` targets every device, and `--grant`/`--downgrade` are supported.
  - `device tune` disables animations, notifications, autofill, Chrome first-run prompts and the lock screen "for automation".
  - `device open` does automatic `adb reverse` for localhost URLs and deep links.
  - Source: [README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)
- `localnet`:
  - Runs surfpool (the default) or solana-test-validator in Docker, with lazy forking from devnet/mainnet via `--network`/`--rpc-url`.
  - Forwards ports 8899/8900/18488 to every device via `adb reverse` and re-applies them on reconnect or reboot.
  - `localnet check` verifies both the host RPC and the device-side tunnel.
  - It attaches to an already-running validator.
  - It flags the cleartext-HTTP (API 28+) pitfall.
  - Source: [README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)
- `playground` serves a bundled wallet-testing page on the device covering connect, SIWS, sign message, sign transaction and sign-and-send. It **streams every MWA interaction result to the terminal**. Supported clusters are devnet, testnet, localnet (with an airdrop button) and mainnet (requires `--url`) — [docs: playground](https://docs.solanamobile.com/cli/playground.md)
- CLI docs pages exist for overview, create, device, doctor, emulator, localnet, playground, templates and webshell. The emulator page was last modified 2026-09-01 — [docs index](https://docs.solanamobile.com/llms.txt), [docs: emulator](https://docs.solanamobile.com/cli/emulator)
- `release check` validates an Expo project and optional APK for the dApp Store. `release build` runs expo prebuild plus the Gradle release build and signs with `apksigner`, with passwords read from env vars — [README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)
- Templates: `solana-mobile/templates` holds "official templates for the Solana Mobile CLI, built on create-solana-dapp". Examples are `expo-kit-uniwind`, `expo-kit-wallet` and `expo-kit-anchor` — [templates repo](https://github.com/solana-mobile/templates), [CLI README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md), [docs: Anchor guide](https://docs.solanamobile.com/llms.txt)
- Legacy and outdated scaffolds:
  - `solana-kotlin-compose-scaffold` is **archived** (last updated 2024-07-15).
  - `solana-mobile-expo-template` was last updated 2025-06-18.
  - `flutter-bottom-sheet` is archived.
  - `solana-mobile-doc-site` is archived (2026-02-05).
  - `react-native-samples` is active (2026-10-08).
  - Source: [solana-mobile org repos](https://github.com/orgs/solana-mobile/repositories)
- The `solana-mobile/solana-mobile-skills` repo holds agent skills (Claude Code, Codex and others) for scaffolding, emulators, toolchain checks, MWA, publishing, Seeker Connect, SGT and .skr domains — [skills README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-skills/main/README.md)
- `seeker-connect` is a newer dApp SDK for connecting directly to a certified device's own wallet over MWA — [docs index](https://docs.solanamobile.com/llms.txt)
- The org also has `gha-free-disk-space-for-android-build-and-test`, a GitHub Action for Android builds and emulator tests (Jan 2025) — [org repos](https://github.com/orgs/solana-mobile/repositories)

### Inferences
- The proposed `sms doctor`/`init`/`devices`/`install`/`stop` commands map almost 1:1 onto `solana-mobile doctor`/`create`/`device list`/`device install`/`emulator stop`/`localnet stop`.
- `sms logs` partly overlaps `localnet logs` and `playground`'s MWA stream.
- `sms tx` overlaps the playground and surfpool Studio.
- Building `sms` as a parallel Rust CLI risks pure duplication of a fast-moving official tool (daily canaries).
- Differentiation would have to come from features the official CLI lacks (section 6). A contribution or plugin to the official CLI might be a better path.

### Gaps
- No visible stars or adoption metrics beyond 4 GitHub stars. npm download counts were not retrieved.
- Contents of the 0.7.0 canaries, which may add more features, were not inspected.

## 4. dApp Store publishing CLI and publisher portal

### Takeaway
The dApp Store CLI is now a thin "portal-backed" uploader. Legacy `init`/`create`/`validate`/`publish` flows were removed. Pre-submission validation lives in `solana-mobile release check`.

### Cited Findings
- `solana-mobile/dapp-publishing`: the only supported entrypoint is `dapp-store --apk-file|--apk-url --whats-new`. It needs a portal API key (`DAPP_STORE_API_KEY`) and a signer keypair. The portal extracts APK metadata and matches the package name. The legacy `init`, `create`, `validate` and `publish submit|update|remove|support` flows are gone — [dapp-publishing](https://github.com/solana-mobile/dapp-publishing)
  - Pre-2026 tutorials that reference `npx dapp-store validate` and similar commands are **outdated**.
- New apps are submitted via the [Publisher Portal](https://publish.solanamobile.com). Docs also cover publishing from Google Play and wrapping web apps (webshell) — [docs index](https://docs.solanamobile.com/llms.txt)

### Inferences
- Publishing validation is covered officially (`release check`). This is not a gap for `sms`.

### Gaps
- What exactly `release check` validates (for example signing key, targetSdk, debuggable flag) was truncated in the README fetch.

## 5. Community simulator-like projects and developer pain points

### Takeaway
I found **no community "Solana Mobile simulator" or MWA CI harness** on GitHub. Community repos are MWA bindings (Capacitor, Unity, RN, a Chrome injector). Pain points centre on the test wallet: building from source, APK distribution, cancellation errors and hangs, lock-screen requirements, and real wallets being slow or crashing on emulators.

### Cited Findings
- GitHub search results:
  - `"solana mobile simulator"` returned zero repos.
  - `"seeker emulator"`, `"mwa e2e"`, `"solana mobile maestro"` and `"solana mobile devtools"` returned nothing relevant.
  - `"seed vault"` returned only the official SDK and unrelated projects.
  - Source: GitHub search via `gh search repos`, 2026-10-18 (no stable URL; see [github.com/search](https://github.com/search?q=solana+mobile+simulator&type=repositories))
- Community MWA tooling found:
  - [capacitor-solana-mobile-wallet-adapter](https://github.com/mstevens843/capacitor-solana-mobile-wallet-adapter) (Apr 2026)
  - [invoke-unity-sdk](https://github.com/plinkdev1/invoke-unity-sdk) (Jun 2026)
  - [solana-mobile-wallet-injector](https://github.com/plinkdev1/solana-mobile-wallet-injector), which injects MWA into Android Chrome pages (Jun 2026)
  - [react-native-solana-mobile-wallet-adapter](https://github.com/nitishxyz/react-native-solana-mobile-wallet-adapter) (Jan 2026)
  - All have 0–2 stars, and none is a dev-environment orchestrator.
- MWA issue #862 (open since 2024-06-12): "Add developer wallet features to fakewallet".
  - The motivation is that Phantom and Solflare "lag or crash on emulators".
  - Requests: persistent keypair, key import, toggling auth, hot-wallet mode, transaction inspector, and an APK download instead of building in Android Studio.
  - The maintainer scoped it down to APK distribution plus key injection and ruled the transaction inspector out of scope.
  - Source: [MWA issue #862](https://github.com/solana-mobile/mobile-wallet-adapter/issues/862)
  - The official CLI's cached `fakewallet` catalog install now addresses APK distribution.
- Other device-specific MWA issues:
  - "Seed Vault Wallet Instability after repeated MWA signing handoffs" (2026-07, closed) — [#1578](https://github.com/solana-mobile/mobile-wallet-adapter/issues/1578)
  - Seeker stock wallet v1.9.0 closing `MWABottomSheetActivity` from TWA/Chrome (2026-04) — [#1458](https://github.com/solana-mobile/mobile-wallet-adapter/issues/1458)
  - Long cooldown after a failed intent disambiguation dialog — [#489](https://github.com/solana-mobile/mobile-wallet-adapter/issues/489)
- Stack Exchange:
  - "How to fix mock-mwa-wallet error when signing in" (2025-12, 0 answers). An Expo app on an emulator with Mock MWA Wallet gets `SolanaMobileWalletAdapterError: java.util.concurrent.CancellationException` — [SE 23878](https://solana.stackexchange.com/questions/23878/how-to-fix-mock-mwa-wallet-error-when-signing-in)
  - "Transaction stuck on loading with Mock MWA Wallet" (2025-07). The bottom sheet opens but signing never completes — [SE 22907](https://solana.stackexchange.com/questions/22907/transaction-stuck-on-loading-with-mock-mwa-wallet-is-there-something-wrong-in)
  - Older examples: "How to get Solana mobile wallet adapter working" (2022, 0 answers) — [SE 642](https://solana.stackexchange.com/questions/642/how-to-get-solana-mobile-wallet-adapter-working); Phantom failing silently with MWA — [SE 4346](https://solana.stackexchange.com/questions/4346/phantom-mobile-app-fails-silently-when-interacting-with-new-mobile-wallet-adapte)
- The MWA adapter only runs in a native Android environment — [Monadical post](https://monadical.com/posts/minting-nft-solana-mobile.html). Expo Go is unsupported, so a custom dev build is required — [docs: Wallet UI](https://docs.solanamobile.com/react-native/wallet_ui_mobile_wallet_adapter)

### Inferences
- The most-reported friction is debugging *why* an MWA session failed (cancellation, hangs, intent and association issues), with little visibility into the protocol.
- The official playground shows results of MWA calls but not session-level traces (association, WebSocket, reauthorize and token state) from the *user's own app*.

### Gaps
- Colosseum hackathon project listings and Discord summaries were not searched within the tool budget. Hackathon "simulator"-like projects may exist there.
- npm search for third-party MWA test packages was not performed.

## 6. Gaps: what remains manual or fragmented (versus the official CLI)

### Takeaway
After the official `solana-mobile` CLI, the remaining gaps are narrower:
- headless and scriptable wallet approval for CI/E2E
- Seed Vault simulator provisioning
- deterministic test identities across emulators
- MWA protocol-level tracing and log capture for the developer's own app
- non-Expo (Kotlin, Flutter, Unity) release and dev flows
- a single-binary, Node-free install

### Cited Findings
- The official CLI's release build and check flow targets "an Expo app" specifically. `webshell` targets web apps — [CLI README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)
- The catalog includes only `fakewallet`. There is no Seed Vault simulator or Mock MWA Wallet entry — [CLI README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)
- The Mock MWA Wallet still requires an Android Studio build, has no APK releases, needs a secure lock screen, and needs a manual "Authenticate" tap with a 15-minute signing window — [mock-mwa-wallet](https://github.com/solana-mobile/mock-mwa-wallet), [docs recipe](https://docs.solanamobile.com/recipes/general/test-with-any-android-device.md)
- The fakewallet transaction inspector and hot-wallet (no-auth) mode were declined or deferred upstream — [issue #862](https://github.com/solana-mobile/mobile-wallet-adapter/issues/862)
- The official CLI requires Node (npx/pnpx/bunx) and Docker for localnet containers — [CLI docs overview](https://docs.solanamobile.com/cli/overview.md), [README](https://raw.githubusercontent.com/solana-mobile/solana-mobile-cli/main/README.md)

### Inferences (candidate genuine value-adds for `sms`)
- **Auto-approving, headless test wallet plus CI harness.** For example, a wallet that auto-approves or rejects according to a policy file. This would enable E2E tests (Maestro, Detox, Appium) of MWA flows in GitHub Actions emulators without UI taps or biometrics. Nothing official or community-built was found for this.
- **MWA session tracing for the user's own app.** `logcat` filtering across the dApp and wallet processes, structured decoding of signed transactions, and correlation with localnet tx logs (`sms tx`). The playground covers only its own test page.
- **Seed Vault simulator provisioning** for wallet developers: build/install the simulator plus the seed-vault fakewallet, and import a test mnemonic non-interactively.
- **Kotlin, Flutter and Unity parity.** The official CLI is Expo-centric.
- **A single static Rust binary.** This is a minor differentiator. It does not justify re-implementing doctor, emulator or localnet, which the official CLI already does well. Wrapping or delegating to `solana-mobile` for those commands would avoid duplication.
- **Risk.** The official CLI is shipping daily canaries, so any `sms` feature could be absorbed upstream. Upstream contribution or an extension may be more durable than a fork.

### Gaps
- Whether the 0.7.0 canaries already add wallet automation or log streaming was not checked.
- Not verified: CI and emulator support in GitHub Actions for MWA flows, and whether `device tune`'s disabled lock screen conflicts with the Mock MWA Wallet's secure-lock requirement.
