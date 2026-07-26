//! Game core — all non-UI logic lives here.
//!
//! This crate knows nothing about Tauri, the webview, or any UI. That
//! separation is deliberate: it lets the logic be unit-tested, benchmarked,
//! or reused headless without a GUI. The grid, move rules, and win detection
//! for Four In A Row belong here; expose them through a thin command in the
//! `src-tauri` shell once implemented.
//!
//! `greeting` below is the skeleton's wiring sample, not game logic — it
//! proves the Rust core <-> Tauri <-> UI round trip works end to end.

/// Build the greeting shown by the sample UI.
pub fn greeting(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        "Hello from the Rust core!".to_string()
    } else {
        format!("Hello, {name}, from the Rust core!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greets_by_name() {
        assert_eq!(greeting("world"), "Hello, world, from the Rust core!");
    }

    #[test]
    fn falls_back_when_the_name_is_blank() {
        assert_eq!(greeting("  "), "Hello from the Rust core!");
    }
}
