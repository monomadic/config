import ArgumentParser
import AVFoundation
import CoreImage
import CoreImage.CIFilterBuiltins
import CoreML
import Vision

// avremove — per-frame (non-temporal) video object removal on Apple silicon.
//
//   mask source (pick one):
//     --point X,Y      Vision foreground-instance segmentation at that pixel on
//                      frame 0, then VNTrackObjectRequest follows it and we
//                      re-segment inside the tracked box every frame.
//     --mask file.png  static mask (white = remove), e.g. a burned-in logo.
//
//   inpaint: LaMa via Core ML (see convert_lama.py). We crop a square window
//   around the mask bbox, run the model at its native size, and paste back —
//   so the rest of the frame is never resampled.
//
// Deterministic model + stable mask = far less flicker than generative
// per-frame fill. It is still not temporally coherent; for that use ProPainter.

@main
struct AVRemove: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Remove an object from every frame with Vision masks + LaMa (Core ML).")

    @Argument(help: "Input video.") var input: String
    @Argument(help: "Output video.") var output: String

    @Option(name: .long, help: "Path to LaMa .mlpackage or compiled .mlmodelc")
    var model: String = "LaMa512.mlpackage"

    @Option(name: .long, help: "Pixel coordinate 'x,y' of the object to remove (frame 0).")
    var point: String?

    @Option(name: .long, help: "Static mask PNG (white = remove).")
    var mask: String?

    @Option(name: .long, help: "Grow the mask by N px before inpainting (hides halo).")
    var dilate: Int = 12

    @Option(name: .long, help: "Feather the paste-back edge in px.")
    var feather: Int = 6

    @Option(name: .long, help: "Extra context around the mask bbox when cropping, as a fraction of bbox size.")
    var margin: Double = 0.6

    @Option(name: .shortAndLong) var codec: OutputCodec = .prores422
    @Option(name: .long) var mbps: Double?

    @Flag(name: .long, help: "Write the mask (as red overlay) instead of inpainting — for checking tracking.")
    var debugMask = false

    func run() async throws {
        guard (point == nil) != (mask == nil) else { throw CLIError("give exactly one of --point or --mask") }
        let inURL = URL(fileURLWithPath: input), outURL = URL(fileURLWithPath: output)
        let ci = CIContext(options: [.cacheIntermediates: false, .useSoftwareRenderer: false])

        // --- model ---
        let mlURL = URL(fileURLWithPath: model)
        let compiled = mlURL.pathExtension == "mlmodelc" ? mlURL : try await MLModel.compileModel(at: mlURL)
        let cfg = MLModelConfiguration(); cfg.computeUnits = .all
        let lama = try MLModel(contentsOf: compiled, configuration: cfg)
        guard let imgDesc = lama.modelDescription.inputDescriptionsByName["image"]?.imageConstraint
        else { throw CLIError("model has no image input named 'image' — convert with convert_lama.py") }
        let tile = imgDesc.pixelsWide
        FileHandle.standardError.write("model tile \(tile)x\(tile)\n".data(using: .utf8)!)

        // --- io ---
        let fmt = kCVPixelFormatType_32BGRA
        let reader = try await VideoReader(url: inURL, pixelFormat: fmt)
        let W = reader.info.width, H = reader.info.height
        let writer = try VideoWriter(url: outURL, width: W, height: H, fps: reader.info.fps,
                                     transform: reader.info.transform, codec: codec,
                                     pixelFormat: fmt, bitrateMbps: mbps,
                                     colorProperties: reader.info.colorProperties)
        let outPool = try BufferPool(attributes: [:], width: W, height: H, pixelFormat: fmt)
        let tilePool = try BufferPool(attributes: [:], width: tile, height: tile, pixelFormat: fmt)
        let maskTilePool = try BufferPool(attributes: [:], width: tile, height: tile, pixelFormat: kCVPixelFormatType_OneComponent8)

        // --- mask source ---
        var staticMask: CIImage?
        if let m = mask {
            guard let img = CIImage(contentsOf: URL(fileURLWithPath: m)) else { throw CLIError("cannot read mask \(m)") }
            staticMask = img.applyingFilter("CIColorControls", parameters: [kCIInputSaturationKey: 0])
                .cropped(to: CGRect(x: 0, y: 0, width: W, height: H))
        }
        var tracker: Tracker?
        if let p = point {
            let parts = p.split(separator: ",").compactMap { Double($0.trimmingCharacters(in: .whitespaces)) }
            guard parts.count == 2 else { throw CLIError("--point wants x,y") }
            tracker = Tracker(point: CGPoint(x: parts[0], y: parts[1]), width: W, height: H, ci: ci)
        }

        var progress = Progress(total: reader.info.frameCount)
        var frameIdx = 0
        while let (pb, pts) = reader.next() {
            let frame = CIImage(cvPixelBuffer: pb)

            // 1. mask for this frame (full-res, 0/1 grayscale CIImage)
            var m: CIImage
            if let s = staticMask { m = s }
            else { m = try tracker!.mask(for: pb, frameIndex: frameIdx) ?? CIImage(color: .black).cropped(to: frame.extent) }
            if dilate > 0 {
                m = m.applyingFilter("CIMorphologyMaximum", parameters: [kCIInputRadiusKey: dilate]).cropped(to: frame.extent)
            }

            let outPB = try outPool.make()
            if debugMask {
                let red = CIImage(color: CIColor(red: 1, green: 0, blue: 0, alpha: 0.6)).cropped(to: frame.extent)
                let over = red.applyingFilter("CIBlendWithMask", parameters: [
                    kCIInputBackgroundImageKey: frame, kCIInputMaskImageKey: m])
                ci.render(over, to: outPB)
                try await writer.append(outPB, at: pts)
                frameIdx += 1; progress.tick(); continue
            }

            // 2. bbox of the mask → square crop window
            guard let bbox = nonzeroBounds(of: m, ci: ci), !bbox.isEmpty else {
                try await writer.append(pb, at: pts)   // nothing to remove this frame
                frameIdx += 1; progress.tick(); continue
            }
            let side = max(bbox.width, bbox.height) * (1 + 2 * margin)
            var win = CGRect(x: bbox.midX - side / 2, y: bbox.midY - side / 2, width: side, height: side)
            win = clampSquare(win, to: frame.extent)
            guard win.width >= 8 else {                 // mask too small to crop around
                try await writer.append(pb, at: pts)
                frameIdx += 1; progress.tick(); continue
            }

            // 3. resample crop + mask to the model tile, run LaMa
            let s = CGFloat(tile) / win.width
            let xform = CGAffineTransform(translationX: -win.minX, y: -win.minY).scaledBy(x: s, y: s)
            let cropImg = frame.transformed(by: xform).cropped(to: CGRect(x: 0, y: 0, width: tile, height: tile))
            let cropMask = m.transformed(by: xform).cropped(to: CGRect(x: 0, y: 0, width: tile, height: tile))

            let tIn = try tilePool.make(), tMask = try maskTilePool.make()
            ci.render(cropImg, to: tIn)
            ci.render(cropMask, to: tMask, bounds: cropMask.extent, colorSpace: nil)

            let feats = try MLDictionaryFeatureProvider(dictionary: [
                "image": MLFeatureValue(pixelBuffer: tIn),
                "mask": MLFeatureValue(pixelBuffer: tMask),
            ])
            let pred = try await lama.prediction(from: feats)
            guard let outTile = pred.featureValue(for: "output")?.imageBufferValue
            else { throw CLIError("model returned no 'output' image") }

            // 4. paste back: scale tile up to the window, blend using feathered mask
            let inpainted = CIImage(cvPixelBuffer: outTile)
                .transformed(by: xform.inverted())
            var blendMask = m.cropped(to: win)
            if feather > 0 {
                blendMask = blendMask.clampedToExtent()
                    .applyingFilter("CIGaussianBlur", parameters: [kCIInputRadiusKey: Double(feather) / 2])
                    .cropped(to: win)
            }
            let composite = inpainted.applyingFilter("CIBlendWithMask", parameters: [
                kCIInputBackgroundImageKey: frame, kCIInputMaskImageKey: blendMask,
            ]).cropped(to: frame.extent)
            ci.render(composite, to: outPB)
            try await writer.append(outPB, at: pts)
            frameIdx += 1; progress.tick()
        }
        try await writer.finish()
        progress.done()
    }

    private func clampSquare(_ r: CGRect, to bounds: CGRect) -> CGRect {
        var side = min(r.width, bounds.width, bounds.height)
        side = CGFloat(Int(side / 8) * 8)
        var x = r.midX - side / 2, y = r.midY - side / 2
        x = max(bounds.minX, min(x, bounds.maxX - side))
        y = max(bounds.minY, min(y, bounds.maxY - side))
        return CGRect(x: x.rounded(), y: y.rounded(), width: side, height: side)
    }
}

/// Bounding box of nonzero pixels; cheap approach via CIAreaMinMax on a downscaled mask
/// would lose precision, so we read a 1/4-res copy and scan it.
func nonzeroBounds(of m: CIImage, ci: CIContext) -> CGRect? {
    let scale: CGFloat = 0.25
    let small = m.transformed(by: CGAffineTransform(scaleX: scale, y: scale))
    let w = Int(small.extent.width), h = Int(small.extent.height)
    guard w > 0, h > 0 else { return nil }
    var buf = [UInt8](repeating: 0, count: w * h)
    ci.render(small, toBitmap: &buf, rowBytes: w, bounds: small.extent, format: .R8, colorSpace: nil)
    var minX = w, minY = h, maxX = -1, maxY = -1
    for y in 0..<h { for x in 0..<w where buf[y * w + x] > 64 {
        minX = min(minX, x); maxX = max(maxX, x); minY = min(minY, y); maxY = max(maxY, y)
    } }
    guard maxX >= 0 else { return nil }
    // CI bitmaps come out top-down; CI coordinates are bottom-up.
    let rect = CGRect(x: minX, y: h - 1 - maxY, width: maxX - minX + 1, height: maxY - minY + 1)
    return rect.applying(CGAffineTransform(scaleX: 1 / scale, y: 1 / scale))
}


/// Frame-0 segmentation at a click point, then track the box and re-segment inside it.
final class Tracker {
    private let point: CGPoint
    private let width: Int, height: Int
    private let ci: CIContext
    private let seq = VNSequenceRequestHandler()
    private var observation: VNDetectedObjectObservation?

    init(point: CGPoint, width: Int, height: Int, ci: CIContext) {
        self.point = point; self.width = width; self.height = height; self.ci = ci
    }

    func mask(for pb: CVPixelBuffer, frameIndex: Int) throws -> CIImage? {
        // Vision normalized coords: origin bottom-left. Our --point is top-left pixel coords.
        var probe = CGPoint(x: point.x / CGFloat(width), y: 1 - point.y / CGFloat(height))

        if frameIndex > 0, let obs = observation {
            let track = VNTrackObjectRequest(detectedObjectObservation: obs)
            track.trackingLevel = .accurate
            try seq.perform([track], on: pb)
            guard let updated = track.results?.first as? VNDetectedObjectObservation, updated.confidence > 0.2
            else { return nil }
            observation = updated
            probe = CGPoint(x: updated.boundingBox.midX, y: updated.boundingBox.midY)
        }

        let req = VNGenerateForegroundInstanceMaskRequest()
        try VNImageRequestHandler(cvPixelBuffer: pb).perform([req])
        guard let res = req.results?.first else { return nil }

        // Which instance is under the probe point?
        let idx = instanceIndex(in: res.instanceMask, at: probe)
        guard idx > 0 else { return nil }
        let maskPB = try res.generateScaledMaskForImage(forInstances: IndexSet(integer: idx),
                                                from: VNImageRequestHandler(cvPixelBuffer: pb))
        let ciMask = CIImage(cvPixelBuffer: maskPB)

        if frameIndex == 0 {
            // Seed the tracker with the segmented instance's bbox.
            let scaleUp = CGAffineTransform(scaleX: CGFloat(width) / ciMask.extent.width,
                                            y: CGFloat(height) / ciMask.extent.height)
            let full = ciMask.transformed(by: scaleUp)
            guard let bb = nonzeroBounds(of: full, ci: ci) else { return nil }
            observation = VNDetectedObjectObservation(boundingBox: CGRect(
                x: bb.minX / CGFloat(width), y: bb.minY / CGFloat(height),
                width: bb.width / CGFloat(width), height: bb.height / CGFloat(height)))
        }
        return ciMask.transformed(by: CGAffineTransform(
            scaleX: CGFloat(width) / ciMask.extent.width, y: CGFloat(height) / ciMask.extent.height))
    }

    private func instanceIndex(in maskPB: CVPixelBuffer, at p: CGPoint) -> Int {
        CVPixelBufferLockBaseAddress(maskPB, .readOnly); defer { CVPixelBufferUnlockBaseAddress(maskPB, .readOnly) }
        let w = CVPixelBufferGetWidth(maskPB), h = CVPixelBufferGetHeight(maskPB)
        let x = min(max(Int(p.x * CGFloat(w)), 0), w - 1)
        let y = min(max(Int((1 - p.y) * CGFloat(h)), 0), h - 1)   // buffer rows are top-down
        let base = CVPixelBufferGetBaseAddress(maskPB)!.assumingMemoryBound(to: UInt8.self)
        let row = CVPixelBufferGetBytesPerRow(maskPB)
        return Int(base[y * row + x])
    }
}

extension OutputCodec: ExpressibleByArgument {}
