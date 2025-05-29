use crate::client::components::{SearchBar, ShareButton};
use crate::server::{
    db::{establish_connection, get_all_employees, load_schedule_from_db, save_schedule_to_db},
    scheduler::generate_balanced_schedule,
    schema::{Employee, MonthlySchedule, Weekday},
};
use chrono::{Datelike, Local, Month, NaiveDate};
use dioxus::{
    html::FormData,
    logger::tracing::{error, info},
    prelude::*,
};
use std::collections::{HashMap, HashSet};

const SCHEDULES_CSS: Asset = asset!("/assets/styles/schedules.css");
const EDIT_ICON: Asset = asset!("/assets/icons/edit.svg");
const X_CLOSE_ICON: Asset = asset!("/assets/icons/x-close.svg");
const ARROW_RIGHT_ICON: Asset = asset!("/assets/icons/arrow-right.svg");
const ARROW_LEFT_ICON: Asset = asset!("/assets/icons/arrow-left.svg");

#[derive(PartialEq, Clone)]
enum ModalView {
    None,
    Month,
    Year,
    EmployeeDetails(usize),       // Employee ID
    EditSchedule(Weekday, usize), // Original Day (can be ignored if needed), Employee ID
}

#[component]
pub fn SchedulesPage() -> Element {
    // --- State Signals ---
    let employees = use_signal(|| match establish_connection() {
        Ok(conn) => match get_all_employees(&conn) {
            Ok(emps) => {
                info!("Loaded {} employees", emps.len());
                emps
            }
            Err(e) => {
                error!("Failed to load employees: {}", e);
                Vec::new()
            }
        },
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            Vec::new()
        }
    });

    let mut current_schedule: Signal<Option<MonthlySchedule>> = use_signal(|| None);
    let mut edit_days: Signal<HashSet<Weekday>> = use_signal(HashSet::new);
    let mut search_query = use_signal(String::new);
    let mut is_generating = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);
    let mut modal_view = use_signal(|| ModalView::None);
    let mut selected_employee = use_signal(|| None::<usize>);

    // --- Date State ---
    let now = Local::now();
    let mut selected_year = use_signal(|| now.year());
    let mut selected_month = use_signal(|| now.month()); // u32
    let mut past_schedules_modal = use_signal(|| HashMap::<usize, Vec<HashSet<Weekday>>>::new());

    // --- Effects ---
    use_effect(move || {
        let year = selected_year();
        let month = selected_month();
        info!("Loading schedule for {}-{}", month, year);
        error_message.set(None);
        current_schedule.set(None);

        spawn(async move {
            match establish_connection() {
                Ok(conn) => match load_schedule_from_db(&conn, year, month) {
                    Ok(Some(schedule)) => {
                        info!("Loaded existing schedule from DB for {}-{}", month, year);
                        current_schedule.set(Some(schedule));
                    }
                    Ok(None) => {
                        info!("No existing schedule found in DB for {}-{}", month, year);
                    }
                    Err(e) => {
                        error!("Failed to load schedule for {}-{}: {}", month, year, e);
                        error_message.set(Some(format!("Failed to load schedule: {}", e)));
                    }
                },
                Err(e) => {
                    error!("Failed to connect to database for loading schedule: {}", e);
                    error_message.set(Some("Database connection error while loading.".to_string()));
                }
            }
        });
    });

    // --- Memos ---
    let _filtered_employees = use_memo(move || {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            employees().clone()
        } else {
            employees()
                .iter()
                .filter(|emp| emp.name.to_lowercase().contains(&query))
                .cloned()
                .collect::<Vec<_>>()
        }
    });

    let day_counts = use_memo(move || {
        let mut counts: HashMap<Weekday, usize> = HashMap::new();
        if let Some(schedule) = &*current_schedule.read() {
            for day in Weekday::values() {
                counts.insert(day.clone(), schedule.get(day).map_or(0, |v| v.len()));
            }
        }
        counts
    });

    let generate_button_text = use_memo(move || {
        if *is_generating.read() {
            if current_schedule.read().is_some() {
                "Regenerating..."
            } else {
                "Generating..."
            }
        } else if current_schedule.read().is_some() {
            "Regenerate"
        } else {
            "Generate"
        }
    });

    let month_name = use_memo(move || {
        Month::try_from(selected_month() as u8)
            .map(|m| m.name().to_string())
            .unwrap_or_else(|_| "Invalid Month".to_string())
    });

    // --- Event Handlers ---
    // (Handlers remain the same as previous correct version)
    let handle_search = move |query: String| search_query.set(query);

    let mut change_month = move |increment: i32| {
        let mut current_month_num = selected_month() as i32 + increment;
        let mut new_year = selected_year();
        if current_month_num > 12 {
            current_month_num = 1;
            new_year += 1;
        } else if current_month_num < 1 {
            current_month_num = 12;
            new_year -= 1;
        }

        if (2025..=2035).contains(&new_year) {
            if let Some(new_month) = Month::try_from(current_month_num as u8).ok() {
                selected_month.set(new_month.number_from_month());
                selected_year.set(new_year);
            } else {
                error_message.set(Some("Internal error calculating month.".to_string()));
            }
        } else {
            error_message.set(Some("Year out of allowed range (2020-2030)".to_string()));
        }
    };

    let mut change_year = move |increment: i32| {
        let new_year = (selected_year() + increment).clamp(2025, 2035);
        if new_year != selected_year() {
            selected_year.set(new_year);
        }
    };

    let handle_generate = move |_| {
        if *is_generating.read() {
            return;
        }
        is_generating.set(true);
        error_message.set(None);
        let year = selected_year();
        let month = selected_month();
        let current_employees = employees.read().clone();

        spawn(async move {
            let today = Local::now().date_naive();
            if let Some(first_day) = NaiveDate::from_ymd_opt(year, month, 1) {
                if first_day.year() < today.year()
                    || (first_day.year() == today.year() && first_day.month() < today.month())
                {
                    error_message.set(Some(format!(
                        "Cannot generate schedules for past months ({}-{}).",
                        month, year
                    )));
                    is_generating.set(false);
                    return;
                }
            } else {
                error_message.set(Some(format!("Invalid date selected ({}-{}).", month, year)));
                is_generating.set(false);
                return;
            }

            // let past_schedules = HashMap::new(); // Placeholder
            // Get past schedules
            let past_schedules = get_past_schedules(year, month, &current_employees).await;
            info!("Generating schedule for {}-{}", month, year);
            let schedule = generate_balanced_schedule(&current_employees, &past_schedules);
            current_schedule.set(Some(schedule));
            is_generating.set(false);
        });
    };

    // implementation for get_past_schedules for each employee
    async fn get_past_schedules(
        year: i32,
        month: u32,
        employees: &[Employee],
    ) -> HashMap<usize, Vec<HashSet<Weekday>>> {
        let mut past_schedules: HashMap<usize, Vec<HashSet<Weekday>>> = HashMap::new();
        if let Ok(conn) = establish_connection() {
            for employee in employees {
                past_schedules.insert(employee.id, Vec::new());
                // Load schedules from the last 3 months
                for i in 1..=3 {
                    let past_month = month as i32 - i;
                    let past_year = year - (if past_month < 1 { 1 } else { 0 });
                    let adjusted_month = if past_month < 1 {
                        (past_month + 12) as u32
                    } else {
                        past_month as u32
                    };

                    if let Ok(Some(schedule)) =
                        load_schedule_from_db(&conn, past_year, adjusted_month)
                    {
                        let mut employee_days: HashSet<Weekday> = HashSet::new();
                        for (day, assigned_employees) in schedule.iter() {
                            if assigned_employees.iter().any(|e| e.id == employee.id) {
                                employee_days.insert(day.clone());
                            }
                        }
                        past_schedules
                            .get_mut(&employee.id)
                            .unwrap()
                            .push(employee_days);
                    }
                }
            }
        }
        past_schedules
    }

    let handle_save = move |_| {
        if let Some(schedule_data) = current_schedule.read().clone() {
            let year = selected_year();
            let month = selected_month();
            error_message.set(None);
            spawn(async move {
                match establish_connection() {
                    Ok(conn) => match save_schedule_to_db(&conn, year, month, &schedule_data) {
                        Ok(_) => {
                            error_message.set(Some("Schedule saved successfully!".to_string()))
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Failed to save schedule: {}", e)))
                        }
                    },
                    Err(e) => error_message.set(Some(format!("Database connection error: {}", e))),
                }
            });
        } else {
            error_message.set(Some("No schedule generated or loaded to save.".to_string()));
        }
    };

    // let mut handle_employee_click = move |emp_id: usize| {
    //     selected_employee.set(Some(emp_id));
    //     modal_view.set(ModalView::EmployeeDetails(emp_id));
    // };

    let mut handle_employee_click = move |emp_id: usize| {
        // Fetch the past schedules *before* opening the modal.
        spawn(async move {
            let current_year = selected_year();
            let current_month = selected_month();
            let current_employees = employees.read().clone(); // Clone necessary data

            // Find the specific employee to pass - more efficient
            let target_employee = current_employees.iter().find(|e| e.id == emp_id).cloned();

            if let Some(emp) = target_employee {
                info!("Fetching past schedules for employee ID: {}", emp_id); // LOG 1
                                                                              // Pass only the relevant employee to get_past_schedules if possible,
                                                                              // or filter inside get_past_schedules. For now, passing all.
                let employee_past_schedules_map =
                    get_past_schedules(current_year, current_month, &current_employees).await;
                info!(
                    "Fetched past schedules data: {:?}",
                    employee_past_schedules_map.get(&emp_id)
                ); // LOG 2

                past_schedules_modal.set(employee_past_schedules_map); // store schedules to signal
                info!(
                    "Set past_schedules_modal signal for employee ID: {}",
                    emp_id
                ); // LOG 3

                selected_employee.set(Some(emp_id)); // Set selected employee *after* data might be ready
                modal_view.set(ModalView::EmployeeDetails(emp_id)); // Open modal last
            } else {
                error!(
                    "Employee with ID {} not found for past schedule fetch.",
                    emp_id
                );
                // Handle error - maybe show an error message?
                error_message.set(Some(format!("Employee {} not found.", emp_id)));
            }
        });
    };

    let mut handle_edit_schedule_click = move |day: Weekday, emp_id: usize| {
        let initial_days = {
            let mut days = HashSet::new();
            if let Some(sched) = &*current_schedule.read() {
                for (day_key, emps_on_day) in sched {
                    if emps_on_day.iter().any(|e| e.id == emp_id) {
                        days.insert(day_key.clone());
                    }
                }
            }
            days
        };
        edit_days.set(initial_days);
        modal_view.set(ModalView::EditSchedule(day.clone(), emp_id));
    };

    let mut handle_update_schedule = move |emp_id: usize, new_days_set: HashSet<Weekday>| {
        current_schedule.with_mut(|maybe_schedule| {
            if let Some(schedule) = maybe_schedule {
                if let Some(emp) = employees.read().iter().find(|e| e.id == emp_id).cloned() {
                    for day_employees in schedule.values_mut() {
                        day_employees.retain(|e| e.id != emp_id);
                    }
                    for new_day in new_days_set {
                        schedule.entry(new_day).or_default().push(emp.clone());
                    }
                    for day_employees in schedule.values_mut() {
                        day_employees.sort_by_key(|e| e.name.clone());
                    }
                    error_message.set(Some(format!("Schedule updated for {}", emp.name)));
                } else {
                    error_message.set(Some(
                        "Failed to update schedule: Employee not found.".to_string(),
                    ));
                }
            } else {
                error_message.set(Some("Cannot update: No schedule loaded.".to_string()));
            }
        });
        modal_view.set(ModalView::None);
    };

    let mut select_month_from_modal = move |month_num: u32| {
        if let Some(new_month) = Month::try_from(month_num as u8).ok() {
            if new_month.number_from_month() != selected_month() {
                selected_month.set(new_month.number_from_month());
            }
            modal_view.set(ModalView::None);
        }
    };

    let mut select_year_from_modal = move |year_val: i32| {
        let clamped_year = year_val.clamp(2025, 2035);
        if clamped_year != selected_year() {
            selected_year.set(clamped_year);
        }
        modal_view.set(ModalView::None);
        if year_val != clamped_year {
            error_message.set(Some("Year must be between 2025 and 2035.".to_string()));
        } else {
            error_message.set(None);
        }
    };

    // --- Schedule Table Calculation ---
    let schedule_display_element = {
        let schedule_read = current_schedule.read();
        match schedule_read.as_ref() {
            Some(schedule) if !schedule.is_empty() => {
                let max_rows = schedule.values().map(|emps| emps.len()).max().unwrap_or(0);
                let schedule_clone = schedule.clone();

                rsx! { // Start of the *outer* rsx! for the table element
                    div { class: "schedule-table-container",
                        table { class: "schedule-table",
                            thead { tr { for day in Weekday::values() { th { "{day}" span { class: "day-count", " ({day_counts().get(day).unwrap_or(&0)})" } } } } }
                            tbody {
                                if max_rows == 0 { tr { td { colspan: Weekday::values().len() as u32, class: "empty-schedule-message", "Schedule is empty." } } }
                                else {
                                    for row_index in 0..max_rows {
                                        tr {
                                            for day_ref in Weekday::values() {
                                                td {
                                                    if let Some(emp) = schedule_clone.get(day_ref).and_then(|emps| emps.get(row_index)) {
                                                        // CORRECT FIX: Use a standard Rust block { } here to contain the 'let' bindings
                                                        {
                                                            // These 'let' bindings are now *outside* any rsx! macro invocation
                                                            let emp_clone = emp.clone();
                                                            let day_clone = day_ref.clone();

                                                            // Now, call rsx! *inside* this standard block to render the element
                                                            rsx! {
                                                                div {
                                                                    key: "{day_ref}-{emp_clone.id}-{row_index}",
                                                                    class: "schedule-employee-card",
                                                                    onclick: move |_| handle_employee_click(emp_clone.id),
                                                                    div { class: "card-name", "{emp_clone.name}" }
                                                                    div { class: "card-role", "{emp_clone.role}" } // Assuming role implements Display
                                                                    button {
                                                                        class: "edit-schedule-entry", title: "Edit schedule",
                                                                        onclick: move |evt| {
                                                                            evt.stop_propagation();
                                                                            handle_edit_schedule_click(day_clone.clone(), emp_clone.id);
                                                                        },
                                                                        img {
                                                                            src: EDIT_ICON,
                                                                            width: "25",
                                                                            height: "25",
                                                                        }
                                                                    }
                                                                }
                                                            } // End of the *inner* rsx! for the card
                                                        } // End of the standard Rust block containing the let bindings
                                                    } else {
                                                        // Render empty placeholder cell content
                                                        div { class: "schedule-employee-card empty-card" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } // End of the *outer* rsx! for the table element
            }
            _ => {
                // Handles None, Some(empty), loading, and error states implicitly
                rsx! {
                    div { class: "no-schedule-message",
                        if employees().is_empty() { p { "No employees found. Add employees first." } }
                        else if *is_generating.read() { p { "Generating schedule..." } }
                        else if schedule_read.is_none() && error_message.read().is_none() { p { "No schedule added yet..." br {}, br{}, "Generate a new schedule to view it here." } } //"Loading schedule..."
                        else { p { "No schedule available for {month_name()} {selected_year()}. Click 'Generate' or select a different date." } }
                    }
                }
            }
        }
    };

    // --- Main Render (rsx!) ---
    // (Main rsx! remains the same as previous correct version, including modals)
    rsx! {
        document::Link { rel: "stylesheet", href: SCHEDULES_CSS }

        div { class: "schedules-container",
            h1 { "Schedules" }

            // --- Action Bar ---
            div { class: "schedule-actions",
                // SearchBar { placeholder: "Search employees...".to_string(), on_search: handle_search }
                div { class: "date-selectors",
                    div { class: "date-selector month-selector",
                        button { class: "arrow-button", onclick: move |_| change_month(-1), img {
                            src: ARROW_LEFT_ICON,
                            width: "20",
                            height: "20",
                        } }
                        button { class: "selector-button", onclick: move |_| modal_view.set(ModalView::Month), "{month_name()}" }
                        button { class: "arrow-button", onclick: move |_| change_month(1), img {
                            src: ARROW_RIGHT_ICON,
                            width: "20",
                            height: "20",
                        } }
                    }
                    div { class: "date-selector year-selector",
                        button { class: "arrow-button", onclick: move |_| change_year(-1), img {
                            src: ARROW_LEFT_ICON,
                            width: "20",
                            height: "20",
                        } }
                        button { class: "selector-button", onclick: move |_| modal_view.set(ModalView::Year), "{selected_year()}" }
                        button { class: "arrow-button", onclick: move |_| change_year(1), img {
                            src: ARROW_RIGHT_ICON,
                            width: "20",
                            height: "20",
                        } }
                    }
                }
                div { class: "action-buttons",
                    button { class: "btn btn-primary", onclick: handle_generate, disabled: *is_generating.read() || employees().is_empty(), title: if employees().is_empty() { "Add employees first" } else { "" }, "{generate_button_text()}" }
                    button { class: "btn btn-secondary", onclick: handle_save, disabled: current_schedule.read().is_none(), "Save" }
                    if let Some(schedule_data) = current_schedule.read().clone() {
                        if !schedule_data.is_empty() { ShareButton { schedule: schedule_data, year: selected_year(), month: selected_month() } }
                    }
                }
            }

            // --- Error Message Area ---
            if let Some(msg) = &*error_message.read() { div { class: "error-message", "{msg}" } }

            // --- Schedule Display Area ---
            {schedule_display_element} // Render the pre-computed element

            // --- Modals ---
            if *modal_view.read() != ModalView::None {
                div { class: "modal-overlay", onclick: move |_| modal_view.set(ModalView::None),
                    div { class: "modal", onclick: move |evt| evt.stop_propagation(),
                        // Modal Content
                        match modal_view.read().clone() {
                            ModalView::Month => rsx! {
                                div { class: "month-selector-modal",
                                    h3 { "Select Month" }
                                    div { class: "month-grid",
                                        for month_num in 1..=12 {
                                            { // Scope for is_selected calculation
                                                let is_selected = month_num == selected_month();
                                                rsx! {
                                                    button {
                                                        class: if is_selected {"month-button selected"} else {"month-button"},
                                                        onclick: move |_| select_month_from_modal(month_num),
                                                        { Month::try_from(month_num as u8).map(|m| m.name().to_string()).unwrap_or("?".to_string()) }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            ModalView::Year => rsx! {
                                div { class: "year-selector-modal",
                                    h3 { "Select Year" }
                                    div { class: "year-input-container",
                                        input {
                                            r#type: "number", class: "year-input", min: "2025", max: "2035", value: "{selected_year()}",
                                            oninput: move |evt: Event<FormData>| {
                                                if let Ok(year_val) = evt.value().parse::<i32>() { select_year_from_modal(year_val); }
                                                else if !evt.value().is_empty() { error_message.set(Some("Invalid year entered.".to_string())); }
                                                else { error_message.set(None); }
                                            },
                                        }
                                    }
                                }
                            },
                            ModalView::EmployeeDetails(emp_id) => rsx! {
                                if let Some(emp) = employees.read().iter().find(|e| e.id == emp_id).cloned() {
                                    div { class: "employee-details-modal",
                                        h3 { "Employee Details" }, div { class: "employee-info", p { strong { "Name: " } "{emp.name}" }, p { strong { "Role: " } "{emp.role}" } /* Add more emp details */ },
                                        div { class: "past-schedules", h4 { "Past Schedules" },
                                                //NEW: Display data
                                                if let Some(employee_past_schedules) = past_schedules_modal.read().get(&emp_id) {
                                                if employee_past_schedules.is_empty() {
                                                    p { class: "past-schedule-message", "No past schedule data available." }
                                                } else {
                                                    ul { class: "past-schedule-list",
                                                        for (index, schedule) in employee_past_schedules.iter().enumerate() {
                                                            li { key: "{index}", class: "past-schedule-item",
                                                                span { class: "past-schedule-month-label", "Month {index + 1}: "}
                                                                { // Add explicit block for the conditional rendering
                                                                    if schedule.is_empty() {
                                                                        // Return a simple text node wrapped in rsx!
                                                                        rsx! { span { class: "past-schedule-days empty", "No days scheduled." } }
                                                                    } else {
                                                                        // Calculate the string
                                                                        let days_str = schedule.iter()
                                                                            .map(|d| d.to_string())
                                                                            .collect::<Vec<String>>()
                                                                            .join(", ");
                                                                        // Render the calculated string wrapped in rsx!
                                                                        rsx! { span { class: "past-schedule-days", "{days_str}" } }
                                                                    }
                                                                } // End of explicit block
                                                            }
                                                        }
                                                    }
                                                }
                                                } else {
                                                    p { class: "past-schedule-message", "Loading past schedules..."}
                                                }
                                        }
                                    }
                                } else { div { class: "employee-details-modal", h3 { "Error" }, p { "Employee details not found."} } }
                            },
                            ModalView::EditSchedule(_, emp_id) => rsx! {
                                if let Some(emp) = employees.read().iter().find(|e| e.id == emp_id).cloned() {
                                    div { class: "edit-schedule-modal",
                                        h3 { "Edit Schedule for {emp.name}" }, p { "Select work days for {month_name()} {selected_year()}:" },
                                        div { class: "day-selection",
                                            for weekday_ref in Weekday::values() { { // Scope for checkbox logic
                                                let current_edit_days = edit_days.read(); let is_checked = current_edit_days.contains(weekday_ref);
                                                let weekday_clone = weekday_ref.clone();
                                                rsx!( label { class: "day-checkbox", input { r#type: "checkbox", checked: is_checked, oninput: move |evt: Event<FormData>| { let checked: bool = evt.value().parse().unwrap_or(false); edit_days.with_mut(|days| { if checked { days.insert(weekday_clone.clone()); } else { days.remove(&weekday_clone); } }); } }, span { class: if is_checked { "day-selected" } else { "" }, "{weekday_ref}" } } )
                                            } }
                                        },
                                        div { class: "modal-actions",
                                            button { class: "btn btn-cancel", onclick: move |_| modal_view.set(ModalView::None), "Cancel" },
                                            button { class: "btn btn-primary", onclick: move |_| handle_update_schedule(emp_id, edit_days.read().clone()), "Save Changes" }
                                        }
                                    }
                                } else { div { class: "edit-schedule-modal", h3 { "Error" }, p { "Employee details not found." } } }
                            },
                            ModalView::None => rsx! { div {} }
                        }
                        // Common Modal Close Button
                        button { class: "modal-close", title: "Close", onclick: move |_| modal_view.set(ModalView::None), img {
                            src: X_CLOSE_ICON,
                            width: "40",
                            height: "40",
                        } }
                    }
                }
            }
        }
    }
}

// Helper Trait/Impl for Weekday iteration
mod weekday_helper {
    use super::Weekday;
    impl Weekday {
        pub fn values() -> &'static [Weekday] {
            &[
                Weekday::Monday,
                Weekday::Tuesday,
                Weekday::Wednesday,
                Weekday::Thursday,
                Weekday::Friday,
            ]
        }
    }
}
