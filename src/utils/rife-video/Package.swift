// swift-tools-version: 6.2
import PackageDescription

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
        .executableTarget(name: "rife-video", dependencies: [
            "RifeVideoCore",
            .product(name: "RifeMetal", package: "rife-metal"),
            .product(name: "ArgumentParser", package: "swift-argument-parser"),
        ]),
        .executableTarget(name: "timeline-tests", dependencies: ["RifeVideoCore"], path: "Tests/RifeVideoCoreTests"),
    ],
    swiftLanguageModes: [.v5]
)
