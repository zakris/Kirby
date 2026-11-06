import Foundation

public struct APIClient: Sendable {
    /// Hosted backend (Render + Neon).
    public static let productionURL = URL(string: "https://kirby-backend-miv5.onrender.com")!

    /// `KIRBY_API_URL` (e.g. `http://127.0.0.1:8080`, set in the Xcode scheme's environment)
    /// overrides the hosted backend for local development.
    public static func configuredBaseURL(_ env: [String: String] = ProcessInfo.processInfo.environment) -> URL {
        env["KIRBY_API_URL"].flatMap(URL.init(string:)) ?? productionURL
    }

    /// Generous timeouts: Render's free tier can take about a minute to wake from idle.
    public static let defaultSession: URLSession = {
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 90
        return URLSession(configuration: config)
    }()

    public var baseURL: URL
    public var session: URLSession

    public init(baseURL: URL = APIClient.configuredBaseURL(), session: URLSession = APIClient.defaultSession) {
        self.baseURL = baseURL
        self.session = session
    }

    public func tasks(on date: String) async throws -> [KirbyTask] {
        var comps = URLComponents(url: baseURL.appendingPathComponent("tasks"), resolvingAgainstBaseURL: false)!
        comps.queryItems = [URLQueryItem(name: "date", value: date)]
        return try await send(URLRequest(url: comps.url!))
    }

    public func create(title: String, date: String) async throws -> KirbyTask {
        try await send(request("tasks", method: "POST", body: ["title": title, "date": date]))
    }

    public func setDone(_ id: UUID, _ done: Bool) async throws -> KirbyTask {
        try await send(request("tasks/\(id.uuidString.lowercased())", method: "PATCH", body: ["done": done]))
    }

    public func delete(_ id: UUID) async throws {
        _ = try await session.data(for: request("tasks/\(id.uuidString.lowercased())", method: "DELETE", body: Optional<[String: String]>.none))
    }

    private func request<B: Encodable>(_ path: String, method: String, body: B?) -> URLRequest {
        var r = URLRequest(url: baseURL.appendingPathComponent(path))
        r.httpMethod = method
        if let body {
            r.setValue("application/json", forHTTPHeaderField: "Content-Type")
            r.httpBody = try? JSONEncoder().encode(body)
        }
        return r
    }

    private func send<T: Decodable>(_ r: URLRequest) async throws -> T {
        let (data, resp) = try await session.data(for: r)
        guard (resp as? HTTPURLResponse)?.statusCode ?? 0 < 300 else { throw URLError(.badServerResponse) }
        return try JSONDecoder().decode(T.self, from: data)
    }
}
