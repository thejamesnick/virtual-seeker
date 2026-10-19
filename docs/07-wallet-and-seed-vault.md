# Wallet (MWA) and Seed Vault on the virtual Seeker (Step 0)

Tested 2026-10-19 on the `Seeker` AVD (see [06-virtual-seeker.md](06-virtual-seeker.md)).

## Key finding: no existing test wallet does both MWA and Seed Vault
| Wallet | Talks MWA to apps? | Keys in Seed Vault? |
|---|---|---|
| MWA `fakewallet` (mobile-wallet-adapter repo) | ✅ | ❌ own keys (BIP39 code copied from the Seed Vault simulator) |
| Seed Vault `fakewallet` (seed-vault-sdk repo) | ❌ | ✅ |
| `mock-mwa-wallet` | ✅ | ❌ (simulates a Seed Vault-*style* approval, not the real API) |
| **Real Seeker: Seed Vault Wallet** | ✅ | ✅ |

➡️ **To match a real Seeker we need our own "bridge" test wallet**: MWA (`walletlib`) outside, the Seed Vault Wallet API inside. Recorded as decision D10.

## Test A: MWA flow (app → wallet) ✅

Setup:
```bash
surfpool start --offline --no-tui          # local Solana network, RPC :8899, WS :8900
adb install -r vendor/apks/fakewallet-v1-release.apk
adb install -r vendor/apks/fakedapp-release.apk
adb reverse tcp:8899 tcp:8899 && adb reverse tcp:8900 tcp:8900
```
Taps were automated with [profiles/seeker/tap.sh](../profiles/seeker/tap.sh) (finds a button by its text through `uiautomator dump`, then `input tap`).

| Step | Result |
|---|---|
| fakedapp → **Authorize** | fakewallet sheet opened (cluster `testnet`, dapp identity "Verification failed" is expected for fakedapp) → tapped AUTHORIZE → app got auth token, account `fakewallet account 0`, protocol **MWA 2.0** ✅ |
| **Sign TXN x1** → AUTHORIZE | logcat: `Verified memo transaction signature for publicKey(base58)=8tUWYzF5gZDYUm28GshMquoBa5XYG8U7z3u4XxiFPghY` ✅ |
| **Sign MSG x1** → DECLINE | app got `JsonRpc20RemoteException: -3/sign request declined` ✅ (correct MWA error code) |

Useful logcat tags: `MobileWalletAdapterSessionCommon`, `JsonRpc20Client`, `JsonRpc20Server`, `LocalWebSocketServer`, `LocalAssociationScenario`, `MainViewModel`. These are what `sms logs` / the desktop app's wallet panel should parse.

### Network notes
- `fakedapp` is **hard-coded to testnet** (`MainViewModel.kt`: `CLUSTER_RPC_URI = https://api.testnet.solana.com`). It can't be pointed at localnet without rebuilding.
- `fakewallet` supports `localnet` → `http://localhost:8899` (reached through `adb reverse`).
- So a full localnet "sign **and send**" test needs a dapp that requests the `localnet` chain. Our own test dapp, or the developer's app.
- Surfpool 1.6.1 started in ~2 s with `--offline` (`getVersion` → `surfnet-version 1.6.1, solana-core 4.2.1`).

## Test B: Seed Vault Simulator ✅

### Build
```bash
cd vendor/seed-vault-sdk
echo "sdk.dir=$HOME/Library/Android/sdk" > local.properties
./gradlew :SeedVaultSimulator:assembleDebug   # → SeedVaultSimulator-debug.apk (43 MB)
./gradlew :fakewallet:assembleDebug           # → fakewallet-Generic-debug.apk (42 MB), also a Privileged flavor
```
The first build took ~5 min on a slow connection (Gradle 9.7.1 + dependencies). Gradle 9.7 runs fine on Java 25 (Android Studio JBR).

Facts from the source:
- Package `com.solanamobile.seedvaultimpl`, **minSdk 33**, compileSdk 37. Signed with a test key in `testkeystore/` (public test passwords).
- Permissions: `ACCESS_SEED_VAULT` (dangerous, for normal wallets) and `ACCESS_SEED_VAULT_PRIVILEGED` (`knownSigner|privileged|signature`).
- On a real device Seed Vault is a **privileged system app**, and intent priority 100 only works when it's installed as privileged.

### Install as a privileged system app (like a real Seeker)
Script: [profiles/seeker/install-seed-vault.sh](../profiles/seeker/install-seed-vault.sh). It pushes the APK to `/system/priv-app/SeedVaultSimulator/` and reboots.
Result: `codePath=/system/priv-app/SeedVaultSimulator`, flags **SYSTEM + PRIVILEGED** ✅

### Test seed
Imported in the simulator UI (Import → name, PIN, 12 words → Save):
- Name `SMS-test-seed`, PIN `1234`
- Phrase: the public BIP-39 test vector `abandon ×11 + about`. **Test only. Everyone knows this phrase, never put real funds on it.**

Typing it via `adb shell input text` worked, but the keyboard's autocomplete corrupted two words ("abando", "abandonmy"). For `sms setup` we need a reliable way: pre-seed the simulator's data store directly, or turn off keyboard suggestions first.

### Wallet ↔ Seed Vault flow (Seed Vault `fakewallet`, Generic flavor)
```bash
adb install -r fakewallet-Generic-debug.apk
adb shell pm grant com.solanamobile.fakewallet com.solanamobile.seedvault.ACCESS_SEED_VAULT
```
| Step | Result |
|---|---|
| Wallet → "Authorize another seed" | Seed Vault bottom sheet **"Connection request: FakeWalletGeneric → SMS-test-seed"** → Use password → PIN → ✅ |
| Accounts derived by Seed Vault | `m/44'/501'/0'` → `GjJyeC1r2RgkuoCWMyPYkCWSGSGLcz266EaAkLA27AhL`, `m/44'/501'/1'` → `ANf3TEKFL6jPWjzkndo4CbnNdUNkBk4KHPggJs2nu8Xi`. **Both match `solana-keygen`** for the same phrase/paths ✅ |
| Sign transaction (txn V0) | Seed Vault sheet "Approve transaction" → PIN → `Transaction signed … mResolvedDerivationPaths=[bip32:/m/44'/501'/0']` ✅ |
| Sign message → **Don't allow** | `Wallet$ActionFailedException: signMessages failed with result=0` (cancelled) ✅ |

Check a derivation yourself (needs a terminal; `expect` used for automation):
```bash
solana-keygen pubkey "prompt://?full-path=m/44'/501'/0'"
```

### Problems hit and fixes
1. **Freezes ("Seed Vault Simulator isn't responding")** with the emulator's default software GPU (SwiftShader). The whole system was slow (4 s per tap). **Fix: `-gpu host`** (uses the Mac GPU). After that: boot in 6 s and no freezes. Now the default in [hardware.ini](../profiles/seeker/hardware.ini) (`hw.gpu.mode=host`). See D11.
2. **Stale UI dump in `tap.sh`**: when `uiautomator dump` failed, the script tapped using the *old* screen and hit wrong buttons. Fixed: it retries with a fresh dump and never reuses an old one.
3. After force-stopping apps mid-request, the wallet lost its view of the authorization. Reinstalling the wallet fixed it.

## Summary
| Piece | Works on virtual Seeker? |
|---|---|
| MWA app ↔ wallet (authorize, sign, decline) | ✅ |
| Seed Vault Simulator as privileged system app | ✅ |
| Wallet ↔ Seed Vault (authorize seed, derive keys, sign, decline) | ✅ |
| **App → MWA → wallet → Seed Vault (the real Seeker path)** | ❌ needs our bridge wallet (D10) |
| Sign *and send* to local Surfpool | ⏳ needs a dapp requesting `localnet` |
