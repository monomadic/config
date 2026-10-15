// swift-tools-version: 6.2
import PackageDescription
import Foundation
let native = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent(".build-deps/native").path

let package = Package(
    name: "rife-video",
    platforms: [.macOS(.v14)],
    products: [.executable(name: "rife-video", targets: ["rife-video"])],
    dependencies: [
        .package(path: ".build-deps/rife-metal"),
        .package(url: "https://github.com/apple/swift-argument-parser", exact: "1.7.1"),
    ],
    targets: [
        .target(name: "RifeVideoCore"),
        .target(name: "CNCNN", linkerSettings: [.unsafeFlags(["-L", native, "-lrife_native", "-Xlinker", "-rpath", "-Xlinker", "@executable_path", "-Xlinker", "-rpath", "-Xlinker", native])]),
        .executableTarget(name: "rife-video", dependencies: [
            "RifeVideoCore", "CNCNN",
            .product(name: "RifeMetal", package: "rife-metal"),
            .product(name: "ArgumentParser", package: "swift-argument-parser"),
        ]),
        .executableTarget(name: "timeline-tests", dependencies: ["RifeVideoCore"], path: "Tests/RifeVideoCoreTests"),
    ],
    swiftLanguageModes: [.v5]
)
