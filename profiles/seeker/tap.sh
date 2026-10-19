#!/usr/bin/env bash
# Dev helper: tap the first on-screen element whose text, content-desc or resource-id matches $1 (case-insensitive).
# Usage: tap.sh "AUTHORIZE" [index]
set -euo pipefail
export ANDROID_SERIAL="${ANDROID_SERIAL:-emulator-5554}"
want="$1"; nth="${2:-1}"

# uiautomator dump sometimes fails ("null root node") while the UI animates; retry with a fresh file
# so we never tap using a stale dump.
xml=""
for _ in 1 2 3 4 5 6; do
  adb shell rm -f /sdcard/ui.xml
  if adb shell uiautomator dump /sdcard/ui.xml 2>/dev/null | grep -q "dumped to"; then
    xml=$(adb shell cat /sdcard/ui.xml)
    break
  fi
  sleep 1
done
[ -z "$xml" ] && { echo "ui dump failed" >&2; exit 1; }

bounds=$(printf '%s' "$xml" | tr '>' '\n' \
  | grep -iE "(text|content-desc|resource-id)=\"$want\"" | sed -n "${nth}p" \
  | grep -oE 'bounds="\[[0-9]+,[0-9]+\]\[[0-9]+,[0-9]+\]"' | grep -oE '[0-9]+' || true)
[ -z "$bounds" ] && { echo "not found: $want" >&2; exit 1; }
set -- $bounds
adb shell input tap $(( ($1 + $3) / 2 )) $(( ($2 + $4) / 2 ))
echo "tapped: $want"
