# Solana Mobile Wallet Adapter (MWA): protocol, official implementations, and emulator testing (as of Oct 18, 2026)

> **Most important finding for a "Solana Mobile Simulator" CLI:** Solana Mobile already publishes an official CLI, `solana-mobile` on npm (v0.6.0 stable, published 2026-10-09; v0.7.0 canaries through 2026-10-17). It has these commands: `emulator create/start/stop/tune/images`, `device install fakewallet`, `device tune`, `device open` (handles `adb reverse`), `localnet` (surfpool or solana-test-validator in Docker, forwarded to devices), and `playground` (an MWA test page whose activity streams to the terminal). Together these cover much of the proposed "emulator + mock wallet orchestration" scope. Sources: [CLI overview](https://docs.solanamobile.com/cli/overview), [CLI playground](https://docs.solanamobile.com/cli/playground), [CLI device](https://docs.solanamobile.com/cli/device), [CLI emulator](https://docs.solanamobile.com/cli/emulator), and the npm registry metadata (`npm view solana-mobile`, repo [github.com/solana-mobile/solana-mobile-cli](https://github.com/solana-mobile/solana-mobile-cli)).

## MWA spec: versions, local and remote association, session crypto, methods, SIWS, chain IDs

### Takeaway
The current spec is MWA 2.0. Its changelog also lists 2.1.0, which adds an optional wallet icon. In local association, the dapp fires a `solana-wallet:/v1/associate/local?...` intent. The wallet then opens a WebSocket server on localhost, and the two sides establish a session using P-256 ECDH, HKDF and AES-128-GCM, carrying encrypted JSON-RPC 2.0. In 2.0, `reauthorize` and `sign_transactions` are deprecated: their replacements are `authorize` with an `auth_token`, and `sign_and_send_transactions`. Chains use CAIP-2-style IDs (`solana:mainnet`, `solana:devnet`, `solana:testnet`).

### Cited Findings
**Versions**
- The spec document is version 2.0.0 and uses semver. Its changelog includes 2.1.0, which adds an optional `wallet_icon` to the authorize result. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Version negotiation:
  - The `v=` URI parameter lists the major protocol versions the dapp supports, and the wallet picks the highest one both sides support.
  - If `v` is absent, the session is "legacy" (v1). In a legacy session the wallet must not send session properties. A wallet that doesn't support legacy may close the connection.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- The fakewallet APKs are published as `fakewallet-legacy-{debug,release}.apk` and `fakewallet-v1-{debug,release}.apk`, which lets you test against legacy and v1 protocol wallets. — [GitHub releases API, tag @solana-mobile/mobile-wallet-adapter-protocol@3.0.1](https://github.com/solana-mobile/mobile-wallet-adapter/releases)

**Local association (Android)**
- URI: `solana-wallet:/v1/associate/local?association=<token>&port=<port>&v=<version>`.
  - The association token is a base64url P-256 public key in X9.62 format.
  - The port is random, in the range 49152–65535.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Connection and timeouts:
  - The dapp connects to `ws://localhost:<port>/solana-wallet`.
  - On Android the wallet listens for at least 10 s and accepts only localhost connections.
  - If no wallet answers within 30 s or more, the dapp should show guidance and may fall back to remote association.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Platform scope: local association is specified for Android only, and iOS is deferred to a future version. Desktop should not use local association. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Endpoint-specific URIs: a wallet may return a `wallet_uri_base` (an HTTPS App Link prefix) for future associations, and dapps must reject non-HTTPS prefixes. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)

**Session establishment and encryption**
- WebSocket subprotocols:
  - `com.solana.mobilewalletadapter.v1` uses binary frames.
  - `com.solana.mobilewalletadapter.v1.base64` uses base64 text frames.
  - The wallet sends periodic PINGs.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Handshake:
  - `HELLO_REQ`: the dapp's ephemeral P-256 public key Qd, followed by an ECDSA-SHA256 (P1363) signature over Qd made with the association keypair.
  - `HELLO_RSP`: the wallet's ephemeral key Qw, followed by encrypted session properties `{"v":"<version>"}`.
  - Key derivation: ECDH, then HKDF (RFC 5869), using the 65-byte association public key as salt, producing a 16-byte AES-128 key.
  - The connection closes on invalid keys, if no response arrives within 10 s, or if a duplicate HELLO arrives.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Message wrapping:
  - Algorithm: AES-128-GCM.
  - Frame layout: a 4-byte big-endian sequence number (also used as AAD), a 12-byte IV, the ciphertext, and a 16-byte tag.
  - Each side keeps its own sequence counter starting at 1, and an out-of-order message closes the connection.
  - The plaintext is JSON-RPC 2.0.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)

**Remote association (reflector)**
- URI: `solana-wallet:/v1/associate/remote?association=<token>&reflector=<host>&id=<id>&v=<version>`.
  - The dapp connects to `wss://<host>/reflect`, receives `REFLECTOR_ID`, and then shows the URI as a QR code or via the clipboard.
  - The wallet connects to `wss://<host>/reflect?id=<id>`.
  - The dapp waits at least 30 s and the wallet at least 10 s.
  - Reflector limits: 4 KB frames, 30 s TTL for half-open connections, 90 s for open ones.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- The spec now also describes an optional Nostr transport as an alternative to the reflector. It uses ephemeral kind 20012 events (a provisional number) and a session ID derived from SHA-256 of the association token. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- In `@solana-mobile/wallet-standard-mobile`, passing `remoteHostAuthority` makes desktop connections go through a QR code via a reflector WebSocket server. — [Solana Mobile docs: web installation](https://docs.solanamobile.com/mobile-wallet-adapter/web-installation)
- Wallet support for remote (QR) association is reported inconsistently. An older compatibility table lists Solflare's QR support as "Planned", while a newer page lists Seed Vault, Solflare and Phantom for local mobile web. — [Solana Mobile docs: web apps](https://docs.solanamobile.com/get-started/web/apps); [web installation](https://docs.solanamobile.com/mobile-wallet-adapter/web-installation)
- The fakewallet source includes `ui/scanqr/ScanQRActivity.kt`, which suggests it can scan remote-association QR codes. — [repo tree](https://github.com/solana-mobile/mobile-wallet-adapter/tree/main/android/fakewallet)

**Methods**
- Non-privileged methods:
  - `authorize`. Parameters: `identity{uri, icon, name}`, `chain` (default `solana:mainnet`), `features`, `addresses`, `auth_token`, `sign_in_payload`, and `cluster` (deprecated alias). Result: `auth_token`, `accounts[]` (base64 `address`, `display_address`, `chains`, `features`, `label`, `icon`), `wallet_uri_base`, `wallet_icon`, `sign_in_result`.
  - `deauthorize`.
  - `get_capabilities`. Returns `max_transactions_per_request`, `max_messages_per_request`, `supported_transaction_versions`, and `features`.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Reauthorization: `authorize` with an `auth_token` reauthorizes silently. The `reauthorize` method is deprecated. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Privileged methods:
  - `sign_and_send_transactions`. Options: `min_context_slot`, `commitment`, `skip_preflight`, `max_retries`, `wait_for_commitment_to_send_next_transaction`.
  - `sign_messages`. It must reject payloads that parse as transactions.
  - `clone_authorization`, which is optional.
  - `sign_transactions`, which is deprecated.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- The spec is internally inconsistent about `sign_and_send_transactions`: it is listed as mandatory, but its prose says it is optional. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Feature IDs:
  - Mandatory: `solana:signMessages`, `solana:signAndSendTransaction`.
  - Optional: `solana:signInWithSolana`, `solana:cloneAuthorization`.
  - Deprecated: `solana:signTransactions`.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Error codes:
  - `ERROR_AUTHORIZATION_FAILED` -1
  - `ERROR_INVALID_PAYLOADS` -2
  - `ERROR_NOT_SIGNED` -3
  - `ERROR_NOT_SUBMITTED` -4
  - `ERROR_NOT_CLONED` -5
  - `ERROR_TOO_MANY_PAYLOADS` -6
  - `ERROR_CHAIN_NOT_SUPPORTED` -7
  - `ERROR_ATTEST_ORIGIN_ANDROID` -100
  - JSON-RPC errors -32601 and -32602
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- Dapp identity verification:
  - Native apps are verified with Digital Asset Links (`assetlinks.json`).
  - Web dapps on Android use an attestation flow, signalled with `ERROR_ATTEST_ORIGIN_ANDROID`.
  - No verification is defined for remote association.
  - [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)

**Sign In With Solana**
- The optional `sign_in_payload` on `authorize` returns `sign_in_result{address, signed_message, signature, signature_type}`, with `signature_type` defaulting to `ed25519`.
- Feature ID: `solana:signInWithSolana`.
- An invalid payload produces `INVALID_SIWS_PAYLOAD`.
- If the wallet lacks the feature, the dapp falls back to building the message itself and calling `sign_messages`.
- [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)

**Chain identifiers**
- The chains are `solana:mainnet`, `solana:testnet` and `solana:devnet`. The legacy `cluster` values `mainnet-beta`, `testnet` and `devnet` are still accepted. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- fakewallet also accepts a non-standard localnet chain/cluster, which it maps to `http://localhost:8899`. A source comment says it "reaches a test validator on the host via `adb reverse tcp:8899 tcp:8899`". — [fakewallet MobileWalletAdapterViewModel.kt](https://github.com/solana-mobile/mobile-wallet-adapter/blob/main/android/fakewallet/src/main/java/com/solana/mobilewalletadapter/fakewallet/MobileWalletAdapterViewModel.kt)

### Inferences
- The crypto is standard (P-256, HKDF-SHA256, AES-GCM), so a mock wallet could be reimplemented outside Android, for example as a Node process. However, for the local flow a native dapp still needs an Android activity to handle the `solana-wallet:` intent. A pure-host mock would therefore require either remote association (a reflector) or an on-device stub.
- The protocol has well-defined error codes, so a simulator's "scenario" feature can map directly onto codes -1 through -7.

### Gaps
- I did not find an official public reflector hostname run by Solana Mobile, or any statement about its SLA or uptime. The spec defines only the protocol, and wallet-standard-mobile takes `remoteHostAuthority` as caller-supplied. Remote association should be treated as "bring your own reflector" until this is confirmed.
- The exact date of spec 2.1.0 is not stated on the page I read.

## Repo structure: github.com/solana-mobile/mobile-wallet-adapter, and fakewallet capabilities

### Takeaway
This is an active monorepo (330 stars, last push 2026-10-15) that contains:
- Android libraries: `clientlib`, `clientlib-ktx`, `walletlib`, `common`.
- Sample apps: `fakewallet`, `fakedapp`.
- JS packages: protocol 3.0.1 plus kit and web3js wrappers, `wallet-adapter-mobile` 3.0.0, `wallet-standard-mobile` 0.7.0, and the React Native `walletlib` 1.4.6.

Prebuilt fakewallet APKs are attached to GitHub releases. fakewallet signs with real Ed25519 keys and can actually submit transactions to devnet, testnet, mainnet or localnet RPC. It also has explicit "Simulate ..." buttons for each MWA error path.

### Cited Findings
**Structure**
- `android/` contains `clientlib`, `clientlib-ktx`, `common`, `docs` (including `integration_guide.md`), `fakedapp`, `fakewallet`, `gradle` and `walletlib`. — [repo android/](https://github.com/solana-mobile/mobile-wallet-adapter/tree/main/android); [README](https://github.com/solana-mobile/mobile-wallet-adapter)
- `js/packages/` contains:
  - `mobile-wallet-adapter-protocol`: the reference implementation.
  - `mobile-wallet-adapter-protocol-kit`: the `@solana/kit` wrapper, marked "recommended".
  - `mobile-wallet-adapter-protocol-web3js`: the "legacy" wrapper for web3.js v1.
  - `wallet-adapter-mobile`.
  - The README also lists `examples/example-web-app` and `examples/example-react-native-app`.
  - [README](https://github.com/solana-mobile/mobile-wallet-adapter)
- The README's Gradle snippet references `com.solanamobile:mobile-wallet-adapter-clientlib:2.0.7`. — [README](https://github.com/solana-mobile/mobile-wallet-adapter)
- The README says "All Android projects within this repository can be built using Android Studio" and gives no fakewallet-specific build steps. — [README](https://github.com/solana-mobile/mobile-wallet-adapter)

**Latest releases (verified against the npm registry and GitHub releases on 2026-10-18)**
- `@solana-mobile/mobile-wallet-adapter-protocol`, `-protocol-web3js` and `-protocol-kit` are at 3.0.1 (2026-10-09). The 3.0.0 releases came out on 2026-10-07. — [GitHub releases](https://github.com/solana-mobile/mobile-wallet-adapter/releases); npm registry (`npm view`)
- `@solana-mobile/wallet-standard-mobile` 0.7.0 (2026-10-07), `@solana-mobile/wallet-adapter-mobile` 3.0.0 (2026-10-07), and `@solana-mobile/mobile-wallet-adapter-walletlib` 1.4.6 (2026-10-07). — [GitHub releases](https://github.com/solana-mobile/mobile-wallet-adapter/releases)
- The last repo-wide release with Android notes is v2.2.0 (17 Aug 2026), which included the Android Gradle Plugin 9 and Kotlin/Gradle updates. Android `.aar` files are attached to the JS release assets. — [GitHub releases](https://github.com/solana-mobile/mobile-wallet-adapter/releases)
- Repo activity: `pushed_at` 2026-10-15, 330 stars, 62 open issues and PRs. — [GitHub API repo metadata](https://api.github.com/repos/solana-mobile/mobile-wallet-adapter)
- **Outdated-info caution:** many tutorials and older docs reference protocol 2.x, `wallet-adapter-mobile` 2.x, and `@solana/web3.js` v1. The 3.0 line shipped in October 2026, and its breaking changes were not reviewed in these notes. — [GitHub releases](https://github.com/solana-mobile/mobile-wallet-adapter/releases)

**React Native**
- The protocol JS packages are the React Native client, and the repo includes `example-react-native-app`. React Native setup guides live on docs.solanamobile.com. — [README](https://github.com/solana-mobile/mobile-wallet-adapter)
- For Expo, use a custom development build, not Expo Go. — [Solana Mobile docs: development setup](https://docs.solanamobile.com/developers/development-setup)

**fakewallet: install, keys, signing, devnet**
- Prebuilt APKs are attached to releases, for example `fakewallet-v1-release.apk` and `fakewallet-legacy-release.apk` on the `@solana-mobile/mobile-wallet-adapter-protocol@3.0.1` release. Example download URL: `https://github.com/solana-mobile/mobile-wallet-adapter/releases/download/%40solana-mobile/mobile-wallet-adapter-protocol%403.0.1/fakewallet-v1-release.apk`. — [GitHub releases](https://github.com/solana-mobile/mobile-wallet-adapter/releases)
- The official CLI's `device install fakewallet` catalog entry downloads `fakewallet-v1-release.apk` from GitHub release tag `@solana-mobile/wallet-adapter-mobile@2.3.0` and checks its sha256 (`550055...46e9`). — `solana-mobile@0.6.0` npm tarball, `dist/cli.mjs` (inspected locally); [CLI device docs](https://docs.solanamobile.com/cli/device)
- Keys:
  - Signing uses BouncyCastle `Ed25519Signer` (`SolanaSigningUseCase.kt`).
  - Keys are derived from a BIP39 seed via SLIP-10 (`Bip39UseCase.kt`, `Ed25519Slip10UseCase.kt`).
  - Keys are persisted in a Room DB (`Ed25519KeyDatabase.kt`, `SeedDao.kt`).
  - So fakewallet produces real, valid signatures.
  - [fakewallet source tree](https://github.com/solana-mobile/mobile-wallet-adapter/tree/main/android/fakewallet/src/main/java/com/solana/mobilewalletadapter/fakewallet)
- Sending: `SendTransactionsUseCase.kt` posts JSON-RPC over `HttpURLConnection`. `chainOrClusterToRpcUri` maps:
  - mainnet to `https://api.mainnet-beta.solana.com`
  - devnet to `https://api.devnet.solana.com`
  - testnet to `https://api.testnet.solana.com`
  - localnet to `http://localhost:8899`
  - [MobileWalletAdapterViewModel.kt](https://github.com/solana-mobile/mobile-wallet-adapter/blob/main/android/fakewallet/src/main/java/com/solana/mobilewalletadapter/fakewallet/MobileWalletAdapterViewModel.kt)
- Failure simulation: the UI strings include "Decline", "Simulate cluster not supported", "Simulate sign in not supported", "Simulate auth token invalid", "Simulate first payload invalid", "Simulate too many payloads", "Simulate submitted", "Simulate submit failed", "Simulate internal error", and a "Send transaction to cluster" toggle. There is also a "Connection Failed" screen for failed session establishment. — [fakewallet strings.xml](https://github.com/solana-mobile/mobile-wallet-adapter/blob/main/android/fakewallet/src/main/res/values/strings.xml)
- The fakewallet ViewModel advertises the `FEATURE_ID_SIGN_TRANSACTIONS` and `FEATURE_ID_SIGN_IN_WITH_SOLANA` features. — [MobileWalletAdapterViewModel.kt](https://github.com/solana-mobile/mobile-wallet-adapter/blob/main/android/fakewallet/src/main/java/com/solana/mobilewalletadapter/fakewallet/MobileWalletAdapterViewModel.kt)
- Emulators: the official docs say you can test "on any Android device or emulator". For a manual fakewallet build, open `android/build.gradle` in Android Studio, select the `fakewallet` run configuration, and run it. — [Solana Mobile docs: development setup](https://docs.solanamobile.com/developers/development-setup); [Solana dev playbook (secondary)](https://siliconninjaa.substack.com/p/building-mobile-dapps-with-solana)

### Inferences
- Failure simulation in fakewallet is manual: a human taps a "Simulate X" button. I found no intent extra or ADB broadcast for scripting a scenario. For CI, a simulator CLI would need UI automation (UIAutomator/Espresso/Maestro tapping those labels), a fork that adds a scripted mode, or its own mock wallet.
- fakewallet hard-codes public RPC endpoints, so devnet tests depend on api.devnet.solana.com rate limits. Localnet via `adb reverse tcp:8899` avoids that dependency.

### Gaps
- I did not confirm whether fakewallet's seed persists across reinstalls, or whether it can import a fixed key (as opposed to mock-mwa-wallet's `local.properties`).
- I found no documentation of a headless or automation hook in fakewallet.

## Other mock/test wallets and real wallets on emulators

### Takeaway
Solana Mobile now recommends **Mock MWA Wallet** ([github.com/solana-mobile/mock-mwa-wallet](https://github.com/solana-mobile/mock-mwa-wallet)) for testing. It simulates Seed Vault Wallet, uses bottom-sheet approval, allows key import, and requires a secure lock screen. fakewallet remains the CLI's catalog wallet. I found no official guidance on running Phantom, Solflare or Backpack on emulators.

### Cited Findings
- Mock MWA Wallet:
  - Officially "recommended" for testing MWA apps, and it "simulates Seed Vault Wallet functionality".
  - Supports `authorize`, `signIn`, `signAndSendTransactions` and `signMessage`.
  - Approval happens in a bottom sheet with no app switch.
  - Supports biometric auth and configurable Ed25519 private key import.
  - [Solana Mobile docs: test with any Android device](https://docs.solanamobile.com/recipes/general/test-with-any-android-device)
- "Mock MWA Wallet requires a secure lock screen", so set a PIN, pattern or biometric on the emulator first. — [Solana Mobile docs: test with any Android device](https://docs.solanamobile.com/recipes/general/test-with-any-android-device)
- Repo details:
  - Pressing "Authenticate" enables signing for 15 minutes.
  - `privateKey` (base58 or base64) and `walletName` (default `mwallet`) are set via `local.properties`. Without a key it generates a random keypair.
  - On the emulator, set a fingerprint or PIN in Settings, and simulate a fingerprint touch from Android Studio's emulator extended controls.
  - Repo stats: Apache-2.0, 75 commits, 18 stars, 6 open issues. Last commit 2026-06-01.
  - No releases are published, so you build it from source in Android Studio.
  - [mock-mwa-wallet repo](https://github.com/solana-mobile/mock-mwa-wallet); [GitHub commits API](https://api.github.com/repos/solana-mobile/mock-mwa-wallet/commits)
- The development-setup page says the mock wallet "can simulate connection or signing errors", "does not store a persistent keypair" and resets each time the app is exited. That wording may describe an older fakewallet-based setup, because the repo now documents key import. — [Solana Mobile docs: development setup](https://docs.solanamobile.com/developers/development-setup)
- A third-party QuickNode guide (updated January 2026) uses Mock MWA Wallet on an Android 16 (API 36.1) AVD with JDK 17, and notes that a screen lock PIN is needed on the emulator. — [QuickNode guide](https://www.quicknode.com/guides/solana-development/dapps/build-a-solana-mobile-app-on-android-with-react-native)
- Real wallets: the official docs list Solflare and Jupiter Mobile as MWA-compatible wallets to test with, without saying whether they work on emulators. The web docs list Seed Vault Wallet, Solflare and Phantom for Android mobile web. — [development setup](https://docs.solanamobile.com/developers/development-setup); [web apps](https://docs.solanamobile.com/get-started/web/apps)
- The CLI's `emulator images install` examples use `google_apis_playstore` system images (for example `system-images/android-36.1/google_apis_playstore/arm64-v8a`), which would allow Play Store installs of real wallets. — `solana-mobile@0.6.0` README (npm tarball)

### Inferences
- In principle, Play Store emulator images let you install Phantom or Solflare by signing in to a Google account. That cannot be automated without credentials, and some wallets may block emulators or rooted devices. Treat it as a manual, optional path.
- A simulator CLI could offer two wallet backends: fakewallet (prebuilt APK, full error simulation) and mock-mwa-wallet (closer to Seed Vault UX, but it needs a Gradle build and a lock screen). Note that `device tune lockscreen-off` conflicts with mock-mwa-wallet's lock-screen requirement.

### Gaps
- I found no first-party source confirming or denying that Phantom, Solflare or Backpack work on the Android emulator, or whether they detect emulators.
- I found no source on whether Backpack currently supports MWA.

## Web dApps: mobile Chrome, wallet-standard-mobile, and desktop limits

### Takeaway
On Android, MWA works only in Chrome (including Chrome PWAs), through `@solana-mobile/wallet-standard-mobile`. It does not work in Firefox, Brave or Opera, or on iOS. Desktop browsers can connect only through remote/QR association, and only if the app supplies a reflector via `remoteHostAuthority`.

### Cited Findings
- Android Chrome is the "Primary Android browser; also works with Chrome PWAs". "Firefox, Opera, Brave, etc. do not support MWA", and "MWA is not available on any iOS browser". — [Solana Mobile docs: web apps](https://docs.solanamobile.com/get-started/web/apps)
- `wallet-standard-mobile` registers MWA as a wallet-standard wallet and works with `@anza/wallet-adapter` apps. On mobile it uses local association via intents. On desktop it uses QR remote association when `remoteHostAuthority` is set. — [web apps](https://docs.solanamobile.com/get-started/web/apps); [web installation](https://docs.solanamobile.com/mobile-wallet-adapter/web-installation)
- The official `solana-mobile playground` command:
  - Serves a bundled test page at `http://localhost:4747` on the device (via `adb reverse`) and opens it in the device browser.
  - Covers connect, SIWS, sign message, sign transaction, and sign-and-send.
  - Streams results to the terminal and marks rejections with ✖.
  - Defaults to devnet. `--cluster` accepts localnet, mainnet (requires `--url`, because the public RPC returns 403 for browser-origin requests) and testnet.
  - Example output shows target `emulator-5554`.
  - [CLI playground](https://docs.solanamobile.com/cli/playground)
- Web dapps on Android go through the spec's origin attestation flow (`ERROR_ATTEST_ORIGIN_ANDROID` -100). — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)

### Inferences
- When testing a web dApp, a simulator must drive Chrome on the emulator; WebView-based or other browsers won't work. The CLI's `device tune chrome-quiet` addresses Chrome first-run prompts.

### Gaps
- I did not verify whether the attestation flow requires a specific Chrome version, or how fakewallet handles web origins.

## Known pitfalls: emulator, Android 14+, iOS

### Takeaway
The documented pitfalls are:
- Mock MWA Wallet needs a lock screen.
- Expo Go is unsupported.
- Only Chrome works for web dApps.
- iOS is unsupported in the spec.
- Public mainnet RPC blocks browser-origin requests.
- Localhost RPC requires `adb reverse`.

I found no authoritative source on an MWA-specific Android 14+ background-activity-launch problem.

### Cited Findings
- iOS is deferred in the spec, and no iOS browser supports MWA. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html); [web apps](https://docs.solanamobile.com/get-started/web/apps)
- A secure lock screen is required for Mock MWA Wallet. — [test with any Android device](https://docs.solanamobile.com/recipes/general/test-with-any-android-device)
- Expo projects need a custom dev build, not Expo Go. — [development setup](https://docs.solanamobile.com/developers/development-setup)
- Localnet from an emulator needs `adb reverse tcp:8899 tcp:8899` (per the fakewallet comment). The CLI's `localnet` command runs surfpool or solana-test-validator in Docker and forwards it to every connected device. — [fakewallet ViewModel](https://github.com/solana-mobile/mobile-wallet-adapter/blob/main/android/fakewallet/src/main/java/com/solana/mobilewalletadapter/fakewallet/MobileWalletAdapterViewModel.kt); `solana-mobile@0.6.0` README
- Historical issue #94 (closed): triggering an association activity while the previous one is still concluding silently fails. This is relevant to rapid automated test loops. — [issue #94](https://github.com/solana-mobile/mobile-wallet-adapter/issues/94)
- Issue #1578 (closed) reported "Seed Vault Wallet Instability after repeated MWA signing handoffs". — [issue #1578](https://github.com/solana-mobile/mobile-wallet-adapter/issues/1578)
- The local-association timeouts (wallet ≥10 s, dapp ≥30 s, HELLO 10 s) can be tripped by slow emulators or cold starts. — [MWA spec](https://solana-mobile.github.io/mobile-wallet-adapter/spec/spec.html)
- The CLI's `device tune` / `emulator tune` tweaks (animations-off, autofill-off, chrome-quiet, lockscreen-off, screen-awake, notifications-off and others) exist specifically to "reduce interference with automation". — [CLI device](https://docs.solanamobile.com/cli/device); [CLI emulator](https://docs.solanamobile.com/cli/emulator)

### Inferences
- Android 14 tightened background activity launches. MWA's flow is a foreground dapp starting the wallet activity, then the wallet finishing back to the dapp, which is generally allowed. Any problems would most likely show up in headless or automated launches (for example `am start` from ADB while the screen is locked). This is unverified.

### Gaps
- I found no first-party source on Android 14/15/16 background-activity-launch restrictions affecting MWA. A GitHub issue search for "background activity" returned unrelated results.
- I found no source on emulator CPU architecture (arm64 vs x86_64) compatibility for the fakewallet APKs.
