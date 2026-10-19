// Verifies the skin is symmetric: metal + black bezel widths on every side and corner must match.
// Run: swift check-skin.swift back.png
import CoreGraphics
import Foundation
import ImageIO
let img = CGImageSourceCreateImageAtIndex(CGImageSourceCreateWithURL(URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL, nil)!, 0, nil)!
let w = img.width, h = img.height
let ctx = CGContext(data: nil, width: w, height: h, bitsPerComponent: 8, bytesPerRow: w*4, space: CGColorSpace(name: CGColorSpace.sRGB)!, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
ctx.draw(img, in: CGRect(x: 0, y: 0, width: w, height: h))
let p = ctx.data!.assumingMemoryBound(to: UInt8.self)
func px(_ x: Int, _ y: Int) -> (l: Int, a: Int) { let i = ((h - 1 - y) * w + x) * 4; return ((Int(p[i]) + Int(p[i+1]) + Int(p[i+2])) / 3, Int(p[i+3])) }
// Walk from outside inward along a direction; report: first opaque (frame start), first glass (lum<8), first transparent (screen).
func walk(_ x0: Int, _ y0: Int, _ dx: Int, _ dy: Int) -> (Int, Int, Int) {
    var x = x0, y = y0, step = 0, frame = -1, glass = -1, hole = -1
    while x >= 0 && y >= 0 && x < w && y < h {
        let v = px(x, y)
        if frame < 0 && v.a > 200 { frame = step }
        if frame >= 0 && glass < 0 && v.l < 8 { glass = step }
        if glass >= 0 && v.a < 20 { hole = step; break }
        x += dx; y += dy; step += 1
    }
    return (frame, glass - frame, hole - glass)   // (offset, metal width, black width) in steps
}
let midX = w / 2, midY = h / 2
// Avoid side buttons: measure sides at a y with no buttons (y = 300 and y = h-300)
for (name, r) in [("top", walk(midX, 0, 0, 1)), ("bottom", walk(midX, h-1, 0, -1)),
                  ("left@300", walk(0, 300, 1, 0)), ("right@300", walk(w-1, 300, -1, 0)),
                  ("left@h-300", walk(0, h-300, 1, 0)), ("right@h-300", walk(w-1, h-300, -1, 0)),
                  ("TL diag", walk(0, 0, 1, 1)), ("TR diag", walk(w-1, 0, -1, 1)),
                  ("BL diag", walk(0, h-1, 1, -1)), ("BR diag", walk(w-1, h-1, -1, -1))] {
    print(name.padding(toLength: 12, withPad: " ", startingAt: 0), "edge-offset \(r.0)  metal \(r.1)  black \(r.2)")
}
