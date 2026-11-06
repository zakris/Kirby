// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "KirbyKit",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [.library(name: "KirbyKit", targets: ["KirbyKit"])],
    targets: [
        .target(name: "KirbyKit"),
        .testTarget(name: "KirbyKitTests", dependencies: ["KirbyKit"]),
    ]
)
