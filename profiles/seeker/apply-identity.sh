#!/usr/bin/env bash
# Makes a running emulator report itself as a Seeker (Build.MODEL / BRAND / MANUFACTURER).
# Needs: a google_apis image (not playstore), emulator started with -writable-system.
# Values come from Solana Mobile docs (device detection via android.os.Build).
set -euo pipefail

export ANDROID_SERIAL="${ANDROID_SERIAL:-emulator-5554}"
MODEL="Seeker"
BRAND="solanamobile"
MANUFACTURER="Solana Mobile Inc."

wait_boot() {
  adb wait-for-device
  until [ "$(adb shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" = "1" ]; do sleep 2; done
}

root_remount() {
  adb root >/dev/null; sleep 2; adb wait-for-device
  adb remount 2>&1 | tail -1
}

# First remount enables overlayfs and needs a reboot before /system is writable.
if ! adb shell 'touch /system/.sms_rw_test && rm /system/.sms_rw_test' 2>/dev/null; then
  root_remount
  echo "Rebooting to enable writable system..."
  adb reboot; wait_boot
fi
root_remount

PROPS="/system/build.prop /vendor/build.prop /product/etc/build.prop /odm/etc/build.prop \
/system_ext/etc/build.prop /vendor_dlkm/etc/build.prop /system_dlkm/etc/build.prop"

for f in $PROPS; do
  adb shell "[ -f $f ] || exit 0
    sed -i -E \
      -e 's/^(ro\.product\.([a-z_]+\.)?model)=.*/\1=$MODEL/' \
      -e 's/^(ro\.product\.([a-z_]+\.)?brand)=.*/\1=$BRAND/' \
      -e 's/^(ro\.product\.([a-z_]+\.)?manufacturer)=.*/\1=$MANUFACTURER/' $f"
done

echo "Rebooting to apply identity..."
adb reboot; wait_boot
adb shell settings put global device_name "$MODEL"
adb shell 'echo "model=$(getprop ro.product.model) brand=$(getprop ro.product.brand) manufacturer=$(getprop ro.product.manufacturer)"'
