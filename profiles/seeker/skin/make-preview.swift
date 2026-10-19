// Composites a device screenshot into the Seeker skin (for docs). Run:
//   swift make-preview.swift <skin-dir> <screenshot.png> <out.png>
import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

let a = CommandLine.arguments
let skin = URL(fileURLWithPath: a[1]), shot = URL(fileURLWithPath: a[2]), out = URL(fileURLWithPath: a[3])
func load(_ u: URL) -> CGImage { CGImageSourceCreateImageAtIndex(CGImageSourceCreateWithURL(u as CFURL, nil)!, 0, nil)! }
let back = load(skin.appendingPathComponent("back.png")), mask = load(skin.appendingPathComponent("mask.png")), screen = load(shot)
// Display offset from the layout file ("x N" / "y N" of part2).
let layout = try! String(contentsOf: skin.appendingPathComponent("layout"), encoding: .utf8)
let nums = layout.components(separatedBy: "part2")[1].components(separatedBy: .whitespacesAndNewlines).compactMap { Int($0) }
let (dx, dy) = (CGFloat(nums[0]), CGFloat(nums[1]))
let W = back.width, H = back.height
let ctx = CGContext(data: nil, width: W, height: H, bitsPerComponent: 8, bytesPerRow: 0,
                    space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
let sw = CGFloat(screen.width), sh = CGFloat(screen.height)
let screenRect = CGRect(x: dx, y: CGFloat(H) - dy - sh, width: sw, height: sh) // CG origin bottom-left
// Same order as the emulator: background (back.png), then the display, then the foreground mask.
ctx.draw(back, in: CGRect(x: 0, y: 0, width: W, height: H))
ctx.draw(screen, in: screenRect)
ctx.draw(mask, in: screenRect)
let dest = CGImageDestinationCreateWithURL(out as CFURL, UTType.png.identifier as CFString, 1, nil)!
CGImageDestinationAddImage(dest, ctx.makeImage()!, nil)
CGImageDestinationFinalize(dest)
print("wrote \(out.path)")
