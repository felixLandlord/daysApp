#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "macos"),
    windows_subsystem = "windows"
)]
#![cfg_attr(feature = "bundle", windows_subsystem = "windows")]

mod client;
mod server;

use dioxus::{logger::tracing::Level, prelude::*};
use dioxus_desktop::{tao::window::Fullscreen, Config, WindowBuilder};

use crate::client::app::App;
use crate::server::db::{create_employee_table, create_schedules_table, establish_connection};

fn main() {
    dioxus::logger::init(Level::INFO).expect("failed to init logger");

    let window = WindowBuilder::new()
        .with_title("days-assign")
        .with_always_on_top(false)
        // .with_fullscreen(Some(Fullscreen::Borderless(None)));
        // .with_maximized(true);
        .with_inner_size(dioxus_desktop::tao::dpi::LogicalSize::new(1200, 800));

    let config = Config::new()
        .with_window(window)
        .with_resource_directory("Contents/Resources/assets");

    // Initialize the database connection and tables
    match establish_connection() {
        Ok(conn) => {
            if let Err(e) = create_employee_table(&conn) {
                eprintln!("Failed to create employee table: {}", e);
                // Handle the error appropriately (e.g., exit the application)
            }
            if let Err(e) = create_schedules_table(&conn) {
                eprintln!("Failed to create schedules table: {}", e);
                // Handle the error appropriately (e.g., exit the application)
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            // Handle the error (e.g., exit the application)
        }
    }

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}
