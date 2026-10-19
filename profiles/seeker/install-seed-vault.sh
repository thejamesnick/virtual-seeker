#!/usr/bin/env bash
# Installs the Seed Vault Simulator as a privileged system app (/system/priv-app), like on a real Seeker.
# Needs: emulator started with -writable-system, google_apis image, APK built from vendor/seed-vault-sdk:
#   (cd vendor/seed-vault-sdk && ./gradlew :SeedVaultSimulator:assembleDebug)
set -euo pipefail

export ANDROID_SERIAL="${ANDROID_SERIAL:-emulator-5554}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APK="${1:-$ROOT/vendor/seed-vault-sdk/SeedVaultSimulator/build/outputs/apk/debug/SeedVaultSimulator-debug.apk}"
DEST=/system/priv-app/SeedVaultSimulator

[ -f "$APK" ] || { echo "APK not found: $APK" >&2; exit 1; }

adb root >/dev/null; sleep 2; adb wait-for-device
adb remount | tail -1
adb shell mkdir -p "$DEST"
adb push "$APK" "$DEST/SeedVaultSimulator.apk" | tail -1
adb shell chmod 644 "$DEST/SeedVaultSimulator.apk"

echo "Rebooting to register the system app..."
adb reboot; adb wait-for-device
until [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do sleep 2; done

adb shell dumpsys package com.solanamobile.seedvaultimpl | grep -E "codePath|privateFlags" | head -2
