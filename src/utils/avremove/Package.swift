// swift-tools-version:6.2
import PackageDescription

let package = Package(
    name: "avremove",
    platforms: [.macOS(.v26)],
    dependencies: [
        .package(url: "https://github.com/apple/swift-argument-parser", from: "1.5.0"),
    ],
    targets: [
        .executableTarget(
            name: "avremove",
            dependencies: [.product(name: "ArgumentParser", package: "swift-argument-parser")]
        ),
    ],
    swiftLanguageModes: [.v5]
)
