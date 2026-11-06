# Kirby
Kirby is a Streaks Killer with MCP Support — a daily task tracker.

## Layout
| Dir | What | Tech |
|---|---|---|
| `backend/` | REST API (`/tasks`, in-memory for now) | Rust, axum |
| `web/` | Web app, deployed to GitHub Pages | Static HTML/JS (no build step) |
| `apple/KirbyKit/` | Shared models, API client, SwiftUI screen | Swift package |
| `apple/macOS/`, `apple/iOS/` | Native app entry points | SwiftUI |

## Run
```sh
# backend (needs Rust: https://rustup.rs)
cd backend && cargo run          # http://127.0.0.1:8080

# web
cd web && python3 -m http.server 3000

# Apple apps (needs full Xcode + `brew install xcodegen`)
cd apple && xcodegen && open Kirby.xcodeproj
cd apple/KirbyKit && swift test
```

Web deploys via `.github/workflows/pages.yml` (enable Pages → "GitHub Actions" source). Point `web/config.js` at the deployed backend.
