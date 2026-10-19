# Dev Mac setup

This is exactly what was installed on the development Mac (Apple Silicon, macOS 26.7.1, 16 GB RAM) on **2026-10-18**, and how. Later, `sms setup` will automate the user-facing parts of this.

## Already on the Mac before setup
- Android Studio (`/Applications/Android Studio.app`), used only for its bundled Java
- Android SDK at `~/Library/Android/sdk` (emulator, platform-tools, build-tools 36.0.0, API 37 image)
- Rust 1.99.0, Node 24.21.0, npm 11.19.0, git, Xcode

## 1. Shell PATH (`~/.zshrc`)
Backup: `~/.zshrc.bak-sms`. Added block:
```bash
# --- SMS project: Android + Java + Solana ---
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$HOME/.local/share/solana/install/active_release/bin:$HOME/.maestro/bin:$PATH"
# --- end SMS ---
```
Java comes from Android Studio's bundled JBR (OpenJDK 25.0.3).
The Solana installer also added its PATH line to `~/.profile`.

## 2. Android SDK command-line tools
- Downloaded `commandlinetools-mac-15641748_latest.zip` (145 MB) from `dl.google.com/android/repository/`
- Unzipped to `~/Library/Android/sdk/cmdline-tools/latest`
- `sdkmanager --version` → 21.0 (it prints a harmless `integer expression expected` warning)
- Emulator updated to 37.2.12, adb 1.0.41

## 3. Android 16 packages
```bash
sdkmanager --install "platforms;android-36" "system-images;android-36;google_apis;arm64-v8a" "platform-tools" "emulator"
```
- `google_apis` (not `playstore`), so `adb root` works.
- Seeker runs Android 16 = API 36.

## 4. Solana CLI (Agave stable 4.3.1)
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
```
**Problem:** the macOS binaries are linked to Homebrew's `libusb` (`/opt/homebrew/opt/libusb/lib/libusb-1.0.0.dylib`), and this Mac has no Homebrew.
**Fix (no Homebrew):**
1. Built libusb 1.0.30 from source (github.com/libusb/libusb release tarball) into `~/.local/libusb`:
   ```bash
   ./configure --prefix=$HOME/.local/libusb && make && make install
   ```
2. Re-pointed the 7 Solana binaries that use it (`solana`, `solana-keygen`, `solana-test-validator`, `solana-tokens`, `solana-stake-accounts`, `spl-token`, `agave-ledger-tool`) and re-signed them:
   ```bash
   install_name_tool -change /opt/homebrew/opt/libusb/lib/libusb-1.0.0.dylib \
     $HOME/.local/libusb/lib/libusb-1.0.0.dylib <binary>
   codesign -s - -f <binary>
   ```
⚠️ **After every `agave-install update`, redo step 2**, because new binaries come unpatched.
For `sms`: users will hit this too. `sms doctor` should detect it, or we avoid needing the `solana` CLI at all.

## 5. Surfpool 1.6.1
- Downloaded `surfpool-darwin-arm64.tar.gz` from `github.com/solana-foundation/surfpool/releases` (v1.6.1)
- Installed to `~/.local/bin/surfpool` (no extra libraries needed)

## 6. Wallet APKs
From the MWA GitHub release `@solana-mobile/mobile-wallet-adapter-protocol@3.0.1`:
- `vendor/apks/fakewallet-v1-release.apk` (11 MB)
- `vendor/apks/fakedapp-release.apk` (9.1 MB)

## 7. Source code (shallow clones in `vendor/`)
- `github.com/solana-mobile/seed-vault-sdk`: Seed Vault Simulator source
- `github.com/solana-mobile/mock-mwa-wallet`
- `github.com/solana-mobile/mobile-wallet-adapter`

## 8. Maestro
```bash
curl -Ls "https://get.maestro.mobile.dev" | bash
```
Installs to `~/.maestro/bin`. Installed version: **2.11.0** (needs Java, uses Android Studio JBR via `JAVA_HOME`).

## Not installed (on purpose)
- Homebrew: not needed
- Docker: not needed (Surfpool runs natively)

## Verified (2026-10-18, fresh shell)
| Tool | Version |
|---|---|
| Java (Android Studio JBR) | 25.0.3 |
| sdkmanager / avdmanager | 21.0 (sees `android-36`) |
| emulator | 37.2.12 |
| adb | 1.0.41 |
| solana / solana-keygen / solana-test-validator | 4.3.1 (Agave) |
| surfpool | 1.6.1 |
| maestro | 2.11.0 |
| rustc / node | 1.99.0 / 24.21.0 |
| System image | `system-images;android-36;google_apis;arm64-v8a` |
