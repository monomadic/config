import AVFoundation
import CoreVideo
import Foundation

// Minimal AVAssetReader / AVAssetWriter wrapper. Shared verbatim between
// avinterp, avupscale and avremove so each project stays self-contained.

struct VideoInfo {
    let width: Int
    let height: Int
    let fps: Double
    let transform: CGAffineTransform
    let frameCount: Int?
    /// Source color tags as AVVideoColorPropertiesKey wants them; nil when untagged.
    let colorProperties: [String: Any]?
}

final class VideoReader {
    let info: VideoInfo
    private let reader: AVAssetReader
    private let output: AVAssetReaderTrackOutput

    init(url: URL, pixelFormat: OSType) async throws {
        let asset = AVURLAsset(url: url)
        guard let track = try await asset.loadTracks(withMediaType: .video).first else {
            throw CLIError("no video track in \(url.path)")
        }
        let (size, fps, transform, duration, formats) = try await track.load(
            .naturalSize, .nominalFrameRate, .preferredTransform, .timeRange, .formatDescriptions)
        let frames = fps > 0 ? Int((duration.duration.seconds * Double(fps)).rounded()) : nil
        info = VideoInfo(width: Int(size.width), height: Int(size.height),
                         fps: Double(fps), transform: transform, frameCount: frames,
                         colorProperties: formats.first.flatMap(colorProperties(of:)))

        reader = try AVAssetReader(asset: asset)
        output = AVAssetReaderTrackOutput(track: track, outputSettings: [
            kCVPixelBufferPixelFormatTypeKey as String: pixelFormat,
            kCVPixelBufferIOSurfacePropertiesKey as String: [:] as [String: Any],
        ])
        output.alwaysCopiesSampleData = false
        reader.add(output)
        guard reader.startReading() else {
            throw CLIError("reader failed: \(reader.error?.localizedDescription ?? "?")")
        }
    }

    /// Stop decoding; for readers opened only to probe the file.
    func cancel() { reader.cancelReading() }

    /// Returns nil at end of stream.
    func next() -> (CVPixelBuffer, CMTime)? {
        guard let sb = output.copyNextSampleBuffer(),
              let pb = CMSampleBufferGetImageBuffer(sb) else { return nil }
        return (pb, CMSampleBufferGetPresentationTimeStamp(sb))
    }
}

enum OutputCodec: String, CaseIterable {
    case hevc, h264, prores422, prores4444

    var avCodec: AVVideoCodecType {
        switch self {
        case .hevc: return .hevc
        case .h264: return .h264
        case .prores422: return .proRes422
        case .prores4444: return .proRes4444
        }
    }
}

/// The CoreMedia extension strings and the AVVideo* constants share values
/// ("ITU_R_709_2" etc.), so the tags pass straight through.
private func colorProperties(of fd: CMFormatDescription) -> [String: Any]? {
    let ext = CMFormatDescriptionGetExtensions(fd) as? [String: Any] ?? [:]
    guard let p = ext[kCMFormatDescriptionExtension_ColorPrimaries as String] as? String,
          let t = ext[kCMFormatDescriptionExtension_TransferFunction as String] as? String,
          let m = ext[kCMFormatDescriptionExtension_YCbCrMatrix as String] as? String
    else { return nil }
    return [AVVideoColorPrimariesKey: p, AVVideoTransferFunctionKey: t, AVVideoYCbCrMatrixKey: m]
}

final class VideoWriter {
    private let writer: AVAssetWriter
    private let input: AVAssetWriterInput
    private let adaptor: AVAssetWriterInputPixelBufferAdaptor

    init(url: URL, width: Int, height: Int, fps: Double, transform: CGAffineTransform,
         codec: OutputCodec, pixelFormat: OSType, bitrateMbps: Double? = nil,
         colorProperties: [String: Any]? = nil) throws {
        try? FileManager.default.removeItem(at: url)
        writer = try AVAssetWriter(outputURL: url, fileType: url.pathExtension.lowercased() == "mov" ? .mov : .mp4)

        var settings: [String: Any] = [
            AVVideoCodecKey: codec.avCodec,
            AVVideoWidthKey: width,
            AVVideoHeightKey: height,
        ]
        if let c = colorProperties { settings[AVVideoColorPropertiesKey] = c }
        if let mbps = bitrateMbps, codec == .hevc || codec == .h264 {
            settings[AVVideoCompressionPropertiesKey] = [
                AVVideoAverageBitRateKey: Int(mbps * 1_000_000),
                AVVideoExpectedSourceFrameRateKey: Int(fps.rounded()),
            ]
        }
        input = AVAssetWriterInput(mediaType: .video, outputSettings: settings)
        input.expectsMediaDataInRealTime = false
        input.transform = transform
        adaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input, sourcePixelBufferAttributes: [
            kCVPixelBufferPixelFormatTypeKey as String: pixelFormat,
            kCVPixelBufferWidthKey as String: width,
            kCVPixelBufferHeightKey as String: height,
        ])
        writer.add(input)
        guard writer.startWriting() else {
            throw CLIError("writer failed: \(writer.error?.localizedDescription ?? "?")")
        }
        writer.startSession(atSourceTime: .zero)
    }

    func append(_ pb: CVPixelBuffer, at time: CMTime) async throws {
        while !input.isReadyForMoreMediaData {
            try await Task.sleep(nanoseconds: 1_000_000)
        }
        guard adaptor.append(pb, withPresentationTime: time) else {
            throw CLIError("append failed at \(time.seconds)s: \(writer.error?.localizedDescription ?? "?")")
        }
    }

    func finish() async throws {
        input.markAsFinished()
        await writer.finishWriting()
        if let e = writer.error { throw CLIError("finish failed: \(e.localizedDescription)") }
    }
}

/// Pixel buffer pool built from a VideoToolbox configuration's attribute dictionary.
final class BufferPool {
    private var pool: CVPixelBufferPool?

    init(attributes: [String: Any], width: Int, height: Int, pixelFormat: OSType) throws {
        var attrs = attributes
        attrs[kCVPixelBufferWidthKey as String] = width
        attrs[kCVPixelBufferHeightKey as String] = height
        attrs[kCVPixelBufferPixelFormatTypeKey as String] = pixelFormat
        attrs[kCVPixelBufferIOSurfacePropertiesKey as String] = [:] as [String: Any]
        let status = CVPixelBufferPoolCreate(nil, nil, attrs as CFDictionary, &pool)
        guard status == kCVReturnSuccess, pool != nil else { throw CLIError("pool create failed: \(status)") }
    }

    func make() throws -> CVPixelBuffer {
        var pb: CVPixelBuffer?
        let status = CVPixelBufferPoolCreatePixelBuffer(nil, pool!, &pb)
        guard status == kCVReturnSuccess, let out = pb else { throw CLIError("buffer alloc failed: \(status)") }
        return out
    }
}

struct CLIError: Error, CustomStringConvertible {
    let description: String
    init(_ s: String) { description = s }
}

struct Progress {
    let total: Int?
    var count = 0
    let start = Date()

    mutating func tick(_ n: Int = 1) {
        count += n
        guard count % 24 == 0 else { return }
        let elapsed = Date().timeIntervalSince(start)
        let rate = Double(count) / max(elapsed, 0.001)
        if let t = total, t > 0 {
            let eta = Double(t - count) / max(rate, 0.001)
            FileHandle.standardError.write("\r\(count)/\(t)  \(String(format: "%.1f", rate)) fps  eta \(Int(eta))s   ".data(using: .utf8)!)
        } else {
            FileHandle.standardError.write("\r\(count)  \(String(format: "%.1f", rate)) fps   ".data(using: .utf8)!)
        }
    }

    func done() {
        let elapsed = Date().timeIntervalSince(start)
        FileHandle.standardError.write("\r\(count) frames in \(String(format: "%.1f", elapsed))s (\(String(format: "%.1f", Double(count) / max(elapsed, 0.001))) fps)\n".data(using: .utf8)!)
    }
}
