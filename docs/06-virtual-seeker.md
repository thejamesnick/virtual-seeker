# Virtual Seeker: build notes (Step 0)

Built by hand on 2026-10-18. Scripts in [profiles/seeker/](../profiles/seeker/).

## Seeker hardware (sources)
| Spec | Value | Source |
|---|---|---|
| Display | 6.36" AMOLED, 120 Hz | [store.solanamobile.com](https://store.solanamobile.com/) |
| Resolution | 1200 × 2670, 460 PPI | reviews (e.g. [madeonsol](https://madeonsol.com/blog/solana-seeker-phone-review), [Decrypt](https://decrypt.co/250628/solana-seeker-preview-everything-we-know-crypto-smartphone)); matches 6.36" mathematically |
| Chip | MediaTek Dimensity 7300 (8 cores) | official store |
| RAM / storage | 8 GB / 128 GB UFS 3.1 | official store |
| Battery | 4,500 mAh, wireless charging | official store |
| Cameras | 108 + 50 + 13 MP rear, 32 MP front | official store |
| Fingerprint | power-button sensor | official store |
| Android | 16 (since MR7 update, ~March 2026) | see [research summary](02-research-summary.md) |

## Step 1: Hardware profile → AVD
Files:
- [profiles/seeker/hardware.ini](../profiles/seeker/hardware.ini): Seeker values for the emulator's `config.ini`
- [profiles/seeker/create-avd.sh](../profiles/seeker/create-avd.sh): creates the `Seeker` AVD and applies the profile

```bash
./profiles/seeker/create-avd.sh
```

| Setting | Virtual Seeker | Real Seeker | Note |
|---|---|---|---|
| Android | 16 (API 36), `google_apis` arm64 | 16 | ✅ |
| Screen | 1200 × 2670 | 1200 × 2670 | ✅ |
| Density | 480 (xxhdpi) | 460 PPI physical. **Android density value unknown** | ⚠️ check on a real Seeker: `adb shell wm density` |
| RAM | 4 GB | 8 GB | lowered so a 16 GB Mac stays usable; configurable |
| Data storage | 16 GB | 128 GB | enough for testing |
| CPU cores | 4 | 8 | |
| Refresh rate | 60 Hz | 120 Hz | emulator limit |

## Step 2: Boot
```bash
emulator -avd Seeker -writable-system -gpu host -no-boot-anim -port 5554
```
- Booted in **~18 s** on the dev Mac (M-series, 16 GB).
- Verified: `wm size` → `1200x2670`, `wm density` → `480`, Android `16`, SDK `36`.
- ⚠️ With `hw.gpu.mode=auto` the GPU fell back to **SwiftShader** (software), which caused freezes later. **Now `hw.gpu.mode=host` / `-gpu host`** (Apple M1 Pro GPU): boot in ~6 s (quick boot). See D11.
- `-writable-system` is **required every launch**, otherwise the identity changes (below) are not loaded.

## Step 3: Device identity → "Seeker"
Seeker's software identity (from Solana Mobile docs): `Build.MODEL = "Seeker"`, `Build.BRAND = "solanamobile"`, `Build.MANUFACTURER = "Solana Mobile Inc."`.

Script: [profiles/seeker/apply-identity.sh](../profiles/seeker/apply-identity.sh)
```bash
./profiles/seeker/apply-identity.sh
```
How it works:
1. `adb root` + `adb remount`. This turns on an overlay over `/system` (needs one reboot the first time).
2. Edits `ro.product.*.model / brand / manufacturer` in every partition's `build.prop` (`/system`, `/vendor`, `/product`, `/odm`, `/system_ext`, `/vendor_dlkm`, `/system_dlkm`).
3. Reboots, then sets the user-visible device name: `settings put global device_name Seeker`.

**Result ✅** (survives reboot):
```
ro.product.model        = Seeker
ro.product.brand        = solanamobile
ro.product.manufacturer = Solana Mobile Inc.
Settings → About → Model: Seeker, Device name: Seeker
```

### What still says "emulator" (known gaps)
| Property | Value now | Why not changed |
|---|---|---|
| `ro.build.fingerprint` | `solanamobile/sdk_gphone64_arm64/emu64a:16/...` | real Seeker fingerprint unknown. Need it from a real device |
| `ro.product.device` / `name` | `emu64a` / `sdk_gphone64_arm64` | real values unknown. Need them from a real device |
| `ro.hardware`, `ro.kernel.qemu` | `ranchu`, `1` | the emulator **needs** these to run. Apps doing "emulator detection" can still tell |
| Settings page title | "About emulated device" | caused by `ro.kernel.qemu`. Cosmetic |

**To do with a real Seeker:** run `adb shell getprop > seeker-props.txt` once and copy the real `ro.product.device`, `ro.product.name`, `ro.build.fingerprint` and `wm density` into the profile.

## Conclusion
Changing device identity **works without a custom system image**, using the stock `google_apis` image + `-writable-system`. Apps that check `Build.MODEL == "Seeker"` (the official way to detect a Seeker) will see a Seeker. This means `sms setup` can do it automatically.
