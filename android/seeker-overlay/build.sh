#!/usr/bin/env bash
# Builds the Seeker display overlay APK with the Android SDK's aapt2 + apksigner (no Gradle needed).
#   Output: android/seeker-overlay/build/SeekerDisplayOverlay.apk
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
SDK="${ANDROID_HOME:-$HOME/Library/Android/sdk}"
BT="$SDK/build-tools/$(ls "$SDK/build-tools" | sort -V | tail -1)"
JAR="$SDK/platforms/android-36/android.jar"
export JAVA_HOME="${JAVA_HOME:-/Applications/Android Studio.app/Contents/jbr/Contents/Home}"
OUT="$HERE/build"
rm -rf "$OUT" && mkdir -p "$OUT"

"$BT/aapt2" compile --dir "$HERE/res" -o "$OUT/res.zip"
"$BT/aapt2" link -o "$OUT/unsigned.apk" -I "$JAR" --manifest "$HERE/AndroidManifest.xml" \
    --min-sdk-version 31 --target-sdk-version 36 "$OUT/res.zip"
"$BT/zipalign" -f 4 "$OUT/unsigned.apk" "$OUT/aligned.apk"
"$BT/apksigner" sign --ks "$HOME/.android/debug.keystore" --ks-pass pass:android --key-pass pass:android \
    --out "$OUT/SeekerDisplayOverlay.apk" "$OUT/aligned.apk"
echo "Built $OUT/SeekerDisplayOverlay.apk"
