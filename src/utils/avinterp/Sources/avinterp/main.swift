import ArgumentParser
import AVFoundation
import VideoToolbox

// avinterp — ML frame interpolation via VideoToolbox VTFrameProcessor
// (VTFrameRateConversionConfiguration). Apple silicon, macOS 15.4+.
//
// Docs: https://developer.apple.com/documentation/videotoolbox/vtframerateconversionconfiguration
// WWDC25 session 300 "Enhance your app with machine-learning-based video effects".

@main
struct AVInterp: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        abstract: "Interpolate video frames with Apple's on-device optical-flow model.")

    @Argument(help: "Input video.") var input: String
    @Argument(help: "Output video (.mp4 or .mov).") var output: String

    @Option(name: .shortAndLong, help: "Multiply frame rate by this integer (2 = 30→60).")
    var factor: Int = 2

    @Option(name: .long, help: "Override output fps (writes timestamps at this rate; frame count unchanged). Useful for slow-mo: --factor 4 --fps 30 on 30fps input = 4x slowmo.")
    var fps: Double?

    @Option(name: .long, help: "quality | normal")
    var quality: String = "quality"

    @Option(name: .shortAndLong, help: "hevc | h264 | prores422 | prores4444")
    var codec: OutputCodec = .hevc

    @Option(name: .long, help: "Bitrate in Mbps for hevc/h264.")
    var mbps: Double?

    @Flag(name: .long, help: "Precompute optical flow with VTOpticalFlowConfiguration instead of on-the-fly (slower, sometimes better on fast motion).")
    var precomputeFlow = false

    func run() async throws {
        guard factor >= 2 else { throw CLIError("--factor must be >= 2") }
        let inURL = URL(fileURLWithPath: input)
        let outURL = URL(fileURLWithPath: output)

        // Probe once to get dimensions, then build the VT configuration.
        let probe = try await VideoReader(url: inURL, pixelFormat: kCVPixelFormatType_32BGRA)
        let w = probe.info.width, h = probe.info.height
        probe.cancel()
        let srcFps = probe.info.fps
        let outFps = fps ?? (srcFps * Double(factor))

        let normal = quality == "normal"
        let prio: VTFrameRateConversionConfiguration.QualityPrioritization = normal ? .normal : .quality

        guard let config = VTFrameRateConversionConfiguration(
            frameWidth: w, frameHeight: h,
            usePrecomputedFlow: precomputeFlow,
            qualityPrioritization: prio,
            revision: .revision1)
        else { throw CLIError("VTFrameRateConversionConfiguration rejected \(w)x\(h) (max 8192x4320 on macOS)") }
        guard VTFrameRateConversionConfiguration.isSupported else { throw CLIError("frame rate conversion not supported on this machine") }

        // Let the processor tell us what pixel format it wants.
        let pixelFormat = config.supportedPixelFormats.first ?? kCVPixelFormatType_420YpCbCr8BiPlanarFullRange
        let reader = try await VideoReader(url: inURL, pixelFormat: pixelFormat)
        let writer = try VideoWriter(url: outURL, width: w, height: h, fps: outFps,
                                     transform: reader.info.transform, codec: codec,
                                     pixelFormat: pixelFormat, bitrateMbps: mbps,
                                     colorProperties: reader.info.colorProperties)
        let pool = try BufferPool(attributes: config.destinationPixelBufferAttributes,
                                  width: w, height: h, pixelFormat: pixelFormat)

        let processor = VTFrameProcessor()
        try processor.startSession(configuration: config)
        defer { processor.endSession() }

        var flowProcessor: VTFrameProcessor?
        var flowPool: BufferPool?
        if precomputeFlow {
            guard let flowCfg = VTOpticalFlowConfiguration(frameWidth: w, frameHeight: h,
                                                           qualityPrioritization: normal ? .normal : .quality,
                                                           revision: .revision1),
                  VTOpticalFlowConfiguration.isSupported
            else { throw CLIError("optical flow processor unavailable") }
            let fp = VTFrameProcessor()
            try fp.startSession(configuration: flowCfg)
            flowProcessor = fp
            flowPool = try BufferPool(attributes: flowCfg.destinationPixelBufferAttributes,
                                      width: w, height: h,
                                      pixelFormat: kCVPixelFormatType_TwoComponent16Half)
        }
        defer { flowProcessor?.endSession() }

        let phases: [Float] = (1..<factor).map { Float($0) / Float(factor) }
        var outIndex = 0
        func ts(_ i: Int) -> CMTime { CMTime(seconds: Double(i) / outFps, preferredTimescale: 90000) }

        var progress = Progress(total: reader.info.frameCount)
        guard var (prev, _) = reader.next() else { throw CLIError("empty video") }

        while let (cur, _) = reader.next() {
            try await writer.append(prev, at: ts(outIndex)); outIndex += 1

            let src = VTFrameProcessorFrame(buffer: prev, presentationTimeStamp: ts(outIndex - 1))!
            let nxt = VTFrameProcessorFrame(buffer: cur, presentationTimeStamp: ts(outIndex + phases.count))!

            var flow: VTFrameProcessorOpticalFlow?
            if let fp = flowProcessor, let fpool = flowPool {
                let fwd = try fpool.make(), bwd = try fpool.make()
                let params = VTOpticalFlowParameters(
                    sourceFrame: src, nextFrame: nxt, submissionMode: .sequential,
                    destinationOpticalFlow: VTFrameProcessorOpticalFlow(forwardFlow: fwd, backwardFlow: bwd)!)!
                try await fp.process(parameters: params)
                flow = params.destinationOpticalFlow
            }

            let dests = try phases.map { _ in try pool.make() }
            let destFrames = dests.enumerated().map { i, pb in
                VTFrameProcessorFrame(buffer: pb, presentationTimeStamp: ts(outIndex + i))!
            }
            let params = VTFrameRateConversionParameters(
                sourceFrame: src, nextFrame: nxt, opticalFlow: flow,
                interpolationPhase: phases, submissionMode: .sequential,
                destinationFrames: destFrames)!
            try await processor.process(parameters: params)

            for pb in dests {
                try await writer.append(pb, at: ts(outIndex)); outIndex += 1
            }
            prev = cur
            progress.tick()
        }
        try await writer.append(prev, at: ts(outIndex)); outIndex += 1
        try await writer.finish()
        progress.done()
        FileHandle.standardError.write("wrote \(outIndex) frames @ \(outFps) fps → \(outURL.path)\n".data(using: .utf8)!)
    }
}

extension OutputCodec: ExpressibleByArgument {}
