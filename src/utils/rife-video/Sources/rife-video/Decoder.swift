import AVFoundation
import CoreVideo
import VideoToolbox
import Foundation

enum DecoderSetup: Error { case remuxHEVC }

struct VideoInfo {
    let width: Int
    let height: Int
    let fps: Double
    let end: CMTime
    let transform: CGAffineTransform
    let color: [String: Any]
    let codec: String
}
struct DecodedFrame {
    let buffer: CVPixelBuffer
    let time: CMTime
    let duration: CMTime
}

/// Compressed AVAssetReader samples -> explicitly hardware-required VT session.
/// Sample PTS is retained. Reorder using the next packet's DTS as a watermark;
/// packets lacking DTS use a bounded 16-frame reorder window (H.264/HEVC DPB).
final class HardwareDecoder {
    let info: VideoInfo
    private let reader: AVAssetReader
    private let output: AVAssetReaderTrackOutput
    private var session: VTDecompressionSession?
    private var sessionFormat: CMFormatDescription?
    private var packet: CMSampleBuffer?
    private var pending: [DecodedFrame] = []
    private var lastPTS: CMTime?
    private var ended = false
    private var callbackError: OSStatus = noErr
    private(set) var hardwareVerified = false
    private(set) var decodeSeconds = 0.0

    init(url: URL) async throws {
        let asset = AVURLAsset(url: url)
        guard let track = try await asset.loadTracks(withMediaType: .video).first else { throw Failure("no video track") }
        let (formats, fps, range, transform) = try await track.load(.formatDescriptions, .nominalFrameRate, .timeRange, .preferredTransform)
        guard let format = formats.first else { throw Failure("no video format description") }
        let codec = CMFormatDescriptionGetMediaSubType(format)
        if codec == 0x68657631 { throw DecoderSetup.remuxHEVC } // hev1; AVFoundation needs hvc1
        guard codec == kCMVideoCodecType_H264 || codec == kCMVideoCodecType_HEVC else {
            throw Failure("input must be H.264 or HEVC")
        }
        let dimensions = CMVideoFormatDescriptionGetDimensions(format)
        let ext = CMFormatDescriptionGetExtensions(format) as? [String: Any] ?? [:]
        let transfer = ext[kCMFormatDescriptionExtension_TransferFunction as String] as? String ?? AVVideoTransferFunction_ITU_R_709_2
        guard transfer != kCVImageBufferTransferFunction_SMPTE_ST_2084_PQ as String,
              transfer != kCVImageBufferTransferFunction_ITU_R_2100_HLG as String else {
            throw Failure("HDR (PQ/HLG) is unsupported: RIFE's current buffer path is 8-bit SDR; tone-map explicitly first")
        }
        if let aspect = ext[kCMFormatDescriptionExtension_PixelAspectRatio as String] as? [String: Any],
           let h = aspect[kCMFormatDescriptionKey_PixelAspectRatioHorizontalSpacing as String] as? NSNumber,
           let v = aspect[kCMFormatDescriptionKey_PixelAspectRatioVerticalSpacing as String] as? NSNumber,
           h != v { throw Failure("non-square pixels are unsupported; normalize the source pixel aspect ratio first") }
        let color: [String: Any] = [
            AVVideoColorPrimariesKey: ext[kCMFormatDescriptionExtension_ColorPrimaries as String] ?? AVVideoColorPrimaries_ITU_R_709_2,
            AVVideoTransferFunctionKey: transfer,
            AVVideoYCbCrMatrixKey: ext[kCMFormatDescriptionExtension_YCbCrMatrix as String] ?? AVVideoYCbCrMatrix_ITU_R_709_2,
        ]
        // Keep the format contract narrow: no unannounced gamut conversion or wrong output tags.
        guard color[AVVideoColorPrimariesKey] as? String == AVVideoColorPrimaries_ITU_R_709_2,
              color[AVVideoYCbCrMatrixKey] as? String == AVVideoYCbCrMatrix_ITU_R_709_2,
              transfer == AVVideoTransferFunction_ITU_R_709_2 else {
            throw Failure("only BT.709 SDR (or untagged) is supported, not HDR/other color spaces; convert explicitly first")
        }
        guard dimensions.width > 0, dimensions.height > 0,
              dimensions.width % 2 == 0, dimensions.height % 2 == 0 else { throw Failure("input dimensions must be positive and even") }
        info = VideoInfo(width: Int(dimensions.width), height: Int(dimensions.height), fps: Double(fps),
                         end: CMTimeRangeGetEnd(range), transform: transform, color: color,
                         codec: codec == kCMVideoCodecType_H264 ? "h264" : "hevc")
        reader = try AVAssetReader(asset: asset)
        output = AVAssetReaderTrackOutput(track: track, outputSettings: nil)
        output.alwaysCopiesSampleData = false
        guard reader.canAdd(output) else { throw Failure("cannot read compressed video samples") }
        reader.add(output)
        guard reader.startReading() else { throw Failure("reader: \(reader.error?.localizedDescription ?? "failed")") }
        packet = nextPacket()
        let decodeFormat = packet.flatMap(CMSampleBufferGetFormatDescription) ?? format
        sessionFormat = decodeFormat
        let status = VTDecompressionSessionCreate(allocator: nil, formatDescription: decodeFormat,
            decoderSpecification: [kVTVideoDecoderSpecification_RequireHardwareAcceleratedVideoDecoder: true] as CFDictionary,
            imageBufferAttributes: [kCVPixelBufferPixelFormatTypeKey: kCVPixelFormatType_32BGRA,
                                    kCVPixelBufferIOSurfacePropertiesKey: [:]] as CFDictionary,
            outputCallback: nil, decompressionSessionOut: &session)
        guard status == noErr else { throw Failure("hardware decoder unavailable (VideoToolbox \(status)); no software fallback") }
    }

    deinit {
        reader.cancelReading()
        if let session { VTDecompressionSessionInvalidate(session) }
    }

    private func nextPacket() -> CMSampleBuffer? {
        // AVAssetReader emits zero-sample marker buffers for edit boundaries/EOF.
        // They carry no compressed image or format and must not reach VideoToolbox.
        while let sb = output.copyNextSampleBuffer() {
            if CMSampleBufferGetNumSamples(sb) > 0 { return sb }
        }
        return nil
    }

    func next() throws -> DecodedFrame? {
        while true {
            pending.sort { CMTimeCompare($0.time, $1.time) < 0 }
            if let first = pending.first {
                // Use the edit-adjusted timestamp domain, matching VT callback PTS.
                let nextDTS = packet.map(CMSampleBufferGetOutputDecodeTimeStamp)
                let safe = ended || (nextDTS?.isNumeric == true && CMTimeCompare(first.time, nextDTS!) < 0)
                    || (nextDTS?.isNumeric != true && pending.count > 16)
                if safe {
                    pending.removeFirst()
                    if let lastPTS, CMTimeCompare(first.time, lastPTS) <= 0 {
                        throw Failure("non-increasing video PTS: \(first.time.seconds) after \(lastPTS.seconds); next DTS \(nextDTS?.seconds ?? -999)")
                    }
                    lastPTS = first.time
                    return first
                }
            }
            if ended { return nil }
            guard let sb = packet else {
                if reader.status == .failed { throw Failure("decode read: \(reader.error?.localizedDescription ?? "failed")") }
                guard reader.status == .completed else { throw Failure("unexpected end of compressed video") }
                ended = true
                continue
            }
            guard let session, let format = CMSampleBufferGetFormatDescription(sb),
                  CMFormatDescriptionEqual(format, otherFormatDescription: sessionFormat!) else {
                throw Failure("midstream video format change is unsupported")
            }
            let start = uptime()
            // No async/temporal flags: callbacks finish before this call returns.
            let status = VTDecompressionSessionDecodeFrame(session, sampleBuffer: sb, flags: [], infoFlagsOut: nil) { [self] status, _, image, pts, duration in
                guard status == noErr, let image, pts.isNumeric else { callbackError = status == noErr ? -1 : status; return }
                pending.append(DecodedFrame(buffer: image, time: pts,
                                            duration: duration.isNumeric ? duration : CMSampleBufferGetOutputDuration(sb)))
            }
            decodeSeconds += uptime() - start
            guard status == noErr, callbackError == noErr else { throw Failure("hardware decode failed (\(status), callback \(callbackError))") }
            if !hardwareVerified {
                var value: Unmanaged<CFTypeRef>?
                let s = VTSessionCopyProperty(session, key: kVTDecompressionPropertyKey_UsingHardwareAcceleratedVideoDecoder,
                                              allocator: nil, valueOut: &value)
                guard s == noErr, (value?.takeRetainedValue() as? NSNumber)?.boolValue == true else { throw Failure("decoder did not confirm hardware acceleration") }
                hardwareVerified = true
            }
            guard pending.count <= 32 else { throw Failure("excessive frame reordering; refusing unbounded buffering") }
            packet = nextPacket()
        }
    }
}

/// Bounded decode-ahead queue; overlaps hardware decode with Metal/encoding.
/// Errors and EOF are delivered after already queued frames. Cancel wakes both sides.
final class DecodeQueue {
    private let condition = NSCondition()
    private var frames: [DecodedFrame] = []
    private var done = false
    private var cancelled = false
    private var failure: Error?
    let decoder: HardwareDecoder
    init(_ decoder: HardwareDecoder) {
        self.decoder = decoder
        DispatchQueue(label: "rife-video.decode", qos: .userInitiated).async { [self] in
            do {
                while true {
                    condition.lock()
                    while frames.count >= 3 && !cancelled { condition.wait() }
                    let stop = cancelled; condition.unlock()
                    if stop { break }
                    let frame = try autoreleasepool { try decoder.next() }
                    condition.lock()
                    if let frame { frames.append(frame) } else { done = true }
                    condition.broadcast(); condition.unlock()
                    if frame == nil { return }
                }
            } catch {
                condition.lock(); failure = error; done = true; condition.broadcast(); condition.unlock()
            }
        }
    }
    func next() throws -> DecodedFrame? {
        condition.lock(); defer { condition.unlock() }
        while frames.isEmpty && !done && !cancelled { condition.wait() }
        if !frames.isEmpty { let f = frames.removeFirst(); condition.broadcast(); return f }
        if let failure { throw failure }
        return nil
    }
    func cancel() { condition.lock(); cancelled = true; condition.broadcast(); condition.unlock() }
}
