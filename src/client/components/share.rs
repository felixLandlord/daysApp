use crate::server::schema::MonthlySchedule;
// use chrono::Month;
use dioxus::{
    logger::tracing::{error, info},
    prelude::*,
};
// use std::{
//     collections::{HashMap, HashSet},
//     error::Error,
// };
use crate::server::export::{
    generate_csv_data, generate_xlsx_data, save_csv_with_dialog, save_xlsx_with_dialog,
};

const SHARE_CSS: Asset = asset!("/assets/styles/share.css");

#[component]
pub fn ShareButton(schedule: MonthlySchedule, year: i32, month: u32) -> Element {
    // Clone data needed for the async task
    let schedule_clone = schedule.clone(); // Clone schedule for the async block

    let handle_click = move |_| {
        // Clone again for the spawned task if necessary, or use the outer clone
        let schedule_for_task = schedule_clone.clone();
        spawn(async move {
            info!("Generate & Save CSV button clicked.");
            match generate_xlsx_data(&schedule_for_task, year, month) {
                // Ok((filename, csv_data)) => match save_csv_with_dialog(filename, csv_data).await {
                Ok((filename, xlsx_data)) => match save_xlsx_with_dialog(filename, xlsx_data).await
                {
                    Ok(_) => info!("CSV save process completed."),
                    Err(e) => error!("Failed during CSV save dialog/write: {}", e),
                },
                Err(e) => {
                    error!("Failed to generate CSV data: {}", e);
                }
            }
        });
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: SHARE_CSS,
        }
        button {
            class: "btn btn-share",
            onclick: handle_click,

            "Share"
        }
    }
}
