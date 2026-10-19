// Draws the Seeker-style wallpaper (our own abstract art: dark background with soft Solana
// purple/green glows; no logos). Run: swift make-wallpaper.swift <out.jpg>
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

// Near-black base with a slight vertical gradient.
let base = CGGradient(colorsSpace: space, colors: [color(0x0B0A12), color(0x050508)] as CFArray, locations: [0, 1])!
ctx.drawLinearGradient(base, start: CGPoint(x: 0, y: h), end: CGPoint(x: 0, y: 0), options: [])

// Soft glows (CG origin is bottom-left).
func glow(_ x: Double, _ y: Double, _ r: Double, _ hex: UInt32, _ a: CGFloat) {
    let g = CGGradient(colorsSpace: space, colors: [color(hex, a), color(hex, 0)] as CFArray, locations: [0, 1])!
    let c = CGPoint(x: x, y: y)
    ctx.drawRadialGradient(g, startCenter: c, startRadius: 0, endCenter: c, endRadius: r, options: [])
}
glow(150, 2250, 1100, 0x9945FF, 0.55)   // purple, top-left
glow(1100, 500, 1000, 0x14F195, 0.35)   // green, bottom-right
glow(700, 1400, 700, 0x5B3FD9, 0.18)    // faint violet center

// Three thin diagonal light bands for depth.
ctx.setLineCap(.round)
for (i, a) in [0.10, 0.07, 0.05].enumerated() {
    ctx.setStrokeColor(color(0xFFFFFF, a))
    ctx.setLineWidth(CGFloat(2 + i))
    let off = Double(i) * 160
    ctx.move(to: CGPoint(x: -100 + off, y: 900 + off))
    ctx.addLine(to: CGPoint(x: 1300 + off, y: 2300 + off))
    ctx.strokePath()
}

let dest = CGImageDestinationCreateWithURL(URL(fileURLWithPath: out) as CFURL, UTType.jpeg.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, ctx.makeImage()!, [kCGImageDestinationLossyCompressionQuality: 0.88] as CFDictionary)
CGImageDestinationFinalize(dest)
print("wrote \(out)")
