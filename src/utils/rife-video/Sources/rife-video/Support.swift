import AVFoundation
import ArgumentParser
import Foundation
import Darwin

struct Failure: Error, CustomStringConvertible {
    let description: String
    init(_ description: String) { self.description = description }
}
func uptime() -> Double { ProcessInfo.processInfo.systemUptime }
func log(_ text: String) { FileHandle.standardError.write(Data((text + "\n").utf8)) }
func logTail(_ url: URL) -> String {
    guard let file = try? FileHandle(forReadingFrom: url) else { return "no encoder log" }
    defer { try? file.close() }
    let end = (try? file.seekToEnd()) ?? 0
    try? file.seek(toOffset: end > 8000 ? end - 8000 : 0)
    return String(data: (try? file.readToEnd()) ?? Data(), encoding: .utf8) ?? "unreadable encoder log"
}
func executable(_ name: String) throws -> URL {
    let paths = (ProcessInfo.processInfo.environment["PATH"] ?? "/opt/homebrew/bin:/usr/bin:/bin").split(separator: ":")
    for path in paths {
        let url = URL(fileURLWithPath: String(path)).appendingPathComponent(name)
        if FileManager.default.isExecutableFile(atPath: url.path) { return url }
    }
    throw Failure("\(name) is required on PATH")
}
func runFFmpeg(_ arguments: [String], directory: URL) throws {
    let url = directory.appendingPathComponent("mux.log")
    FileManager.default.createFile(atPath: url.path, contents: nil)
    let file = try FileHandle(forWritingTo: url); defer { try? file.close() }
    let p = Process(); p.executableURL = try executable("ffmpeg")
    p.arguments = ["-hide_banner", "-loglevel", "warning", "-nostdin", "-y"] + arguments
    p.standardInput = FileHandle.nullDevice; p.standardOutput = FileHandle.nullDevice; p.standardError = file
    try p.run(); p.waitUntilExit()
    guard p.terminationStatus == 0 else { throw Failure("ffmpeg: \(logTail(url))") }
}
enum Codec: String, ExpressibleByArgument { case hevc, prores, av1 }
enum Tier: String, ExpressibleByArgument { case hq, balanced, fast }
enum Audio: String, ExpressibleByArgument { case copy, aac, none }
enum ProResProfile: String, ExpressibleByArgument {
    case proxy, lt, standard, hq, p4444 = "4444", xq
    var avCodec: AVVideoCodecType {
        switch self {
        case .proxy: return .proRes422Proxy
        case .lt: return .proRes422LT
        case .standard: return .proRes422
        case .hq: return .proRes422HQ
        case .p4444: return .proRes4444
        case .xq: return AVVideoCodecType(rawValue: "ap4x")
        }
    }
}
func publish(_ source: URL, to destination: URL, overwrite: Bool) throws {
    if overwrite {
        guard Darwin.rename(source.path, destination.path) == 0 else { throw Failure("cannot publish output: \(String(cString: strerror(errno)))") }
    } else {
        // Same-directory staging allows atomic, no-clobber publication even if another job races us.
        guard Darwin.link(source.path, destination.path) == 0 else { throw Failure("cannot publish without overwriting: \(String(cString: strerror(errno)))") }
        try FileManager.default.removeItem(at: source)
    }
}
