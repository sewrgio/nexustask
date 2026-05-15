# NexusTask Enterprise

This is the re-implemented NexusTask system according to the "INFORME FINAL DEL PROYECTO NEXUSTASK".

## Features Implemented:
- **Hexagonal Architecture**: Clear separation between Domain, Application, and Infrastructure layers.
- **Pure Rust Stack**: 
  - **GUI**: egui (Immediate Mode GUI)
  - **API**: Axum (Asynchronous REST API)
  - **Database**: SQLite with Nested Set and JSON support.
- **Enterprise Schema**: 
  - Workspaces (Multi-team isolation)
  - Tasks with Recursive Hierarchies (Nested Set)
  - Threaded Comments
  - Unified Audit Log
  - Custom Fields (JSON-based)
  - User Sessions and RBAC roles.
- **Single Executable**: The main application starts both the GUI and the API server in parallel.

## Project Structure:
- `src/domain`: Business entities and logic.
- `src/application`: Use cases and services.
- `src/ports`: Interfaces for adapters.
- `src/infrastructure`: Concrete implementations (SQLite, Axum, egui).

## How to run:
(Requires Rust/Cargo installed)
```bash
cargo run
```

The API server will listen on `localhost:8765`.
The database will be created as `nexustask.db`.
