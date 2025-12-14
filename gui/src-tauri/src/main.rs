#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
  // Call the 'run' function from your local library (focusd_dashboard)
  focusd_dashboard::run();
}