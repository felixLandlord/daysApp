use crate::server::{db, schema::Employee};
use anyhow::Result;
use dioxus::logger::tracing::{error, info};
use rusqlite::Connection;
use serde::Deserialize;
use std::error::Error;

// Define a struct that matches the expected JSON format
#[derive(Debug, Deserialize)]
pub struct EmployeeImport {
    pub name: String,
    pub sex: String,
    pub role: String,
    pub required_days: u8,
    pub fixed_days: Vec<String>,
    pub is_nsp: bool,
}

// Convert the imported data to the Employee struct
fn convert_to_employee(import: EmployeeImport) -> Result<Employee, Box<dyn Error>> {
    // Parse sex
    let sex = match import.sex.to_lowercase().as_str() {
        "male" => crate::server::schema::Sex::Male,
        "female" => crate::server::schema::Sex::Female,
        _ => return Err(format!("Invalid sex value: {}", import.sex).into()),
    };

    // Parse role
    let role = match import.role.as_str() {
        "Human Resource Manager" => crate::server::schema::Role::HR,
        "AI-LLM Engineer" => crate::server::schema::Role::AiLlmEngineer,
        "Social Media Marketing" => crate::server::schema::Role::SocialMediaMarketing,
        // "Marketing Manager" => crate::server::schema::Role::MarketingManager,
        "IT Support" => crate::server::schema::Role::ITSupport,
        "Machine Learning Engineer" => crate::server::schema::Role::MLEngineer,
        "Data Scientist" => crate::server::schema::Role::DataScientist,
        "Data Analyst" => crate::server::schema::Role::DataAnalyst,
        "Full-stack Engineer" => crate::server::schema::Role::FullStackEngineer,
        "Backend Engineer" => crate::server::schema::Role::BackendEngineer,
        "Frontend Engineer" => crate::server::schema::Role::FrontendEngineer,
        "Blockchain Engineer" => crate::server::schema::Role::BlockchainEngineer,
        "QA Engineer" => crate::server::schema::Role::QaEngineer,
        "Project Manager" => crate::server::schema::Role::ProjectManager,
        "UI/UX Designer" => crate::server::schema::Role::UiUxDesigner,
        "Mobile Engineer" => crate::server::schema::Role::MobileEngineer,
        "DevOps Engineer" => crate::server::schema::Role::DevOpsEngineer,
        "Operations Manager" => crate::server::schema::Role::OperationsManager,
        _ => return Err(format!("Invalid role value: {}", import.role).into()),
    };

    // Parse fixed days
    let mut fixed_days = Vec::new();
    for day in import.fixed_days {
        let weekday = match day.to_lowercase().as_str() {
            "monday" => crate::server::schema::Weekday::Monday,
            "tuesday" => crate::server::schema::Weekday::Tuesday,
            "wednesday" => crate::server::schema::Weekday::Wednesday,
            "thursday" => crate::server::schema::Weekday::Thursday,
            "friday" => crate::server::schema::Weekday::Friday,
            _ => return Err(format!("Invalid weekday value: {}", day).into()),
        };
        fixed_days.push(weekday);
    }

    Ok(Employee {
        id: 0, // added 0
        name: import.name,
        sex,
        role,
        required_days: import.required_days,
        fixed_days,
        is_nsp: import.is_nsp,
    })
}

// Import employees from JSON string
pub fn import_employees_from_json(json_data: &str) -> Result<Vec<Employee>> {
    let imports: Vec<EmployeeImport> = serde_json::from_str(json_data)?;

    let mut employees = Vec::new();
    for (i, import) in imports.into_iter().enumerate() {
        match convert_to_employee(import) {
            Ok(employee) => employees.push(employee),
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error converting employee at index {}: {}",
                    i,
                    e
                ))
            }
        }
    }

    Ok(employees)
}

// Save imported employees to database
pub fn save_imported_employees(conn: &Connection, employees: Vec<Employee>) -> Result<usize> {
    let mut count = 0;

    for employee in employees {
        match db::insert_employee_with_auto_id(conn, &employee) {
            Ok(inserted_employee) => {
                count += 1;
                // info!("Imported employee: {}", employee.name);
                info!(
                    "Imported employee: {} with ID: {}",
                    inserted_employee.name, inserted_employee.id
                );
            }
            Err(e) => {
                error!("Failed to import employee {}: {}", employee.name, e);
                return Err(anyhow::anyhow!("Database error: {}", e));
            }
        }
    }

    Ok(count)
}
