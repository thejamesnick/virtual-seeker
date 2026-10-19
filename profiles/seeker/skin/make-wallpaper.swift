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

// Vertical base (CG origin is bottom-left): teal-blue bottom -> black middle -> navy top.
// Measured: top #0F263E / #001633, middle ~#00000F, bottom #003854 -> #345E72.
let base = CGGradient(colorsSpace: space,
                      colors: [color(0x2E5668), color(0x003854), color(0x02060F), color(0x010309), color(0x001633), color(0x0F263E)] as CFArray,
                      locations: [0, 0.12, 0.38, 0.6, 0.86, 1])!
ctx.drawLinearGradient(base, start: CGPoint(x: 0, y: 0), end: CGPoint(x: 0, y: h), options: [])

// Soft glows to give depth (no hard shapes).
func glow(_ x: Double, _ y: Double, _ r: Double, _ hex: UInt32, _ a: CGFloat) {
    let g = CGGradient(colorsSpace: space, colors: [color(hex, a), color(hex, 0)] as CFArray, locations: [0, 1])!
    let c = CGPoint(x: x, y: y)
    ctx.drawRadialGradient(g, startCenter: c, startRadius: 0, endCenter: c, endRadius: r, options: [])
}
glow(600, -200, 1250, 0x3F7F98, 0.45)   // teal-blue glow rising from the bottom
glow(250, 2700, 900, 0x163A5E, 0.35)    // navy haze, top left
glow(1050, 2500, 700, 0x0C2440, 0.30)   // navy haze, top right

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
