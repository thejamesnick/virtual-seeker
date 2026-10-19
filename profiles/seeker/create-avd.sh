#!/usr/bin/env bash
# Creates the "Seeker" virtual device (AVD) from the Seeker hardware profile.
# Manual Step 0 version. Later this becomes part of `sms setup`.
set -euo pipefail

AVD_NAME="${AVD_NAME:-Seeker}"
IMAGE="system-images;android-36;google_apis;arm64-v8a"
HERE="$(cd "$(dirname "$0")" && pwd)"
AVD_DIR="$HOME/.android/avd/$AVD_NAME.avd"

: "${ANDROID_HOME:?ANDROID_HOME not set}"

echo "no" | avdmanager create avd -n "$AVD_NAME" -k "$IMAGE" --force >/dev/null

# Apply the Seeker hardware profile on top of the generated config.
CFG="$AVD_DIR/config.ini"
while IFS='=' read -r key value; do
  [[ -z "$key" || "$key" == \#* ]] && continue
  if grep -q "^$key[ ]*=" "$CFG"; then
    sed -i '' "s|^$key[ ]*=.*|$key=$value|" "$CFG"
  else
    echo "$key=$value" >> "$CFG"
  fi
done < "$HERE/hardware.ini"

echo "Created AVD '$AVD_NAME' at $AVD_DIR"
