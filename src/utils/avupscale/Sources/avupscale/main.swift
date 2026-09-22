import ArgumentParser
import AVFoundation
import VideoToolbox

// avupscale — ML super-resolution via VideoToolbox VTFrameProcessor
// (VTSuperResolutionScalerConfiguration). Apple silicon, macOS 15.4+.
// The model is downloaded on first use (Apple's on-demand asset); this CLI
// handles that step. Video mode is temporal: it feeds the previous input
// and previous output frame back into the model to reduce flicker.
//
// Docs: https://developer.apple.com/documentation/videotoolbox/vtsuperresolutionscalerconfiguration

@main
struct AVUpscale: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Upscale video with Apple's on-device super-resolution model.")

    @Argument(help: "Input video.") var input: String
    @Argument(help: "Output video (.mp4 or .mov).") var output: String

    @Option(name: .shortAndLong, help: "Integer scale factor. Use --list to see what this machine supports.")
    var scale: Int = 2

    @Flag(name: .long, help: "Treat each frame independently (no temporal feedback). Use for slideshows / cuts-heavy footage.")
    var noTemporal = false

    @Option(name: .shortAndLong, help: "hevc | h264 | prores422 | prores4444")
    var codec: OutputCodec = .prores422

    @Option(name: .long, help: "Bitrate in Mbps for hevc/h264.")
    var mbps: Double?

    @Flag(name: .long, help: "Print supported scale factors and model status, then exit.")
    var list = false

    func run() async throws {
        let inURL = URL(fileURLWithPath: input)
        let outURL = URL(fileURLWithPath: output)

        let probe = try await VideoReader(url: inURL, pixelFormat: kCVPixelFormatType_32BGRA)
        let w = probe.info.width, h = probe.info.height
        probe.cancel()

        guard VTSuperResolutionScalerConfiguration.isSupported else {
            throw CLIError("super-resolution not supported on this machine")
        }
        let supported = VTSuperResolutionScalerConfiguration.supportedScaleFactors
        if list {
            print("supported scale factors: \(supported)")
        }
        guard supported.contains(scale) else {
            throw CLIError("scale \(scale) not supported; available: \(supported)")
        }

        guard let config = VTSuperResolutionScalerConfiguration(
            frameWidth: w, frameHeight: h,
            scaleFactor: scale,
            inputType: noTemporal ? .image : .video,
            usePrecomputedFlow: false,
            qualityPrioritization: .normal,   // the only case the SDK defines
            revision: .revision1)
        else { throw CLIError("configuration rejected \(w)x\(h) @ \(scale)x") }

        if list {
            print("model status: \(describe(config.configurationModelStatus))")
            return
        }
        try await ensureModel(config)

        let pixelFormat = config.supportedPixelFormats.first ?? kCVPixelFormatType_420YpCbCr8BiPlanarFullRange
        let ow = w * scale, oh = h * scale

        let reader = try await VideoReader(url: inURL, pixelFormat: pixelFormat)
        let writer = try VideoWriter(url: outURL, width: ow, height: oh, fps: reader.info.fps,
                                     transform: reader.info.transform, codec: codec,
                                     pixelFormat: pixelFormat, bitrateMbps: mbps,
                                     colorProperties: reader.info.colorProperties)
        let pool = try BufferPool(attributes: config.destinationPixelBufferAttributes,
                                  width: ow, height: oh, pixelFormat: pixelFormat)

        let processor = VTFrameProcessor()
        try processor.startSession(configuration: config)
        defer { processor.endSession() }

        var progress = Progress(total: reader.info.frameCount)
        var prevIn: VTFrameProcessorFrame?
        var prevOut: VTFrameProcessorFrame?

        while let (pb, pts) = reader.next() {
            let src = VTFrameProcessorFrame(buffer: pb, presentationTimeStamp: pts)!
            let dst = VTFrameProcessorFrame(buffer: try pool.make(), presentationTimeStamp: pts)!
            let params = VTSuperResolutionScalerParameters(
                sourceFrame: src,
                previousFrame: noTemporal ? nil : prevIn,
                previousOutputFrame: noTemporal ? nil : prevOut,
                opticalFlow: nil,
                submissionMode: .sequential,
                destinationFrame: dst)!
            try await processor.process(parameters: params)
            try await writer.append(dst.buffer, at: pts)
            prevIn = src
            prevOut = dst
            progress.tick()
        }
        try await writer.finish()
        progress.done()
        FileHandle.standardError.write("wrote \(ow)x\(oh) → \(outURL.path)\n".data(using: .utf8)!)
    }

    /// Download Apple's SR model asset if it isn't on this machine yet.
    private func ensureModel(_ config: VTSuperResolutionScalerConfiguration) async throws {
        switch config.configurationModelStatus {
        case .ready:
            return
        case .downloading:
            FileHandle.standardError.write("model download already in progress; waiting…\n".data(using: .utf8)!)
        case .downloadRequired:
            FileHandle.standardError.write("downloading super-resolution model (one-time)…\n".data(using: .utf8)!)
        @unknown default:
            break
        }
        // The SDK's download call has no progress callback; poll the percentage instead.
        let ticker = Task {
            while !Task.isCancelled {
                let pct = Int(config.configurationModelPercentageAvailable * 100)
                FileHandle.standardError.write("\r  \(pct)%   ".data(using: .utf8)!)
                try? await Task.sleep(nanoseconds: 500_000_000)
            }
        }
        defer { ticker.cancel(); FileHandle.standardError.write("\n".data(using: .utf8)!) }
        try await config.downloadConfigurationModel()
    }

    private func describe(_ s: VTSuperResolutionScalerConfiguration.ModelStatus) -> String {
        switch s {
        case .ready: return "ready"
        case .downloading: return "downloading"
        case .downloadRequired: return "download required (runs automatically on first upscale)"
        @unknown default: return "unknown (\(s.rawValue))"
        }
    }
}

extension OutputCodec: ExpressibleByArgument {}
