import CoreMedia
import Foundation

public struct FrameRate: Equatable {
    public let numerator: Int32
    public let denominator: Int32
    public init(_ text: String) throws {
        let parts = text.split(separator: "/", omittingEmptySubsequences: false)
        guard (1...2).contains(parts.count), let n = Int32(parts[0]),
              let d = parts.count == 2 ? Int32(parts[1]) : 1,
              n > 0, d > 0, Double(n) / Double(d) <= 240 else {
            throw TimelineError("fps must be a positive rational <= 240 (60 or 60000/1001)")
        }
        numerator = n; denominator = d
    }
    public var value: Double { Double(numerator) / Double(denominator) }
    public func time(_ index: Int64) -> CMTime { CMTime(value: index * Int64(denominator), timescale: numerator) }
}

public struct ScheduledFrame {
    public let index: Int64
    public let time: CMTime
    /// Nil means copy the left source frame; otherwise synthesize at this phase.
    public let phase: Float?
}

/// Half-open source intervals [left, right); exact boundary frames are copied.
/// Rational CMTime comparisons avoid drift and accidental near-endpoint interpolation.
public struct Timeline {
    public let rate: FrameRate
    public private(set) var nextIndex: Int64 = 0
    public init(rate: FrameRate) { self.rate = rate }
    public mutating func interval(left: CMTime, right: CMTime) throws -> [ScheduledFrame] {
        guard left.isNumeric, right.isNumeric, CMTimeCompare(right, left) > 0 else {
            throw TimelineError("source timestamps must be finite and strictly increasing")
        }
        var result: [ScheduledFrame] = []
        while CMTimeCompare(rate.time(nextIndex), right) < 0 {
            let t = rate.time(nextIndex)
            guard CMTimeCompare(t, left) >= 0 else { throw TimelineError("source timeline gap or scheduler discontinuity") }
            let phase: Float? = CMTimeCompare(t, left) == 0 ? nil :
                Float(CMTimeSubtract(t, left).seconds / CMTimeSubtract(right, left).seconds)
            // Float may round a rational very close to an endpoint; clamp to open interval.
            result.append(ScheduledFrame(index: nextIndex, time: t,
                                         phase: phase.map { min(max($0, Float.leastNonzeroMagnitude), Float(1).nextDown) }))
            nextIndex += 1
        }
        return result
    }
    /// Hold the final source frame until its sample duration ends, never extrapolate motion.
    public mutating func tail(end: CMTime) throws -> [ScheduledFrame] {
        guard end.isNumeric, end.seconds > 0 else { throw TimelineError("invalid source end time") }
        var result: [ScheduledFrame] = []
        while CMTimeCompare(rate.time(nextIndex), end) < 0 {
            result.append(ScheduledFrame(index: nextIndex, time: rate.time(nextIndex), phase: nil))
            nextIndex += 1
        }
        return result
    }
}
public struct TimelineError: Error, CustomStringConvertible {
    public let description: String
    public init(_ description: String) { self.description = description }
}
