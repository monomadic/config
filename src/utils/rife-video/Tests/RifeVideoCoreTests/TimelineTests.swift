import Foundation
import CoreMedia
import RifeVideoCore

final class TimelineTests {
    func testRatesAndExactDuration() throws {
        for (input, expectedCopies) in [(24, 12), (25, 5), (30, 30), (50, 10)] {
            var timeline = Timeline(rate: try FrameRate("60"))
            var copies = 0, generated = 0
            // Include a boundary at t=1 so this tests a whole second, excluding final-frame hold.
            for i in 0..<input {
                let plan = try timeline.interval(left: CMTime(value: Int64(i), timescale: Int32(input)),
                                                  right: CMTime(value: Int64(i+1), timescale: Int32(input)))
                copies += plan.filter { $0.phase == nil }.count
                generated += plan.filter { $0.phase != nil }.count
                for item in plan { if let phase = item.phase { checkGreaterThan(phase, 0); checkLessThan(phase, 1) } }
            }
            checkEqual(timeline.nextIndex, 60)
            checkEqual(copies, expectedCopies)
            checkEqual(generated, 60-expectedCopies)
        }
    }
    func test24To60Phases() throws {
        var t = Timeline(rate: try FrameRate("60"))
        let frames = try t.interval(left: .zero, right: CMTime(value: 1, timescale: 24))
        checkEqual(frames.count, 3); checkNil(frames[0].phase)
        checkEqual(frames[1].phase!, 0.4, accuracy: 1e-6)
        checkEqual(frames[2].phase!, 0.8, accuracy: 1e-6)
    }
    func testFractionalAndVFR() throws {
        var t = Timeline(rate: try FrameRate("60000/1001"))
        for i in 0..<24000 {
            _ = try t.interval(left: CMTime(value: Int64(i)*1001, timescale: 24000), right: CMTime(value: Int64(i+1)*1001, timescale: 24000))
        }
        checkEqual(t.nextIndex, 60000) // no drift after 1001 seconds
        var v = Timeline(rate: try FrameRate("60"))
        for (a,b) in [(0,31),(31,80),(80,100)] {
            _ = try v.interval(left: CMTime(value: Int64(a), timescale: 1000), right: CMTime(value: Int64(b), timescale: 1000))
        }
        checkEqual(v.nextIndex, 6)
    }
    func testFinalFrameHeldToDuration() throws {
        var t = Timeline(rate: try FrameRate("60"))
        for i in 0..<234 { _ = try t.interval(left: CMTime(value: Int64(i), timescale: 30), right: CMTime(value: Int64(i+1), timescale: 30)) }
        let held = try t.tail(end: CMTime(value: 235, timescale: 30))
        checkEqual(held.count, 2); checkEqual(t.nextIndex, 470)
    }
    func testBadTimestampsAndRatesFail() throws {
        for rate in ["0", "-1", "60/0", "NaN", "60/", "60/1/2", "241"] { checkThrowsError(try FrameRate(rate)) }
        var t = Timeline(rate: try FrameRate("60"))
        checkThrowsError(try t.interval(left: .zero, right: .zero))
        checkThrowsError(try t.interval(left: CMTime(value: 1,timescale: 1), right: .zero))
        checkThrowsError(try t.interval(left: .zero, right: .invalid))
    }
}

func checkEqual<T: Equatable>(_ a: T, _ b: T) { precondition(a == b, "\(a) != \(b)") }
func checkEqual(_ a: Float, _ b: Float, accuracy: Float) { precondition(abs(a-b) <= accuracy) }
func checkNil<T>(_ a: T?) { precondition(a == nil) }
func checkGreaterThan<T: Comparable>(_ a: T, _ b: T) { precondition(a > b) }
func checkLessThan<T: Comparable>(_ a: T, _ b: T) { precondition(a < b) }
func checkThrowsError<T>(_ body: @autoclosure () throws -> T) {
    do { _ = try body(); fatalError("expected failure") } catch {}
}
let tests = TimelineTests()
try tests.testRatesAndExactDuration()
try tests.test24To60Phases()
try tests.testFractionalAndVFR()
try tests.testFinalFrameHeldToDuration()
try tests.testBadTimestampsAndRatesFail()
print("PASS: rate, phase, fractional/VFR, duration, invalid-input scheduler tests")
