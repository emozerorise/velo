// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if velo_lib::transcript::worker::run_if_requested() {
        return;
    }
    velo_lib::run();
}
