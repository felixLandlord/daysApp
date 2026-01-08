use crate::{IntegrationError, Result};
use common::{Employee, Schedule, Weekday};
// use serde_json::json;

pub struct GoogleSheetsClient {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl GoogleSheetsClient {
    pub fn new() -> Self {
        Self {
            api_key: None,
            client: reqwest::Client::new(),
        }
    }

    pub fn configure(&mut self, api_key: String) {
        self.api_key = Some(api_key);
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    // Placeholder for importing employees from Google Sheets
    pub async fn import_employees(&self, sheet_url: &str) -> Result<Vec<Employee>> {
        if !self.is_configured() {
            return Err(IntegrationError::NotConfigured);
        }

        // TODO: Implement actual Google Sheets API integration
        // This would involve:
        // 1. Parsing the sheet URL to extract sheet ID
        // 2. Making authenticated requests to Google Sheets API
        // 3. Parsing the response and converting to Employee structs

        // For now, return empty vector as foundation
        Ok(Vec::new())
    }

    // Placeholder for importing schedule from Google Sheets
    pub async fn import_schedule(&self, sheet_url: &str) -> Result<Schedule> {
        if !self.is_configured() {
            return Err(IntegrationError::NotConfigured);
        }

        // TODO: Implement actual Google Sheets API integration

        // For now, return empty schedule
        Ok(Schedule::new(2025, 1))
    }

    // Placeholder for exporting schedule to Google Sheets
    pub async fn export_schedule(
        &self,
        schedule: &Schedule,
        employees: &[Employee],
        share_mode: common::config::ShareMode,
    ) -> Result<String> {
        if !self.is_configured() {
            return Err(IntegrationError::NotConfigured);
        }

        // TODO: Implement actual Google Sheets API integration
        // This would involve:
        // 1. Creating a new Google Sheet
        // 2. Formatting the schedule data
        // 3. Writing to the sheet
        // 4. Setting appropriate sharing permissions based on share_mode
        // 5. Returning the shareable link

        // For now, return placeholder URL
        Ok("https://docs.google.com/spreadsheets/d/placeholder".to_string())
    }

    // Helper to convert schedule to spreadsheet format
    fn schedule_to_sheet_data(
        &self,
        schedule: &Schedule,
        employees: &[Employee],
    ) -> Vec<Vec<String>> {
        let mut data = Vec::new();

        // Header row
        let mut header = vec!["Name".to_string()];
        for day in Weekday::all() {
            header.push(day.to_string());
        }
        data.push(header);

        // Count row
        let mut counts = vec!["".to_string()];
        for day in Weekday::all() {
            counts.push(schedule.count_for_day(day).to_string());
        }
        data.push(counts);

        // Employee rows
        for emp in employees {
            let mut row = vec![emp.name.clone()];
            for day in Weekday::all() {
                let assigned = schedule.get_employees_for_day(day).contains(&emp.id);
                row.push(if assigned {
                    "X".to_string()
                } else {
                    "".to_string()
                });
            }
            data.push(row);
        }

        data
    }
}

impl Default for GoogleSheetsClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_client_configuration() {
        let mut client = GoogleSheetsClient::new();
        assert!(!client.is_configured());

        client.configure("test_api_key".to_string());
        assert!(client.is_configured());
    }
}
