use crate::client::routes::Route;
use dioxus::{logger::tracing::info, prelude::*};

const NAVBAR_CSS: Asset = asset!("/assets/styles/navbar.css");

// const LOGO_ICON: Asset = asset!("/assets/icons/logo.svg");
const EMPLOYEES_ICON: Asset = asset!("/assets/icons/employees.svg");
const SCHEDULES_ICON: Asset = asset!("/assets/icons/schedules.svg");
const SETTINGS_ICON: Asset = asset!("/assets/icons/settings.svg");

#[component]
pub fn NavBar() -> Element {
    let current_route = use_route::<Route>();
    use_effect(|| {
        info!("Nav bar loaded");
    });

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: NAVBAR_CSS,
        }

        div {
            id: "navbar",
            // Navigation Links
            div { class: "nav-links",
                Link {
                    to: Route::EmployeesPage {},
                    class: (current_route == Route::EmployeesPage {}).then_some("active").unwrap_or(""),
                    div { class: "nav-item",
                        img {
                            class: "nav-icon",
                            src: "{EMPLOYEES_ICON}",
                            alt: "Employees Icon"
                        }
                        // span { "Employees" }
                    }
                }
                Link {
                    to: Route::SchedulesPage {},
                    class: (current_route == Route::SchedulesPage {}).then_some("active").unwrap_or(""),
                    div { class: "nav-item",
                        img {
                            class: "nav-icon",
                            src: "{SCHEDULES_ICON}",
                            alt: "Schedules Icon"
                        }
                        // span { "Schedules" }
                    }
                }
                Link {
                    to: Route::SettingsPage {},
                    class: (current_route == Route::SettingsPage {}).then_some("active").unwrap_or(""),
                    div { class: "nav-item",
                        img {
                            class: "nav-icon",
                            src: "{SETTINGS_ICON}",
                            alt: "Settings Icon"
                        }
                        // span { "Settings" }
                    }
                }
            }
        }

        // Main content wrapper
        div { class: "main-content",
            Outlet::<Route> {}
        }
    }
}
