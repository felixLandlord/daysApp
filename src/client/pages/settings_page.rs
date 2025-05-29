use crate::client::components::ImportButton;
use crate::server::db;
use dioxus::{
    logger::tracing::{error, info},
    prelude::*,
};

const SETTINGS_CSS: Asset = asset!("/assets/styles/settings.css");

// Define a component for the confirmation modal
#[component]
fn ConfirmModal(
    show: Signal<bool>, // Signal to control visibility
    title: String,
    message: String,
    on_confirm: EventHandler<()>, // Use EventHandler instead of Signal for callbacks
) -> Element {
    // Return empty fragment instead of None when not showing
    if !*show.read() {
        return rsx!({});
    }

    let close_modal = move |_| show.set(false);

    rsx! {
        // Overlay
        div {
            class: "modal-overlay",
            onclick: close_modal, // Close modal when clicking outside
            // Modal Content
            div {
                class: "modal-content",
                onclick: move |event| event.stop_propagation(), // Prevent clicks inside from closing modal
                h2 { "{title}" }
                p { "{message}" }
                div {
                    class: "modal-buttons",
                    button {
                        class: "modal-button cancel",
                        onclick: close_modal,
                        "Cancel"
                    }
                    button {
                        class: "modal-button confirm",
                        onclick: move |_| {
                            on_confirm.call(());  // Call the confirm action
                            show.set(false); // Close the modal
                        },
                        "Confirm"
                    }
                }
            }
        }
    }
}

#[component]
pub fn SettingsPage() -> Element {
    use_effect(|| {
        info!("Settings page loaded");
    });

    // State to control modal visibility
    let mut show_employee_confirm_modal = use_signal(|| false);
    let mut show_schedule_confirm_modal = use_signal(|| false);

    // State to track when an action has been triggered
    let employee_data_cleared = use_signal(|| false);
    let schedule_data_cleared = use_signal(|| false);

    // Event handlers for clearing data
    let clear_employee_data = move |_: ()| {
        to_owned![employee_data_cleared];
        employee_data_cleared.set(true);
    };

    let clear_schedule_data = move |_: ()| {
        to_owned![schedule_data_cleared];
        schedule_data_cleared.set(true);
    };

    // Effect for clearing employee data
    use_effect(move || {
        // Only trigger the effect when the flag is set to true
        if !*employee_data_cleared.read() {
            return;
        }

        info!("Attempting to clear employee data...");

        // Create a separate effect to reset the flag after the operation
        let mut employee_data_cleared_clone = employee_data_cleared.clone();

        // Spawn a task to handle the database operation
        spawn(async move {
            match db::establish_connection() {
                Ok(conn) => match db::delete_all_employees(&conn) {
                    Ok(_) => {
                        info!("All employee data cleared successfully.");
                        // Reset the flag after the operation completes
                        employee_data_cleared_clone.set(false);
                    }
                    Err(e) => {
                        error!("Failed to clear employee data: {}", e);
                        employee_data_cleared_clone.set(false);
                    }
                },
                Err(e) => {
                    error!(
                        "Failed to connect to database for clearing employees: {}",
                        e
                    );
                    employee_data_cleared_clone.set(false);
                }
            }
        });
    });

    // Effect for clearing schedule data
    use_effect(move || {
        // Only trigger the effect when the flag is set to true
        if !*schedule_data_cleared.read() {
            return;
        }

        info!("Attempting to clear schedule data...");

        // Create a separate effect to reset the flag after the operation
        let mut schedule_data_cleared_clone = schedule_data_cleared.clone();

        // Spawn a task to handle the database operation
        spawn(async move {
            match db::establish_connection() {
                Ok(conn) => match db::delete_all_schedules(&conn) {
                    Ok(_) => {
                        info!("All schedule data cleared successfully.");
                        // Reset the flag after the operation completes
                        schedule_data_cleared_clone.set(false);
                    }
                    Err(e) => {
                        error!("Failed to clear schedule data: {}", e);
                        schedule_data_cleared_clone.set(false);
                    }
                },
                Err(e) => {
                    error!(
                        "Failed to connect to database for clearing schedules: {}",
                        e
                    );
                    schedule_data_cleared_clone.set(false);
                }
            }
        });
    });

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: SETTINGS_CSS,
        }

        div { class: "settings-container",

            h1 { "Settings" }

            div { class: "settings-section data-management-section",
                h2 { "Data Management" }

                div { class: "import-section",
                    ImportButton {}
                }

                div { class: "settings-actions",
                    button {
                        class: "button danger",
                        onclick: move |_| show_employee_confirm_modal.set(true),
                        "Clear Employee Data"
                    }
                    button {
                         class: "button danger",
                         onclick: move |_| show_schedule_confirm_modal.set(true),
                         "Clear Schedule Data"
                    }
                }
            }
        }

        // Confirmation Modals (Conditionally rendered)
        ConfirmModal {
            show: show_employee_confirm_modal.clone(),
            title: "Confirm Clear Employee Data".to_string(),
            message: "Are you sure you want to permanently delete ALL employee data? This action cannot be undone.".to_string(),
            on_confirm: clear_employee_data, // Pass the event handler
        }

         ConfirmModal {
            show: show_schedule_confirm_modal.clone(),
            title: "Confirm Clear Schedule Data".to_string(),
            message: "Are you sure you want to permanently delete ALL schedule data? This action cannot be undone.".to_string(),
            on_confirm: clear_schedule_data, // Pass the event handler
        }
    }
}
