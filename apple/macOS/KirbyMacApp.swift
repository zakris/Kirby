import SwiftUI
import KirbyKit

@main
struct KirbyMacApp: App {
    var body: some Scene {
        WindowGroup { TaskListView().frame(minWidth: 360, minHeight: 420) }
    }
}
