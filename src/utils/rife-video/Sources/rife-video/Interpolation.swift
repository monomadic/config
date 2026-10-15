import Accelerate
import AVFoundation
import ArgumentParser
import CNCNN
import Foundation
import RifeMetal

enum Backend: String, ExpressibleByArgument { case metal, ncnn }

/// One ordered output slot. NCNN completes slots concurrently; encoding consumes in timestamp order.
final class FrameResult {
    private let condition = NSCondition()
    private var result: Result<CVPixelBuffer, Error>?
    init(_ buffer: CVPixelBuffer? = nil) { if let buffer { result = .success(buffer) } }
    func complete(_ value: Result<CVPixelBuffer, Error>) {
        condition.lock(); result = value; condition.broadcast(); condition.unlock()
    }
    func get() throws -> CVPixelBuffer {
        condition.lock(); defer { condition.unlock() }
        while result == nil { condition.wait() }
        return try result!.get()
    }
}

protocol InterpolationStream {
    func push(_ frame: CVPixelBuffer, timesteps: [Float]) throws -> [FrameResult]
}

final class MetalStream: InterpolationStream {
    private let stream: RifeStream
    init(model: URL, tier: Tier, width: Int, height: Int) throws {
        let engine = try RifeInterpolator(configuration: RifeConfiguration(modelURL: model,
            qualityTier: RifeQualityTier(rawValue: tier.rawValue)!))
        stream = try engine.makeStream(width: width, height: height)
    }
    func push(_ frame: CVPixelBuffer, timesteps: [Float]) throws -> [FrameResult] {
        try stream.push(frame, timesteps: timesteps).map { FrameResult($0) }
    }
}

final class NCNNStream: InterpolationStream {
    private let context: UnsafeMutableRawPointer
    private let width: Int, height: Int
    private let queue = DispatchQueue(label: "rife.ncnn", qos: .userInitiated, attributes: .concurrent)
    private let slots: DispatchSemaphore
    private let tasks = DispatchGroup()
    private var previous: Data?
    private let pool: CVPixelBufferPool

    init(model: URL, workers: Int, fp16Math: Bool, width: Int, height: Int) throws {
        self.width = width; self.height = height; slots = DispatchSemaphore(value: workers)
        // The architecture determines padding. Reject unknown directories rather than silently using 32.
        let name = model.lastPathComponent
        let padding: Int32
        if name == "rife-v4.25-lite" { padding = 128 }
        else if ["rife-v4.25", "rife-v4.25-heavy", "rife-v4.26", "rife-v4.26-large"].contains(name) { padding = 64 }
        else if ["rife-v4.6", "rife-v4.15-lite", "rife-v4.18", "rife-v4.22"].contains(name) { padding = 32 }
        else { throw Failure("unsupported NCNN model directory: \(name)") }
        for file in ["flownet.param", "flownet.bin"] {
            guard FileManager.default.isReadableFile(atPath: model.appendingPathComponent(file).path) else {
                throw Failure("missing NCNN model file: \(model.appendingPathComponent(file).path)")
            }
        }
        var createdPool: CVPixelBufferPool?
        let attributes: [CFString: Any] = [kCVPixelBufferWidthKey: width, kCVPixelBufferHeightKey: height,
            kCVPixelBufferPixelFormatTypeKey: kCVPixelFormatType_32BGRA, kCVPixelBufferIOSurfacePropertiesKey: [:]]
        guard CVPixelBufferPoolCreate(nil, nil, attributes as CFDictionary, &createdPool) == kCVReturnSuccess,
              let createdPool else { throw Failure("cannot create NCNN output buffer pool") }
        pool = createdPool
        var error = [CChar](repeating: 0, count: 1024)
        guard let native = rv_ncnn_create(model.path, padding, fp16Math ? 1 : 0, &error, error.count) else {
            throw Failure("NCNN initialization: \(String(cString: error))")
        }
        context = native
    }
    deinit { tasks.wait(); rv_ncnn_destroy(context) }

    func push(_ frame: CVPixelBuffer, timesteps: [Float]) throws -> [FrameResult] {
        let current = try pack(frame)
        defer { previous = current }
        guard let left = previous else { return [] }
        return timesteps.map { phase in
            let result = FrameResult()
            slots.wait(); tasks.enter()
            queue.async { [self] in
                defer { slots.signal(); tasks.leave() }
                result.complete(Result { try autoreleasepool { try process(left, current, phase) } })
            }
            return result
        }
    }
    private func pack(_ frame: CVPixelBuffer) throws -> Data {
        guard CVPixelBufferGetWidth(frame) == width, CVPixelBufferGetHeight(frame) == height,
              CVPixelBufferGetPixelFormatType(frame) == kCVPixelFormatType_32BGRA else { throw Failure("NCNN input layout changed") }
        CVPixelBufferLockBaseAddress(frame, .readOnly)
        defer { CVPixelBufferUnlockBaseAddress(frame, .readOnly) }
        var src = vImage_Buffer(data: CVPixelBufferGetBaseAddress(frame), height: UInt(height), width: UInt(width), rowBytes: CVPixelBufferGetBytesPerRow(frame))
        var packed = Data(count: width * height * 3)
        let status = packed.withUnsafeMutableBytes { bytes in
            var dst = vImage_Buffer(data: bytes.baseAddress, height: UInt(height), width: UInt(width), rowBytes: width * 3)
            return vImageConvert_BGRA8888toRGB888(&src, &dst, vImage_Flags(kvImageNoFlags))
        }
        guard status == kvImageNoError else { throw Failure("NCNN RGB packing failed: \(status)") }
        return packed
    }
    private func process(_ left: Data, _ right: Data, _ phase: Float) throws -> CVPixelBuffer {
        var rgb = Data(count: width * height * 3)
        let status = left.withUnsafeBytes { a in right.withUnsafeBytes { b in rgb.withUnsafeMutableBytes { out in
            rv_ncnn_process(context, a.bindMemory(to: UInt8.self).baseAddress!, b.bindMemory(to: UInt8.self).baseAddress!,
                out.bindMemory(to: UInt8.self).baseAddress!, Int32(width), Int32(height), phase)
        } } }
        guard status == 0 else { throw Failure("NCNN inference failed: \(status)") }
        var optional: CVPixelBuffer?
        guard CVPixelBufferPoolCreatePixelBuffer(nil, pool, &optional) == kCVReturnSuccess, let buffer = optional else {
            throw Failure("NCNN output allocation failed")
        }
        CVPixelBufferLockBaseAddress(buffer, [])
        defer { CVPixelBufferUnlockBaseAddress(buffer, []) }
        var dst = vImage_Buffer(data: CVPixelBufferGetBaseAddress(buffer), height: UInt(height), width: UInt(width), rowBytes: CVPixelBufferGetBytesPerRow(buffer))
        let conversion = rgb.withUnsafeMutableBytes { bytes in
            var src = vImage_Buffer(data: bytes.baseAddress, height: UInt(height), width: UInt(width), rowBytes: width * 3)
            return vImageConvert_RGB888toBGRA8888(&src, nil, 255, &dst, false, vImage_Flags(kvImageNoFlags))
        }
        guard conversion == kvImageNoError else { throw Failure("NCNN BGRA conversion failed") }
        return buffer
    }
}
