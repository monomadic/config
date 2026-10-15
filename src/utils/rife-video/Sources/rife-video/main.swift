import ArgumentParser
import AVFoundation
import Foundation
import RifeMetal
import RifeVideoCore
import Darwin

@main
struct RifeVideo: AsyncParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "rife-video",
        abstract: "Hardware-decode H.264/HEVC, interpolate with RIFE Metal or NCNN Vulkan, encode HEVC/ProRes/AV1. No PNG intermediates.",
        version: "0.2.0 (Metal 0.1.7 / NCNN 13338e38)")
    @Argument(help: "Input H.264/HEVC video (BT.709 SDR; interpolation is 8-bit).") var input: String
    @Argument(help: "Output .mp4/.mov/.mkv; ProRes requires .mov.") var output: String
    @Option(help: "Target frame rate, integer or rational.") var fps = "60"
    @Option(help: "metal | ncnn (resident Vulkan)") var backend: Backend = .metal
    @Option(help: "NCNN concurrent GPU jobs (1–4); default auto: 1 for full 4K, otherwise 2.") var workers: Int?
    @Flag(help: "Use FP16 arithmetic in NCNN (changes numerical results; benchmark quality before use).") var ncnnFp16 = false
    @Option(help: "hq | balanced | fast") var tier: Tier = .hq
    @Option(help: "hevc (hardware) | prores (hardware) | av1 (software)") var codec: Codec = .hevc
    @Option(help: "ProRes proxy | lt | standard | hq | 4444 | xq") var proresProfile: ProResProfile = .hq
    @Option(help: "HEVC bitrate in Mbps; default 20 at 1080p, scaled by pixel count.") var mbps: Double?
    @Option(help: "SVT-AV1 CRF (0–63).") var crf = 25
    @Option(help: "SVT-AV1 speed preset (0–13).") var preset = 8
    @Option(help: "copy | aac | none; incompatible audio copy fails without replacing the destination.") var audio: Audio = .copy
    @Option(help: "Write benchmark JSON (end-to-end wall time, RIFE calls, encode backpressure, frame counts).") var stats: String?
    @Option(help: "Metal .rmw weights or NCNN model directory override.") var model: String?
    @Flag(help: "Atomically replace an existing output only after successful encoding/muxing.") var overwrite = false

    mutating func validate() throws {
        _ = try FrameRate(fps)
        if ncnnFp16 && backend != .ncnn { throw ValidationError("--ncnn-fp16 requires --backend ncnn") }
        if let workers, !(1...4).contains(workers) { throw ValidationError("--workers must be 1–4") }
        guard (0...63).contains(crf), (0...13).contains(preset) else { throw ValidationError("--crf must be 0–63 and --preset 0–13") }
        if let mbps, !mbps.isFinite || mbps <= 0 || mbps > 1000 { throw ValidationError("--mbps must be > 0 and <= 1000") }
        let source = URL(fileURLWithPath: input).resolvingSymlinksInPath().standardizedFileURL
        let dest = URL(fileURLWithPath: output).resolvingSymlinksInPath().standardizedFileURL
        guard source != dest else { throw ValidationError("input and output must differ") }
        guard FileManager.default.fileExists(atPath: source.path) else { throw ValidationError("input does not exist") }
        guard ["mp4", "mov", "mkv"].contains(dest.pathExtension.lowercased()) else { throw ValidationError("output must be .mp4, .mov or .mkv") }
        guard codec != .prores || dest.pathExtension.lowercased() == "mov" else { throw ValidationError("ProRes requires a .mov output") }
        if !overwrite && FileManager.default.fileExists(atPath: dest.path) { throw ValidationError("output exists; use --overwrite to replace after success") }
        if let stats {
            let url = URL(fileURLWithPath: stats).resolvingSymlinksInPath().standardizedFileURL
            guard url != source, url != dest else { throw ValidationError("--stats must differ from input/output") }
            guard !FileManager.default.fileExists(atPath: url.path) else { throw ValidationError("stats file exists; choose a new --stats path") }
        }
    }

    func run() async throws {
        signal(SIGPIPE, SIG_IGN)
        let start = uptime()
        _ = try executable("ffmpeg")
        let source = URL(fileURLWithPath: input).standardizedFileURL
        let destination = URL(fileURLWithPath: output).standardizedFileURL
        let directory = destination.deletingLastPathComponent().appendingPathComponent(".rife-video-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: false)
        defer { try? FileManager.default.removeItem(at: directory) }
        let rate = try FrameRate(fps)
        var decodeURL = source
        func normalizeContainer() throws -> URL {
            let normalized = directory.appendingPathComponent("decode-source.mov")
            log("Remuxing input video for AVFoundation compatibility (compressed packets only).")
            // hvc1 is needed for HEVC; probe codec without decoding any frames.
            let probe = Process(); probe.executableURL = try executable("ffprobe")
            let pipe = Pipe(); probe.standardOutput = pipe; probe.standardError = FileHandle.nullDevice
            probe.arguments = ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=codec_name", "-of", "csv=p=0", source.path]
            try probe.run()
            let codecName = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
            probe.waitUntilExit()
            guard probe.terminationStatus == 0 else { throw Failure("cannot probe input codec") }
            var remux = ["-copyts", "-i", source.path, "-map", "0:v:0", "-an", "-c:v", "copy", "-avoid_negative_ts", "disabled"]
            if codecName.contains("hevc") { remux += ["-tag:v", "hvc1"] }
            remux.append(normalized.path)
            try runFFmpeg(remux, directory: directory)
            return normalized
        }
        if !["mp4", "mov", "m4v"].contains(source.pathExtension.lowercased()) {
            decodeURL = try normalizeContainer()
        }
        let decoder: HardwareDecoder
        do { decoder = try await HardwareDecoder(url: decodeURL) }
        catch DecoderSetup.remuxHEVC {
            decodeURL = try normalizeContainer()
            decoder = try await HardwareDecoder(url: decodeURL)
        }
        let info = decoder.info
        let gpuWorkers = backend == .ncnn ? (workers ?? (info.width * info.height >= 3840 * 2160 ? 1 : 2)) : 1
        let angle = Double(-atan2(info.transform.b, info.transform.a) * 180 / .pi)
        if codec == .av1 {
            let t = info.transform
            guard abs(t.a * t.d - t.b * t.c - 1) < 0.001,
                  abs(hypot(t.a, t.b) - 1) < 0.001,
                  abs(angle / 90 - (angle / 90).rounded()) < 0.001 else {
                throw Failure("AV1 path supports right-angle rotation only, not mirrored/scaled display transforms")
            }
        }
        log("\(info.width)x\(info.height) \(info.codec) @ \(String(format: "%.3f", info.fps)) → \(fps) fps; \(tier.rawValue), \(codec.rawValue)")
        log("8-bit BT.709 SDR interpolation; 10-bit source precision is reduced. Hardware decode required.")
        let ncnnName = tier == .hq ? "rife-v4.26" : tier == .balanced ? "rife-v4.25" : "rife-v4.25-lite"
        let defaultModel = backend == .metal ? RifeConfiguration.bundledModelURL :
            FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent(".local/share/rife-ncnn-vulkan/\(ncnnName)")
        let modelURL = model.map { URL(fileURLWithPath: $0) } ?? defaultModel
        let stream: InterpolationStream
        if backend == .metal {
            stream = try MetalStream(model: modelURL, tier: tier, width: info.width, height: info.height)
        } else {
            stream = try NCNNStream(model: modelURL, workers: gpuWorkers, fp16Math: ncnnFp16, width: info.width, height: info.height)
        }
        log("Backend: \(backend.rawValue); model: \(modelURL.lastPathComponent); GPU jobs: \(gpuWorkers)")
        let videoURL = directory.appendingPathComponent(codec == .av1 ? "video.mp4" : "video.mov")
        let writer: FrameWriter
        if codec == .av1 {
            writer = try AV1Writer(url: videoURL, directory: directory, info: info, rate: rate, crf: crf, preset: preset)
        } else {
            writer = try HardwareWriter(url: videoURL, info: info, rate: rate, codec: codec, profile: proresProfile, mbps: mbps)
        }
        var finished = false
        defer { if !finished { writer.cancel() } }
        let queue = DecodeQueue(decoder); defer { queue.cancel() }
        guard var previous = try queue.next() else { throw Failure("empty video") }
        let origin = previous.time
        var timeline = Timeline(rate: rate)
        var generated = 0, copied = 0, held = 0, decoded = 1
        var inference = 0.0, appendTime = 0.0, lastProgress = uptime()
        let warm = uptime()
        _ = try stream.push(previous.buffer, timesteps: [])
        let primeTime = uptime() - warm
        var lastInterval = CMTime(value: 1, timescale: max(1, Int32(info.fps.rounded())))
        var pending: [(CMTime, FrameResult)] = []
        let pendingLimit = backend == .ncnn ? gpuWorkers * 3 : 1
        func drainOne() async throws {
            let (time, result) = pending.removeFirst()
            let wait = uptime(); let buffer = try result.get(); inference += uptime() - wait
            let w = uptime(); try await writer.append(buffer, at: time); appendTime += uptime() - w
        }
        while let current = try queue.next() {
            decoded += 1
            let left = CMTimeSubtract(previous.time, origin), right = CMTimeSubtract(current.time, origin)
            lastInterval = CMTimeSubtract(current.time, previous.time)
            // Protect bounded buffering if the source has an hours-long timestamp discontinuity.
            guard lastInterval.seconds <= 10 else { throw Failure("source timestamp gap exceeds 10 seconds") }
            let plan = try timeline.interval(left: left, right: right)
            guard plan.count <= 16 else { throw Failure("source gap requires more than 16 output frames in one interval") }
            let phases = plan.compactMap(\.phase)
            let t = uptime()
            let results = try autoreleasepool { try stream.push(current.buffer, timesteps: phases) }
            inference += uptime() - t
            var index = 0
            for item in plan {
                let buffer: FrameResult
                if item.phase == nil { buffer = FrameResult(previous.buffer); copied += 1 }
                else { buffer = results[index]; index += 1; generated += 1 }
                pending.append((item.time, buffer))
                if pending.count >= pendingLimit { try await drainOne() }
            }
            previous = current
            if uptime() - lastProgress > 1 {
                log("\(timeline.nextIndex) output / \(generated) synthesized; \(String(format: "%.1f", Double(timeline.nextIndex) / (uptime() - start))) output fps end-to-end so far")
                lastProgress = uptime()
            }
        }
        while !pending.isEmpty { try await drainOne() }
        let duration = previous.duration.isNumeric && previous.duration.seconds > 0 ? previous.duration : lastInterval
        var end = CMTimeSubtract(CMTimeAdd(previous.time, duration), origin)
        if info.end.isNumeric && CMTimeCompare(info.end, previous.time) > 0 {
            end = CMTimeSubtract(info.end, origin)
        }
        guard end.seconds - CMTimeSubtract(previous.time, origin).seconds <= 10 else { throw Failure("source tail exceeds 10 seconds") }
        for item in try timeline.tail(end: end) {
            let w = uptime(); try await writer.append(previous.buffer, at: item.time); appendTime += uptime() - w
            held += 1
        }
        let outputEnd = rate.time(timeline.nextIndex)
        let flushStart = uptime()
        try await writer.finish(at: outputEnd); finished = true
        let flushTime = uptime() - flushStart
        let encodeWall = uptime() - start
        let finalURL = directory.appendingPathComponent("final.\(destination.pathExtension.lowercased())")
        var args = ["-copyts"]
        // Native writer already preserves preferredTransform; AV1's raw pipe needs the display matrix restored.
        if codec == .av1 && abs(angle) > 0.001 { args += ["-display_rotation:v:0", String(angle)] }
        args += ["-i", videoURL.path]
        if audio != .none {
            args += ["-itsoffset", String(-origin.seconds), "-i", source.path, "-map", "0:v:0", "-map", "1:a?",
                     "-map_metadata", "1", "-map_chapters", "-1", "-c:v", "copy", "-c:a", audio == .copy ? "copy" : "aac"]
            if audio == .aac { args += ["-b:a", "192k"] }
        } else { args += ["-map", "0:v:0", "-c:v", "copy", "-an"] }
        // Preserve encoder priming/negative DTS; an output -ss would discard the initial keyframe.
        // Bound audio to the output video duration.
        args += ["-t", String(outputEnd.seconds), "-avoid_negative_ts", "disabled"]
        if codec == .hevc && destination.pathExtension.lowercased() != "mkv" { args += ["-tag:v", "hvc1"] }
        if destination.pathExtension.lowercased() != "mkv" { args += ["-movflags", "+faststart"] }
        args.append(finalURL.path)
        let muxStart = uptime(); try runFFmpeg(args, directory: directory); let muxTime = uptime() - muxStart
        let finalAsset = AVURLAsset(url: finalURL)
        // AVFoundation cannot inspect Matroska; FFmpeg handles that container.
        if destination.pathExtension.lowercased() != "mkv" {
            guard let finalTrack = try await finalAsset.loadTracks(withMediaType: .video).first else {
                throw Failure("mux produced no video track")
            }
            let finalRange = try await finalTrack.load(.timeRange)
            guard abs(finalRange.duration.seconds - outputEnd.seconds) < 1 / rate.value + 0.002 else {
                throw Failure("mux changed video duration unexpectedly")
            }
        }
        try publish(finalURL, to: destination, overwrite: overwrite)
        let wall = uptime() - start
        let report: [String: Any] = [
            "schema": 2, "ncnn_fp16_arithmetic": backend == .ncnn && ncnnFp16, "backend": backend.rawValue, "workers": gpuWorkers, "input": source.path, "output": destination.path,
            "width": info.width, "height": info.height, "input_nominal_fps": info.fps,
            "target_fps": fps, "tier": tier.rawValue, "codec": codec.rawValue,
            "model": modelURL.path, "engine_version": backend == .metal ? "0.1.7" : "13338e38/ncnn-ec19da2b",
            "model_version": backend == .metal ? "4.26" : modelURL.lastPathComponent,
            "hardware_decode_verified": decoder.hardwareVerified, "hardware_encode_required": codec != .av1,
            "input_frames": decoded, "output_frames": timeline.nextIndex, "generated_frames": generated,
            "copied_frames": copied, "tail_frames": held, "duration_s": outputEnd.seconds,
            "wall_s": wall, "through_encode_wall_s": encodeWall, "decode_call_s": decoder.decodeSeconds,
            "stream_prime_s": primeTime, "interpolation_call_s": inference, "writer_append_s": appendTime,
            "encoder_flush_s": flushTime, "mux_s": muxTime,
            "output_fps_end_to_end": Double(timeline.nextIndex) / wall,
            "generated_fps_interpolation": Double(generated) / max(inference, 0.000001),
            "timing_note": "Stages overlap; stage times must not be summed. Interpolation time is host submission plus ordered result waits, not summed GPU service time. Includes warm-up/cache rebases. Wall includes probe, model load, decode, interpolation, encode, audio mux, publish.",
        ]
        if let stats {
            let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
            // No-clobber avoids overwriting unrelated data in a race after argument validation.
            try data.write(to: URL(fileURLWithPath: stats), options: .withoutOverwriting)
        }
        log("Wrote \(timeline.nextIndex) frames (\(generated) synthesized) in \(String(format: "%.2f", wall))s — \(String(format: "%.2f", Double(timeline.nextIndex) / wall)) output fps; hardware decode verified.")
        log(destination.path)
    }
}
