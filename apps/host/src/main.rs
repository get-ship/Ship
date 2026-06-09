// Thin entry shim — all app wiring lives in the lib so the same logic can back
// desktop and (later) mobile targets. See `lib.rs::run`.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ship_host_lib::run();
}
