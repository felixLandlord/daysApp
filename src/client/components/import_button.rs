use crate::server::{db, import};
use dioxus::{
    logger::tracing::{error, info},
    prelude::*,
};
use std::error::Error;

const IMPORT_CSS: Asset = asset!("/assets/styles/import.css");

async fn open_json_file_dialog() -> Result<Option<String>, Box<dyn Error>> {
    info!("Opening file dialog for JSON import");

    let file_handle = rfd::AsyncFileDialog::new()
        .add_filter("JSON", &["json"])
        .set_title("Import Employees from JSON")
        .pick_file()
        .await;

    match file_handle {
        Some(handle) => {
            info!("Reading JSON file: {:?}", handle.path());
            let content = handle.read().await;
            let json_str = String::from_utf8(content)?;
            Ok(Some(json_str))
        }
        None => {
            info!("JSON import cancelled by user");
            Ok(None)
        }
    }
}

#[component]
pub fn ImportButton() -> Element {
    let mut import_status = use_signal(|| None::<String>);
    let mut is_importing = use_signal(|| false);

    let handle_import = move |_| {
        if *is_importing.read() {
            return;
        }

        is_importing.set(true);
        import_status.set(None);

        spawn(async move {
            match open_json_file_dialog().await {
                Ok(Some(json_data)) => {
                    info!("JSON file loaded, processing data");

                    match import::import_employees_from_json(&json_data) {
                        Ok(employees) => {
                            let employee_count = employees.len();
                            info!("Successfully parsed {} employees from JSON", employee_count);

                            match db::establish_connection() {
                                Ok(conn) => {
                                    match import::save_imported_employees(&conn, employees) {
                                        Ok(count) => {
                                            import_status.set(Some(format!(
                                                "Successfully imported {} employees",
                                                count
                                            )));
                                        }
                                        Err(e) => {
                                            error!("Failed to save imported employees: {}", e);
                                            import_status.set(Some(format!(
                                                "Error saving employees: {}",
                                                e
                                            )));
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to connect to database: {}", e);
                                    import_status
                                        .set(Some(format!("Database connection error: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse JSON data: {}", e);
                            import_status.set(Some(format!("Invalid JSON format: {}", e)));
                        }
                    }
                }
                Ok(None) => {
                    // User cancelled the import
                    info!("Import cancelled by user");
                }
                Err(e) => {
                    error!("Error reading file: {}", e);
                    import_status.set(Some(format!("Error reading file: {}", e)));
                }
            }

            is_importing.set(false);
        });
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: IMPORT_CSS,
        }
        div { class: "import-container",
            button {
                class: "button import",
                disabled: *is_importing.read(),
                onclick: handle_import,
                if *is_importing.read() {
                    "Importing..."
                } else {
                    "Import Employees"
                }
            }

            // Show status message if available
            if let Some(status) = import_status.read().as_ref() {
                div {
                    class: format!("import-status {}",
                        if status.contains("Successfully") { "success" } else { "error" }
                    ),
                    "{status}"
                }
            }
        }
    }
}
