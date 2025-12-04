# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs`: Iced desktop UI (Windows) with file picker, list management, merge controls, progress updates.
- `src/lib.rs`: PDF merge logic built on `lopdf` (renumber objects, rebuild page tree, save output); add new core functionality here.
- `src/cli.rs`: Simple CLI wrapper for merging via arguments; mirrors library API.
- `Cargo.toml` / `Cargo.lock`: Dependency versions; keep in sync when adding crates.
- `README*.md`, `RUST-TUTORIAL.md`, `TESTING.md`: User docs and learning material; update if UX or behavior changes.

## Build, Test, and Development Commands
- `cargo run --release`: Launch the desktop app (recommended for realistic performance).
- `cargo run --bin pdf-merger-cli -- <in1.pdf> <in2.pdf> <out.pdf>`: Run the CLI merger.
- `cargo build --release`: Produce Windows-ready binaries under `target/release/`.
- `cargo test`: Run unit tests (currently minimal; add more around merge logic).
- `cargo fmt` and `cargo clippy -- -D warnings`: Format and lint before pushing.

## Coding Style & Naming Conventions
- Rust 2021 edition; prefer explicit types for public APIs and clear error messages.
- Use `snake_case` for functions/vars, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Keep UI logic in `main.rs`; keep PDF manipulation pure in `lib.rs`. Avoid mixing async/UI concerns into the library.
- Handle user-facing errors with actionable messages (e.g., distinguish encrypted vs corrupted PDFs).

## Testing Guidelines
- Framework: built-in `cargo test`. Add focused unit tests for object renumbering, page order, and failure cases (encrypted/empty PDFs).
- Naming: use `test_*` functions grouped by feature; prefer small sample PDFs committed to `tests/` if added.
- For manual checks, follow `TESTING.md`; keep new edge cases documented there.

## Commit & Pull Request Guidelines
- Commits: concise, imperative summary (e.g., `Improve page tree rebuilding`). Squash only if history is noisy.
- PRs: include what changed, why, and validation steps (`cargo test`, manual merge steps). Attach screenshots or notes for UI tweaks; link issues when applicable.

## Security & Configuration Tips
- Validate file paths before merge; never assume PDFs are trusted. Keep error messages non-leaky (no full system paths in logs).
- If adding deps, prefer audited/maintained crates; run `cargo tree -d` to spot duplicates. Document new config/env needs in `README.md`.
