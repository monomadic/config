import AVFoundation
import VideoToolbox
import Foundation
import RifeVideoCore

protocol FrameWriter: AnyObject {
    func append(_ buffer: CVPixelBuffer, at time: CMTime) async throws
    func finish(at end: CMTime) async throws
    func cancel()
}

final class HardwareWriter: FrameWriter {
    private let writer: AVAssetWriter
    private let input: AVAssetWriterInput
    private let adaptor: AVAssetWriterInputPixelBufferAdaptor
    init(url: URL, info: VideoInfo, rate: FrameRate, codec: Codec, profile: ProResProfile, mbps: Double?) throws {
        writer = try AVAssetWriter(outputURL: url, fileType: .mov)
        var settings: [String: Any] = [
            AVVideoCodecKey: codec == .hevc ? AVVideoCodecType.hevc : profile.avCodec,
            AVVideoWidthKey: info.width, AVVideoHeightKey: info.height,
            AVVideoColorPropertiesKey: info.color,
            AVVideoEncoderSpecificationKey: [kVTVideoEncoderSpecification_RequireHardwareAcceleratedVideoEncoder: true],
        ]
        if codec == .hevc {
            let bits = (mbps ?? max(2, 20 * Double(info.width * info.height) / (1920 * 1080))) * 1_000_000
            settings[AVVideoCompressionPropertiesKey] = [
                AVVideoAverageBitRateKey: Int(bits),
                AVVideoExpectedSourceFrameRateKey: rate.value,
                AVVideoMaxKeyFrameIntervalDurationKey: 2,
            ]
        }
        guard writer.canApply(outputSettings: settings, forMediaType: .video) else { throw Failure("hardware encoder rejects output settings") }
        input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
        input.expectsMediaDataInRealTime = false
        input.mediaTimeScale = rate.numerator
        input.transform = info.transform
        adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: [
            kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32BGRA,
            kCVPixelBufferWidthKey as String: info.width, kCVPixelBufferHeightKey as String: info.height,
            kCVPixelBufferIOSurfacePropertiesKey as String: [:] as [String: Any],
        ])
        guard writer.canAdd(input) else { throw Failure("cannot add video writer input") }
        writer.add(input)
        guard writer.startWriting() else { throw Failure("hardware encoder unavailable: \(writer.error?.localizedDescription ?? "unknown error"); no software fallback") }
        writer.startSession(atSourceTime: .zero)
    }
    func append(_ buffer: CVPixelBuffer, at time: CMTime) async throws {
        while !input.isReadyForMoreMediaData {
            guard writer.status == .writing else { throw Failure("encoder: \(writer.error?.localizedDescription ?? "stopped")") }
            try Task.checkCancellation()
            try await Task.sleep(nanoseconds: 500_000)
        }
        guard adaptor.append(buffer, withPresentationTime: time) else { throw Failure("encode at \(time.seconds): \(writer.error?.localizedDescription ?? "failed")") }
    }
    func finish(at end: CMTime) async throws {
        writer.endSession(atSourceTime: end)
        input.markAsFinished()
        await writer.finishWriting()
        guard writer.status == .completed else { throw Failure("encoder finish: \(writer.error?.localizedDescription ?? "failed")") }
    }
    func cancel() { writer.cancelWriting() }
}

final class AV1Writer: FrameWriter {
    private let process: Process
    private let pipe: Pipe
    private let log: URL
    private let width: Int
    private let height: Int
    private var closed = false
    private var rowData: Data
    init(url: URL, directory: URL, info: VideoInfo, rate: FrameRate, crf: Int, preset: Int) throws {
        width = info.width; height = info.height
        rowData = Data(count: width * height * 4)
        log = directory.appendingPathComponent("av1.log")
        FileManager.default.createFile(atPath: log.path, contents: nil)
        process = Process(); pipe = Pipe()
        process.executableURL = try executable("ffmpeg")
        process.arguments = ["-hide_banner", "-loglevel", "warning", "-nostdin", "-y", "-f", "rawvideo",
            "-pixel_format", "bgra", "-video_size", "\(width)x\(height)",
            "-framerate", "\(rate.numerator)/\(rate.denominator)", "-i", "pipe:0", "-an",
            "-vf", "scale=in_range=full:out_range=tv:out_color_matrix=bt709,format=yuv420p",
            "-c:v", "libsvtav1", "-crf", String(crf), "-preset", String(preset),
            "-color_primaries", "bt709", "-color_trc", "bt709", "-colorspace", "bt709", "-color_range", "tv", url.path]
        process.standardInput = pipe
        process.standardOutput = FileHandle.nullDevice
        process.standardError = try FileHandle(forWritingTo: log)
        try process.run()
        // The child owns this read endpoint now; don't keep it alive on encoder failure.
        try pipe.fileHandleForReading.close()
    }
    func append(_ buffer: CVPixelBuffer, at time: CMTime) async throws {
        guard process.isRunning else { throw Failure("AV1 encoder stopped: \(logTail(log))") }
        CVPixelBufferLockBaseAddress(buffer, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(buffer, .readOnly) }
        guard let ptr = CVPixelBufferGetBaseAddress(buffer) else { throw Failure("no pixel buffer address") }
        let pitch = CVPixelBufferGetBytesPerRow(buffer)
        rowData.withUnsafeMutableBytes { target in
            for y in 0..<height { memcpy(target.baseAddress!.advanced(by: y * width * 4), ptr.advanced(by: y * pitch), width * 4) }
        }
        do { try pipe.fileHandleForWriting.write(contentsOf: rowData) }
        catch { throw Failure("AV1 pipe: \(error); \(logTail(log))") }
    }
    func finish(at end: CMTime) async throws {
        try pipe.fileHandleForWriting.close(); closed = true
        process.waitUntilExit()
        guard process.terminationStatus == 0 else { throw Failure("AV1 encoder: \(logTail(log))") }
    }
    func cancel() {
        if !closed { try? pipe.fileHandleForWriting.close(); closed = true }
        if process.isRunning { process.terminate(); process.waitUntilExit() }
    }
}
