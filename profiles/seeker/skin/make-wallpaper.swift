// Draws the virtual Seeker wallpaper: our own abstract art in the colors of the real Seeker's default
// home screen (measured from review photos): deep navy at the top, near-black middle, blue-teal glow
// at the bottom. No logos. Run: swift make-wallpaper.swift <out.jpg>
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let w = 1200, h = 2670
let out = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "seeker_wallpaper.jpg"
let space = CGColorSpace(name: CGColorSpace.sRGB)!
let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: 0, space: space,
                    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!

func color(_ hex: UInt32, _ a: CGFloat = 1) -> CGColor {
    CGColor(srgbRed: CGFloat((hex >> 16) & 0xff) / 255, green: CGFloat((hex >> 8) & 0xff) / 255,
            blue: CGFloat(hex & 0xff) / 255, alpha: a)
}

// Vertical base (CG origin is bottom-left). Values from a real Seeker home-screen photo, color-corrected
// against the photo's white point (#C0B4E3 -> white): top #01223C navy, upper-middle near black,
// middle #252A3C slate-navy, lower #044558 deep teal-navy, bottom #546D68 gray-teal.
let base = CGGradient(colorsSpace: space,
                      colors: [color(0x546D68), color(0x2F5560), color(0x044558), color(0x252A3C), color(0x0B0D17), color(0x01223C)] as CFArray,
                      locations: [0, 0.08, 0.2, 0.45, 0.7, 1])!
ctx.drawLinearGradient(base, start: CGPoint(x: 0, y: 0), end: CGPoint(x: 0, y: h), options: [])

// Soft glows to give depth (no hard shapes).
func glow(_ x: Double, _ y: Double, _ r: Double, _ hex: UInt32, _ a: CGFloat) {
    let g = CGGradient(colorsSpace: space, colors: [color(hex, a), color(hex, 0)] as CFArray, locations: [0, 1])!
    let c = CGPoint(x: x, y: y)
    ctx.drawRadialGradient(g, startCenter: c, startRadius: 0, endCenter: c, endRadius: r, options: [])
}
glow(600, -250, 1100, 0x5E7A74, 0.35)   // gray-teal light rising from the bottom
glow(600, 1200, 900, 0x2C3048, 0.30)    // slate-navy haze in the middle
glow(300, 2750, 900, 0x0A2C4A, 0.40)    // navy haze, top

// Subtle grain so the gradient doesn't band.
var rng = SystemRandomNumberGenerator()
for _ in 0..<60000 {
    let x = Double.random(in: 0..<Double(w), using: &rng), y = Double.random(in: 0..<Double(h), using: &rng)
    ctx.setFillColor(color(0xFFFFFF, CGFloat.random(in: 0.005...0.02, using: &rng)))
    ctx.fill(CGRect(x: x, y: y, width: 1, height: 1))
}

let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: out) as CFURL, UTType.jpeg.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, ctx.makeImage()!, [kCGImageDestinationLossyCompressionQuality: 0.9] as CFDictionary)
CGImageDestinationFinalize(dest)
print("wrote \(out)")
