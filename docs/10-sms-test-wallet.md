# SMS Test Wallet: the bridge wallet (D10)

Built 2026-10-19. Source: [android/sms-wallet/](../android/sms-wallet/). Package `dev.sms.wallet`.

## Why
On a real Seeker, apps talk to the built-in wallet over **Mobile Wallet Adapter (MWA)**, and that wallet keeps keys in **Seed Vault**, which asks the user to approve with fingerprint/PIN. None of the official test wallets do both (see [doc 07](07-wallet-and-seed-vault.md)). The SMS Test Wallet does both, so the virtual Seeker now runs the **same path as a real Seeker**:

```
your app ──MWA──▶ SMS Test Wallet ──Seed Vault API──▶ Seed Vault (PIN) ──signature──▶ back to your app
```

## What it does
| MWA request | What happens |
|---|---|
| `authorize` | First time: asks Seed Vault to authorize a seed (Seed Vault "Connection request" + PIN). Then shows **"Connect <app>?"** (or follows the policy) and returns the Seed Vault account `m/44'/501'/0'`. |
| `reauthorize` | Accepted. |
| `sign_transactions` | Wallet asks (or follows the policy) → sends the **message bytes** of each transaction to Seed Vault → **Seed Vault "Approve transaction" + PIN** → puts the signature in the right slot → returns signed transactions. Legacy + v0 transactions. |
| `sign_and_send_transactions` | Same, then `sendTransaction` to the chain's RPC (`solana:localnet` → `http://localhost:8899` through `adb reverse`; also devnet/testnet/mainnet). |
| `sign_messages` | Seed Vault signs each message; returns message + signature. |
| Decline anywhere | Correct MWA error back to the app (e.g. `-3 sign request declined`). |

### Approval policy (for scripts and CI)
The wallet's own question can be automated. Seed Vault's PIN is still asked, like on a real Seeker:
```bash
adb shell am start -n dev.sms.wallet/.MainActivity --es policy approve   # or: reject, ask (default)
```

### Wallet screen
Open "SMS Test Wallet" on the virtual Seeker: Seed Vault status, permission, account, policy buttons, and **Connect Seed Vault**.

## Code
| File | Purpose |
|---|---|
| `MwaActivity.kt` | MWA session (walletlib `Scenario`), requests → policy/dialog → Seed Vault intents → results |
| `SeedVaultAccount.kt` | auth token (saved), finds account `m/44'/501'/0'` via Seed Vault's content provider |
| `SolanaTx.kt` | parses legacy/v0 transactions: message bytes, signer slot, inserts signature; Base58 |
| `Rpc.kt` | `sendTransaction` JSON-RPC, chain → URL |
| `Policy.kt` | ask / approve / reject |
| `MainActivity.kt` | status screen ("Wallet" on the home screen, own icon), Connect Seed Vault, policy (also via intent extra) |
| `DappsActivity.kt` | "dApps" home-screen entry in the dApp Store spot: lists installed apps |
| `ThemeActivity.kt` | sets the Seeker wallpaper |

Libraries: `com.solanamobile:mobile-wallet-adapter-walletlib:2.2.0`, `com.solanamobile:seedvault-wallet-sdk:0.4.0`, coroutines. AGP 9.1.1, Gradle 9.7.1, compileSdk 37, minSdk 31.

### Build
```bash
cd android/sms-wallet
echo "sdk.dir=$HOME/Library/Android/sdk" > local.properties
JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home" ./gradlew :app:assembleRelease
# → app/build/outputs/apk/release/app-release.apk (~10 MB, signed with the debug key: test wallet)
```
Build gotchas we hit:
- `walletlib 2.2.0` requires **compileSdk 37**.
- AGP 9's built-in Kotlin compiler is 2.2, but dependencies bring Kotlin 2.4 stdlib → put `kotlin-gradle-plugin:2.4.20` on the buildscript classpath (same as seed-vault-sdk).
- sign-and-send requests use `completeWithInvalidSignatures` (not `completeWithInvalidPayloads`).

## Test results (2026-10-19, virtual Seeker, fakedapp as the app)
| Test | Result |
|---|---|
| Connect Seed Vault (wallet screen) | Seed Vault "Connection request" → PIN → Account `GjJyeC1r2RgkuoCWMyPYkCWSGSGLcz266EaAkLA27AhL`, path `m/44'/501'/0'` ✅ |
| fakedapp **Authorize** | wallet dialog "Connect FakeDApp? … Network: solana:testnet" → Approve → app got `SMS test account` ✅ |
| fakedapp **Sign TXN x1** | wallet dialog → Approve → Seed Vault "Approve transaction" → PIN → `Signed 1 transaction(s) via Seed Vault: 23pc7n56…` → fakedapp: **`Verified memo transaction signature for publicKey GjJyeC1r…`** ✅ |
| fakedapp **Sign MSG x1** | → Seed Vault → PIN → fakedapp verified the off-chain message signature (no error) ✅ |
| **Decline** | fakedapp got `-3/sign request declined` ✅ |
| Policy **approve** via `adb shell am start … --es policy approve` | authorize returned with no dialog; sign went straight to Seed Vault's PIN; signature verified ✅ |

**Fresh device, from zero** (`sms --avd SeekerFresh2 setup`, 108 s): identity → privileged Seed Vault → test seed → SMS Test Wallet. Then the app's very first **Authorize** made the wallet open Seed Vault's **"Connection request: from SMS Test Wallet"** inside the MWA flow → PIN → Approve → **Sign TXN** → Seed Vault PIN → `Verified memo transaction signature for publicKey GjJyeC1r…` ✅ (test device deleted after).

**Localnet sign-and-send ✅** (fakedapp rebuilt for `solana:localnet` with [build-localnet-dapp.sh](../profiles/seeker/build-localnet-dapp.sh)): authorize → Request Airdrop (0.001 SOL from Surfpool) → **Sign and send** → Seed Vault PIN → wallet: `Sent transaction JZQMiaUE…uAjd to http://localhost:8899` → `solana confirm` on Surfpool: **Status: Ok**, fee payer `GjJyeC1r…`, Memo program ran, balance 0.001 → 0.000994999 SOL.

Not tested yet: SIWS (sign-in), v1 transactions (rejected as invalid on purpose).

## In `sms`
- `sms setup` finds the APK (`$SMS_WALLET_APK`, `~/.sms/apks/SmsTestWallet.apk`, or the repo build output), **builds it from `android/sms-wallet` if missing**, installs it, grants the Seed Vault permission, and **disables the generic fakewallet** so there's one MWA wallet, like on a real Seeker (no "choose a wallet" popup).
- `sms doctor`: `✓ Wallet on device  SMS Test Wallet (MWA → Seed Vault)`
- `sms logs`: includes the `SmsWallet` tag.

## Limits
- Seed Vault's PIN still needs a tap (Maestro can do it in CI). Automating Seed Vault itself would make it *less* like a real Seeker, so it stays manual for now.
- The real Seeker's Seed Vault may expect slightly different payloads than the simulator, which signs the raw bytes it is given. Compare against a real Seeker when possible.
