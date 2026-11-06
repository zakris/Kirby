import Foundation

public struct KirbyTask: Codable, Identifiable, Equatable, Sendable {
    public let id: UUID
    public var title: String
    /// `YYYY-MM-DD`
    public var date: String
    public var done: Bool
}

public enum DayString {
    public static func today(_ now: Date = Date(), calendar: Calendar = .current) -> String {
        let c = calendar.dateComponents([.year, .month, .day], from: now)
        return String(format: "%04d-%02d-%02d", c.year!, c.month!, c.day!)
    }
}
