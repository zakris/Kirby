import SwiftUI

/// Shared SwiftUI screen used by both the macOS and iOS apps.
public struct TaskListView: View {
    @State private var tasks: [KirbyTask] = []
    @State private var newTitle = ""
    @State private var error: String?
    private let api: APIClient
    private let today = DayString.today()

    public init(api: APIClient = APIClient()) { self.api = api }

    public var body: some View {
        VStack(alignment: .leading) {
            Text("Today").font(.largeTitle).bold()
            HStack {
                TextField("Add a task", text: $newTitle).onSubmit(add)
                Button("Add", action: add).disabled(newTitle.isEmpty)
            }
            if let error { Text(error).foregroundStyle(.red) }
            List(tasks) { task in
                HStack {
                    Toggle(isOn: Binding(get: { task.done }, set: { toggle(task, $0) })) {
                        Text(task.title).strikethrough(task.done)
                    }
                    Spacer()
                    Button(role: .destructive) { remove(task) } label: { Image(systemName: "trash") }
                        .buttonStyle(.borderless)
                }
            }
        }
        .padding()
        .task { await reload() }
    }

    private func reload() async {
        do { tasks = try await api.tasks(on: today); error = nil }
        catch { self.error = "Can't reach backend: \(error.localizedDescription)" }
    }
    private func add() {
        let title = newTitle; guard !title.isEmpty else { return }
        newTitle = ""
        Task { _ = try? await api.create(title: title, date: today); await reload() }
    }
    private func toggle(_ t: KirbyTask, _ done: Bool) {
        Task { _ = try? await api.setDone(t.id, done); await reload() }
    }
    private func remove(_ t: KirbyTask) {
        Task { try? await api.delete(t.id); await reload() }
    }
}
