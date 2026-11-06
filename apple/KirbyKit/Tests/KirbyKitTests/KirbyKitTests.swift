import XCTest
@testable import KirbyKit

final class KirbyKitTests: XCTestCase {
    func testDayString() {
        var cal = Calendar(identifier: .gregorian)
        cal.timeZone = TimeZone(identifier: "UTC")!
        XCTAssertEqual(DayString.today(Date(timeIntervalSince1970: 0), calendar: cal), "1970-01-01")
    }

    func testDecodesBackendTask() throws {
        let json = #"{"id":"6f1c3b7e-2f0a-4d57-9a3e-1c2b3d4e5f60","title":"x","date":"2026-01-02","done":false}"#
        let t = try JSONDecoder().decode(KirbyTask.self, from: Data(json.utf8))
        XCTAssertEqual(t.title, "x")
    }
}
