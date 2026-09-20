// Renders the knockout-bolt mark to PNG. Same geometry as icon.svg;
// icon.png is the copy embedded in the binary (include_bytes! in main.rs).
//
//   swift assets/genicon.swift assets/icon.png

import AppKit
import ImageIO

let size = 1024
let cs = CGColorSpace(name: CGColorSpace.sRGB)!
let ctx = CGContext(data: nil, width: size, height: size, bitsPerComponent: 8,
                    bytesPerRow: 0, space: cs,
                    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!

// Flip to top-left origin so design coordinates read naturally.
ctx.translateBy(x: 0, y: CGFloat(size))
ctx.scaleBy(x: 1, y: -1)

// Big Sur icon grid: 824x824 squircle centered on a 1024 canvas, r ~= 185.
let tile = CGRect(x: 100, y: 100, width: 824, height: 824)
let squircle = CGPath(roundedRect: tile, cornerWidth: 185, cornerHeight: 185, transform: nil)
ctx.addPath(squircle)
ctx.setFillColor(CGColor(srgbRed: 0xF1/255.0, green: 0xFF/255.0, blue: 0x0F/255.0, alpha: 1))
ctx.fillPath()

// Knockout bolt, scaled 5.15x from the 160px concept tile, centered.
let bolt: [CGPoint] = [
    CGPoint(x: 573.8, y: 275.1),
    CGPoint(x: 388.4, y: 553.2),
    CGPoint(x: 491.4, y: 553.2),
    CGPoint(x: 439.9, y: 748.9),
    CGPoint(x: 645.9, y: 460.5),
    CGPoint(x: 532.6, y: 460.5),
]
ctx.beginPath()
ctx.addLines(between: bolt)
ctx.closePath()
ctx.setFillColor(CGColor(srgbRed: 0x0D/255.0, green: 0x0D/255.0, blue: 0x10/255.0, alpha: 1))
ctx.fillPath()

let img = ctx.makeImage()!
let out = URL(fileURLWithPath: CommandLine.arguments[1])
let dest = CGImageDestinationCreateWithURL(out as CFURL, "public.png" as CFString, 1, nil)!
CGImageDestinationAddImage(dest, img, nil)
CGImageDestinationFinalize(dest)
print("wrote \(out.path)")
