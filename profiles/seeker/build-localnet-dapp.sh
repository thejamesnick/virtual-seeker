#!/usr/bin/env bash
# Builds the official MWA fakedapp pointed at the local Solana network (Surfpool on the host,
# reached from the device as http://localhost:8899 via `adb reverse`), for sign-and-send tests.
# Patches vendor/mobile-wallet-adapter temporarily, builds, copies the APK, then restores the source.
#   Output: vendor/apks/fakedapp-localnet.apk
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
REPO="$ROOT/vendor/mobile-wallet-adapter/android"
VM="fakedapp/src/main/java/com/solana/mobilewalletadapter/fakedapp/MainViewModel.kt"
MANIFEST="fakedapp/src/main/AndroidManifest.xml"
export JAVA_HOME="${JAVA_HOME:-/Applications/Android Studio.app/Contents/jbr/Contents/Home}"
export ANDROID_HOME="${ANDROID_HOME:-$HOME/Library/Android/sdk}"

cd "$REPO"
restore() { git checkout -q -- "$VM" "$MANIFEST"; }
trap restore EXIT

echo "sdk.dir=$ANDROID_HOME" > local.properties
sed -i '' 's#Uri.parse("https://api.testnet.solana.com")#Uri.parse("http://localhost:8899")#' "$VM"
sed -i '' 's#ProtocolContract.CHAIN_SOLANA_TESTNET#"solana:localnet"#' "$VM"
sed -i '' 's#<application#<application android:usesCleartextTraffic="true"#' "$MANIFEST"
grep -q 'localhost:8899' "$VM" && grep -q 'solana:localnet' "$VM" || { echo "patch failed" >&2; exit 1; }

./gradlew --no-daemon -q :fakedapp:assembleDebug
mkdir -p "$ROOT/vendor/apks"
cp fakedapp/build/outputs/apk/debug/fakedapp-debug.apk "$ROOT/vendor/apks/fakedapp-localnet.apk"
echo "Built $ROOT/vendor/apks/fakedapp-localnet.apk"
