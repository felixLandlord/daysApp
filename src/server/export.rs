use crate::{
    client::pages::settings_page,
    server::schema::{Employee, MonthlySchedule, Weekday},
};
use chrono::Month;
use dioxus::{
    logger::tracing::{error, info},
    // prelude::*,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use rust_xlsxwriter::*;

// Helper to get month name safely
fn get_month_name(month: u32) -> String {
    Month::try_from(month as u8)
        .map(|m| m.name().to_string())
        .unwrap_or_else(|_| format!("Month_{}", month)) // Fallback for invalid month
}

pub fn generate_csv_data(
    schedule: &MonthlySchedule,
    year: i32,
    month: u32,
) -> Result<(String, String), Box<dyn Error>> {
    let month_name = get_month_name(month);
    let filename = format!("office_schedule_{}_{}.csv", month_name, year);

    let weekdays = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
    ];

    // --- Header Row 1 ---
    let header1_parts: Vec<String> = std::iter::once("Name".to_string())
        .chain(weekdays.iter().map(|d| d.to_string()))
        .collect();
    let header1 = header1_parts.join(",");

    // --- Header Row 2 (Counts) ---
    let counts: Vec<String> = weekdays
        .iter()
        .map(|day| {
            let count = schedule.get(day).map_or(0, |emps| emps.len());
            format!("{}", count)
        })
        .collect();
    // Add an empty cell at the beginning to align with the 'Name' column
    let header2 = format!(",{}", counts.join(",")); // Prepend comma for empty first cell

    // --- Data Rows ---

    // 1. Collect all unique employees and sort them by name
    let mut all_employee_refs: Vec<&Employee> = schedule
        .values()
        .flatten()
        .collect::<HashSet<_>>() // Deduplicate Employee references
        .into_iter()
        .collect();
    all_employee_refs.sort_by(|a, b| a.name.cmp(&b.name)); // Sort by name

    // 2. Create assignment lookup: HashMap<Weekday, HashSet<EmployeeName>>
    let assignments: HashMap<Weekday, HashSet<String>> = schedule
        .iter()
        .map(|(day, emps)| {
            let names = emps.iter().map(|e| e.name.clone()).collect::<HashSet<_>>();
            (day.clone(), names) // Clone Weekday for the key
        })
        .collect();

    // 3. Generate data rows
    let data_rows: Vec<String> = all_employee_refs
        .iter()
        .map(|emp| {
            let mut row_parts = vec![emp.name.clone()]; // Start row with employee name
            for day in &weekdays {
                // Check if this employee is assigned on this day
                let is_assigned = assignments
                    .get(day) // Get the set of names for the day
                    .map_or(false, |names_on_day| names_on_day.contains(&emp.name)); // Check if emp's name is in the set

                row_parts.push(if is_assigned {
                    "X".to_string()
                } else {
                    "".to_string() // Empty string if not assigned
                });
            }
            row_parts.join(",") // Join parts into a CSV row string
        })
        .collect();

    // --- Combine all parts ---
    let csv_data = format!("{}\n{}\n{}", header1, header2, data_rows.join("\n"));

    Ok((filename, csv_data))
}

// CSV
pub async fn save_csv_with_dialog(
    suggested_filename: String,
    csv_data: String,
) -> Result<(), Box<dyn Error>> {
    info!("Requesting file save dialog...");

    let file_handle = rfd::AsyncFileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name(&suggested_filename)
        .set_title("Save Schedule as CSV")
        .save_file()
        .await;

    match file_handle {
        Some(handle) => {
            let path = handle.path(); // Get the path chosen by the user
            info!("Saving CSV to: {:?}", path);
            // rfd's FileHandle provides an async write method that works on both native and wasm
            match handle.write(csv_data.as_bytes()).await {
                Ok(_) => {
                    info!("CSV file saved successfully.");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to write CSV file: {}", e);
                    Err(Box::new(e)) // Convert rfd::Error into Box<dyn Error>
                }
            }
        }
        None => {
            info!("CSV save cancelled by user.");
            Ok(()) // User cancellation is not an error
        }
    }
}

// XLSX
pub async fn save_xlsx_with_dialog(
    suggested_filename: String,
    xlsx_data: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    info!("Requesting XLSX file save dialog...");
    let file_handle = rfd::AsyncFileDialog::new()
        .add_filter("Excel", &["xlsx"])
        .set_file_name(&suggested_filename)
        .set_title("Save Schedule as Excel")
        .save_file()
        .await;

    match file_handle {
        Some(handle) => {
            let path = handle.path();
            info!("Saving XLSX to: {:?}", path);
            match handle.write(&xlsx_data).await {
                Ok(_) => {
                    info!("XLSX file saved successfully.");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to write XLSX file: {}", e);
                    Err(Box::new(e))
                }
            }
        }
        None => {
            info!("XLSX save cancelled by user.");
            Ok(())
        }
    }
}

// creating a formatted XLSX file that looks good
pub fn generate_xlsx_data(
    schedule: &MonthlySchedule,
    year: i32,
    month: u32,
) -> Result<(String, Vec<u8>), Box<dyn Error>> {
    let month_name = get_month_name(month);
    let filename = format!("office_schedule_{}_{}.xlsx", month_name, year);

    let weekdays = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
    ];

    // Create a new workbook
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name("Schedule")?;

    // Define formats
    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4F81BD))
        .set_font_color(Color::White)
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_font_size(14);

    let count_format = Format::new()
        .set_italic()
        .set_bold()
        .set_align(FormatAlign::Center)
        // .set_background_color(Color::RGB(0xF2F2F2))
        .set_background_color(Color::RGB(0x4F81BD))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin)
        .set_font_size(13);

    let name_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin)
        .set_font_size(11);

    let data_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_border(FormatBorder::Thin);

    let x_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_bold()
        // .set_font_color(Color::RGB(0x00B050))
        .set_font_color(Color::RGB(0x7A52A3))
        .set_border(FormatBorder::Thin)
        .set_font_size(12);

    // Set column widths
    worksheet.set_column_width(0, 17.0)?; // Name column
    for i in 1..=5 {
        worksheet.set_column_width(i as u16, 12.0)?; // Weekday columns
    }

    // --- Header Row 1 ---
    worksheet.write_string_with_format(0, 0, "Name", &header_format)?;
    for (i, day) in weekdays.iter().enumerate() {
        worksheet.write_string_with_format(0, (i + 1) as u16, &day.to_string(), &header_format)?;
    }

    // --- Header Row 2 (Counts) ---
    worksheet.write_string_with_format(1, 0, "", &count_format)?; // Empty cell for name column
    for (i, day) in weekdays.iter().enumerate() {
        let count = schedule.get(day).map_or(0, |emps| emps.len());
        worksheet.write_string_with_format(
            1,
            (i + 1) as u16,
            &format!("{}", count),
            &count_format,
        )?;
    }

    // --- Data Rows ---
    // 1. Collect all unique employees and sort them by name
    let mut all_employee_refs: Vec<&Employee> = schedule
        .values()
        .flatten()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_employee_refs.sort_by(|a, b| a.name.cmp(&b.name));

    // 2. Create assignment lookup
    let assignments: HashMap<Weekday, HashSet<String>> = schedule
        .iter()
        .map(|(day, emps)| {
            let names = emps.iter().map(|e| e.name.clone()).collect::<HashSet<_>>();
            (day.clone(), names)
        })
        .collect();

    // 3. Generate data rows
    for (row_idx, emp) in all_employee_refs.iter().enumerate() {
        let excel_row = (row_idx + 2) as u32; // Start from row 2 (0-indexed, after headers)

        // Employee name (bold)
        worksheet.write_string_with_format(excel_row, 0, &emp.name, &name_format)?;

        // Assignment status for each day
        for (col_idx, day) in weekdays.iter().enumerate() {
            let excel_col = (col_idx + 1) as u16;
            let is_assigned = assignments
                .get(day)
                .map_or(false, |names_on_day| names_on_day.contains(&emp.name));

            if is_assigned {
                worksheet.write_string_with_format(excel_row, excel_col, "X", &x_format)?;
            } else {
                worksheet.write_string_with_format(excel_row, excel_col, "", &data_format)?;
            }
        }
    }

    // Convert workbook to bytes
    let xlsx_data = workbook.save_to_buffer()?;

    Ok((filename, xlsx_data))
}

// Alternative function that returns both CSV and XLSX
// pub fn generate_schedule_files(
//     schedule: &MonthlySchedule,
//     year: i32,
//     month: u32,
// ) -> Result<(String, String, String, Vec<u8>), Box<dyn Error>> {
//     // Generate CSV
//     let (csv_filename, csv_data) = generate_csv_data(schedule, year, month)?;

//     // Generate XLSX
//     let (xlsx_filename, xlsx_data) = generate_xlsx_data(schedule, year, month)?;

//     Ok((csv_filename, csv_data, xlsx_filename, xlsx_data))
// }
