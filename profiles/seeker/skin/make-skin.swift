// Draws the Seeker skin for the Android emulator: back.png (phone body), mask.png (screen corners +
// camera hole) and layout. Our own drawing, to scale, from public measurements (no Solana Mobile
// photos or logos). Run: swift make-skin.swift <out-dir>
//
// Measurements (see docs/11-seeker-look.md):
//   body 73.5 x 154.8 mm (official spec), display 1200x2670 @ 460 ppi = 66.3 x 147.4 mm
//   corner radius 9.7 mm (render ~10.8, see below), metal band ~0.9 mm + black bezel ~2.7 mm per side, buttons stick out ~0.5 mm
//   shading is symmetric so every side looks identical (no directional light)
//   left side: two volume keys at 53.1-65.3 mm and 68.3-80.5 mm from the top
//   right side: power / Seed Vault fingerprint key at 111.3-129.9 mm from the top
//   front camera: centered punch hole, ~3.7 mm below the top of the display
//   (button positions measured from Solana Mobile's official front render; camera from review photos)
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let pxPerMm: CGFloat = 1200 / 66.3                // display pixels per mm (~18.1)
func mm(_ v: CGFloat) -> CGFloat { (v * pxPerMm).rounded() }

let screenW: CGFloat = 1200, screenH: CGFloat = 2670
let bodyW = mm(73.5)                               // 1330
let bodyH = screenH + (bodyW - screenW)           // 2800 (154.7 mm): identical border on all four sides
let buttonOut = mm(0.5)                           // 9
let padY: CGFloat = 4
let W = bodyW + 2 * buttonOut, H = bodyH + 2 * padY
let body = CGRect(x: buttonOut, y: padY, width: bodyW, height: bodyH)
let screenX = body.minX + ((bodyW - screenW) / 2).rounded()
let screenY = body.minY + ((bodyH - screenH) / 2).rounded()
// Corner radius: the official render measures ~10.8 mm, but at that radius the metal reaches inside the
// display's bounding rectangle. The emulator scales the display layer and the frame layer by slightly
// different factors, so metal drawn there can never line up and leaves a 1-2 px step. With 9.7 mm and a
// 0.9 mm metal band, the glass curve clears the display corner by 3 px and the layers only ever meet
// black-on-black. Constraint: (R - border) * sqrt(2) <= R - rim - 3.
let bodyRadius = mm(9.7)
let rim = mm(0.9)                               // visible metal band (front view)
let screenRadius = bodyRadius - (screenX - body.minX) + 2   // concentric with the body corner
let cameraCenterY = mm(3.7)                                  // in screen coordinates
let cameraR = mm(1.35)

let outDir = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "."
let space = CGColorSpace(name: CGColorSpace.sRGB)!

func context(_ w: CGFloat, _ h: CGFloat) -> CGContext {
    let ctx = CGContext(data: nil, width: Int(w), height: Int(h), bitsPerComponent: 8, bytesPerRow: 0,
                        space: space, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
    ctx.translateBy(x: 0, y: h)   // top-left origin, like the layout file
    ctx.scaleBy(x: 1, y: -1)
    return ctx
}
func color(_ hex: UInt32, _ a: CGFloat = 1) -> CGColor {
    CGColor(srgbRed: CGFloat((hex >> 16) & 0xff) / 255, green: CGFloat((hex >> 8) & 0xff) / 255,
            blue: CGFloat(hex & 0xff) / 255, alpha: a)
}
func rr(_ r: CGRect, _ radius: CGFloat) -> CGPath { CGPath(roundedRect: r, cornerWidth: radius, cornerHeight: radius, transform: nil) }
func gradient(_ stops: [(UInt32, CGFloat)]) -> CGGradient {
    CGGradient(colorsSpace: space, colors: stops.map { color($0.0) } as CFArray, locations: stops.map { $0.1 })!
}
func save(_ ctx: CGContext, _ name: String) {
    let url = URL(fileURLWithPath: outDir).appendingPathComponent(name)
    let dest = CGImageDestinationCreateWithURL(url as CFURL, UTType.png.identifier as CFString, 1, nil)!
    CGImageDestinationAddImage(dest, ctx.makeImage()!, nil)
    CGImageDestinationFinalize(dest)
    print("wrote \(url.path)")
}

// ---- the phone body (frame, keys, glass), drawn into any context ---------------------------------
// Used twice: for back.png (behind the display) and for mask.png (on top of the display corners).
// The Seeker's corners are so round that the metal reaches *inside* the display rectangle; the
// emulator draws the display rectangle over back.png, so the mask has to repaint the frame there,
// otherwise the corners show a black square notch.

let metalKey = gradient([(0x2D373A, 0), (0x8A9A9F, 0.5), (0x2D373A, 1)])
let vaultKey = gradient([(0x1F2425, 0), (0x516066, 0.5), (0x1F2425, 1)])   // power / Seed Vault fingerprint key

// Metal frame, shaded identically on every side: concentric bands from the outer edge inward
// (dark outer lip -> bright polished chamfer -> brushed -> dark inner lip), so no side looks thicker.
// Colors sampled from Solana Mobile's official renders (median / percentiles of frame-edge and body
// pixels): the Seeker's metal is a cool slate with a slight teal tint, not neutral gray.
let bands: [(inset: CGFloat, hex: UInt32)] = [
    (0, 0x1F2425),            // outer lip (edge p25)
    (1.5, 0x3A464A),
    (3.5, 0xB5C4C7),          // polished chamfer highlight (edge p95)
    (6, 0x708187),            // (body p90)
    (9, 0x516066),            // brushed band (edge p75)
    (rim - 4, 0x2D373A),      // (edge median)
    (rim - 1.5, 0x1B2123),    // inner lip against the glass (body p10)
]
let glass = body.insetBy(dx: rim, dy: rim)

func drawBody(_ g: CGContext) {
    // Side keys first; the body covers their inner half.
    func key(_ x: CGFloat, _ fromMm: CGFloat, _ toMm: CGFloat, _ fill: CGGradient) {
        let r = CGRect(x: x, y: body.minY + mm(fromMm), width: buttonOut + 10, height: mm(toMm) - mm(fromMm))
        g.saveGState()
        g.addPath(rr(r, 5)); g.clip()
        g.drawLinearGradient(fill, start: CGPoint(x: r.minX, y: 0), end: CGPoint(x: r.maxX, y: 0), options: [])
        g.restoreGState()
    }
    key(0, 53.1, 65.3, metalKey)                       // volume up (left)
    key(0, 68.3, 80.5, metalKey)                       // volume down (left)
    key(W - buttonOut - 10, 111.3, 129.9, vaultKey)    // power + fingerprint (right)

    for band in bands {
        g.setFillColor(color(band.hex))
        g.addPath(rr(body.insetBy(dx: band.inset, dy: band.inset), bodyRadius - band.inset)); g.fillPath()
    }
    // Black glass. Pure black, the same as the mask's black.
    g.setFillColor(color(0x000000))
    g.addPath(rr(glass, bodyRadius - rim)); g.fillPath()
    // Faint, even glass-edge reflection.
    g.setStrokeColor(color(0xFFFFFF, 0.05)); g.setLineWidth(2)
    g.addPath(rr(glass.insetBy(dx: 1.5, dy: 1.5), bodyRadius - rim - 1.5)); g.strokePath()
    // Earpiece slit in the top bezel.
    g.setFillColor(color(0x1B2123))
    g.addPath(rr(CGRect(x: W / 2 - mm(5), y: body.minY + rim + 6, width: mm(10), height: 5), 2.5)); g.fillPath()
}

// ---- back.png --------------------------------------------------------------------------------
let back = context(W, H)
drawBody(back)
// Transparent display area, inset so the glass runs a few px *under* the display edge: when the
// emulator scales the window it smooths the display's border pixels, which must blend black-on-black.
let underlap: CGFloat = 4
back.setBlendMode(.clear)
back.addPath(rr(CGRect(x: screenX, y: screenY, width: screenW, height: screenH).insetBy(dx: underlap, dy: underlap), screenRadius - underlap)); back.fillPath()
save(back, "back.png")

// ---- mask.png: display-sized, drawn over the display. Opaque = the phone (frame + glass) shows. ---
let mask = context(screenW, screenH)
mask.saveGState()
mask.translateBy(x: -screenX, y: -screenY)   // same coordinates as back.png
drawBody(mask)
mask.restoreGState()
mask.setBlendMode(.clear)
mask.addPath(rr(CGRect(x: 0, y: 0, width: screenW, height: screenH), screenRadius)); mask.fillPath()
mask.setBlendMode(.normal)
let cam = CGRect(x: screenW / 2 - cameraR, y: cameraCenterY - cameraR, width: 2 * cameraR, height: 2 * cameraR)
mask.setFillColor(color(0x000000)); mask.fillEllipse(in: cam)
mask.setStrokeColor(color(0x16181C)); mask.setLineWidth(3); mask.strokeEllipse(in: cam.insetBy(dx: 6, dy: 6))
mask.setFillColor(color(0x1E2A44, 0.9)); mask.fillEllipse(in: CGRect(x: cam.midX - 4, y: cam.midY - 9, width: 7, height: 7)) // lens glint
save(mask, "mask.png")

// ---- layout ----------------------------------------------------------------------------------
let layout = """
parts {
  device {
    display {
      width \(Int(screenW))
      height \(Int(screenH))
      x 0
      y 0
    }
  }
  portrait {
    background {
      image back.png
    }
    foreground {
      mask mask.png
      cutout hole
    }
  }
}
layouts {
  portrait {
    width \(Int(W))
    height \(Int(H))
    event EV_SW:0:1
    part1 {
      name portrait
      x 0
      y 0
    }
    part2 {
      name device
      x \(Int(screenX))
      y \(Int(screenY))
    }
  }
}

"""
try! layout.write(toFile: URL(fileURLWithPath: outDir).appendingPathComponent("layout").path, atomically: true, encoding: .utf8)
print("layout \(Int(W))x\(Int(H)), body \(Int(bodyW))x\(Int(bodyH)), display at \(Int(screenX)),\(Int(screenY)), radii body \(Int(bodyRadius)) screen \(Int(screenRadius))")
