//! Tauri shell — the thin adapter layer.
//!
//! This binary owns the window and exposes the Four In A Row game core to
//! the web UI as a small set of commands. It contains no game logic — that
//! lives in the core crate (grid state, move rules, win detection). Keep
//! this layer thin: translate between core types and the JSON protocol the
//! front end renders, and hand heavy work to
//! `tauri::async_runtime::spawn_blocking` so the UI never freezes.

// Hide the console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;

// ---------------------------------------------------------------------------
// Protocol — the JSON shapes the front end consumes (mirrored in src/api.ts).
// ---------------------------------------------------------------------------

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Greeting {
    message: String,
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Ask the core for a greeting. Skeleton wiring sample — will be replaced by
/// the grid-state and disc-drop commands once the game core is implemented.
#[tauri::command]
fn greet(name: &str) -> Greeting {
    Greeting {
        message: four_in_a_row_core::greeting(name),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running the Tauri application");
}

// The command handlers are thin wrappers over the core crate (where the logic
// and its tests live); what is worth pinning down here is the protocol — the
// exact JSON shapes the front end deserializes (`src/api.ts` mirrors them).
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn greeting_serializes_to_the_wire_shape_the_frontend_expects() {
        let greeting = Greeting {
            message: "hi".to_string(),
        };
        assert_eq!(serde_json::to_value(&greeting).unwrap(), json!({ "message": "hi" }));
    }
}
