# The Seeker look: skin + theme

Added 2026-10-19. Everything here is **our own drawing**, made to scale from public measurements. No Solana Mobile photos, renders or logos are included (decision D6).

![Virtual Seeker](img-virtual-seeker.png)

## 1. Skin (the phone frame around the emulator screen)
Files: [profiles/seeker/skin/](../profiles/seeker/skin/)
| File | What |
|---|---|
| `make-skin.swift` | draws `back.png` (phone body), `mask.png` (rounded screen corners + camera hole) and `layout` |
| `check-skin.swift` | **verifies symmetry**: measures metal + black bezel on all 4 sides and 4 corners |
| `make-preview.swift` | composites a device screenshot into the skin (for docs) |
| `make-wallpaper.swift` | draws the wallpaper (bundled into the SMS Test Wallet app) |

### Measurements used
| Item | Value | Source |
|---|---|---|
| Body | 73.5 × 154.8 × 9.3 mm | official spec, solanamobile.com/seeker |
| Display | 1200 × 2670 @ 460 ppi = 66.3 × 147.4 mm | official spec |
| Corner radius | ~10.8 mm | measured from Solana Mobile's official front render (pixel scan) |
| Volume keys | **left** side, 53.1–65.3 mm and 68.3–80.5 mm from the top | measured from the official front render |
| Power / **Seed Vault fingerprint key** | **right** side, 111.3–129.9 mm from the top | official render (front) + back render (key next to the "Seed Vault" panel) |
| Front camera | small **centered punch hole** near the top | hands-on review photo (Decrypt) |
| Frame | gray metal, thin even black bezels | official renders + review photo |

Scale: 18.1 px/mm (1200 px = 66.3 mm). Skin: 1348 × 2808, body 1330 × 2800, display at (74, 69).

### Pixel-perfect symmetry (verified)
`swift profiles/seeker/skin/check-skin.swift profiles/seeker/skin/back.png`
```
top          metal 23  black 42
bottom       metal 23  black 42
left         metal 23  black 42
right        metal 23  black 42
TL/TR/BL/BR diagonal: metal 16  black 31   (identical; concentric corners)
```
- The body height was set to `display + (bodyW − displayW)` (154.7 mm instead of 154.8), so every side is **exactly** the same width. The 0.1 mm difference is invisible, while a 1 px difference was visible.
- The frame shading is **symmetric**: concentric bands (dark lip → polished chamfer → brushed gray → inner lip), no directional light. The first version had a top-left light, which made the sides *look* different widths.

### Note about the live emulator window
The emulator shrinks the window to fit the screen, and it scales the frame and the screen by slightly different factors (e.g. 55.98% vs 56.25%). That can create 1–2 px of rounding in the live window. It's inside the emulator, not in the skin. At 100% the skin is exact.

### How it's applied
`sms setup` writes the skin (embedded in the `sms` binary) to `~/.sms/skins/seeker` and sets in the AVD `config.ini`: `skin.name=seeker`, `skin.path=…`, `skin.dynamic=no`, `showDeviceFrame=yes`.

## 2. Theme (inside Android)
Applied by `sms setup` ([theme.rs](../crates/sms-core/src/theme.rs)):
| What | How |
|---|---|
| Dark mode | `cmd uimode night yes` |
| System colors (Material You) seeded with Solana purple `#9945FF` | `settings put secure theme_customization_overlay_packages '{"android.theme.customization.system_palette":"9945FF", …}'` |
| Wallpaper (home + lock) | Android has no shell command for it, so the SMS Test Wallet app has a `ThemeActivity` that sets the bundled wallpaper: `am start -W -n dev.sms.wallet/.ThemeActivity` |

`sms doctor` shows `✓ Theme  Seeker style (dark, Solana purple)` and `✓ Virtual Seeker … Seeker skin`.

## Not done yet
- Home screen layout: a real Seeker shows dApp Store + Wallet on the home screen. Ours still has the stock Google layout (Gmail, Photos, YouTube).
- Boot animation (needs a custom system image).
- The real Seeker's default wallpaper (ours is Seeker-style, not a copy).
