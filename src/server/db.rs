use crate::server::schema::{Employee, MonthlySchedule, Role, Sex, Weekday};
use anyhow::Result;
use rusqlite::{params, Connection, Result as SqliteResult};

// pub fn establish_connection() -> Result<Connection> {
//     let conn = Connection::open("employees.db")?;
//     // Enable foreign keys (important for referential integrity)
//     conn.execute("PRAGMA foreign_keys = ON;", [])?;
//     Ok(conn)
// }
pub fn establish_connection() -> Result<Connection> {
    let app_dir = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join("days-app");

    // Create the app directory if it doesn't exist
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| anyhow::anyhow!("Failed to create app directory: {}", e))?;

    let db_path = app_dir.join("employees.db");

    let conn = Connection::open(db_path)?;

    // Enable foreign keys
    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    Ok(conn)
}

pub fn create_employee_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY AUTOINCREMENT, -- Added AUTOINCREMENT
            name TEXT NOT NULL,
            sex TEXT NOT NULL,
            role TEXT NOT NULL,
            required_days INTEGER NOT NULL,
            fixed_days TEXT,  -- Store as JSON
            is_nsp INTEGER NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn insert_employee(conn: &Connection, employee: &Employee) -> SqliteResult<()> {
    let fixed_days_json = serde_json::to_string(&employee.fixed_days).unwrap();
    conn.execute(
        "INSERT INTO employees (id, name, sex, role, required_days, fixed_days, is_nsp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            employee.id,
            employee.name,
            employee.sex.to_string(),
            employee.role.to_string(),
            employee.required_days,
            fixed_days_json,
            employee.is_nsp as i32
        ],
    )?;
    Ok(())
}

// Alternative: Insert employee and return the new employee with assigned ID
pub fn insert_employee_with_auto_id(
    conn: &Connection,
    employee: &Employee,
) -> SqliteResult<Employee> {
    let fixed_days_json = serde_json::to_string(&employee.fixed_days).unwrap();
    conn.execute(
        "INSERT INTO employees (name, sex, role, required_days, fixed_days, is_nsp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            employee.name,
            employee.sex.to_string(),
            employee.role.to_string(),
            employee.required_days,
            fixed_days_json,
            employee.is_nsp as i32
        ],
    )?;

    let new_id = conn.last_insert_rowid() as usize;

    Ok(Employee {
        id: new_id,
        name: employee.name.clone(),
        sex: employee.sex.clone(),
        role: employee.role.clone(),
        required_days: employee.required_days,
        fixed_days: employee.fixed_days.clone(),
        is_nsp: employee.is_nsp,
    })
}

pub fn update_employee(conn: &Connection, employee: &Employee) -> SqliteResult<()> {
    let fixed_days_json = serde_json::to_string(&employee.fixed_days).unwrap();
    conn.execute(
        "UPDATE employees SET name = ?2, sex = ?3, role = ?4, required_days = ?5, fixed_days = ?6, is_nsp = ?7 WHERE id = ?1",
        params![
            employee.id,
            employee.name,
            employee.sex.to_string(),
            employee.role.to_string(),
            employee.required_days,
            fixed_days_json,
            employee.is_nsp as i32
        ],
    )?;
    Ok(())
}

pub fn delete_employee(conn: &Connection, id: usize) -> SqliteResult<()> {
    conn.execute("DELETE FROM employees WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_all_employees(conn: &Connection) -> SqliteResult<Vec<Employee>> {
    let mut stmt = conn
        .prepare("SELECT id, name, sex, role, required_days, fixed_days, is_nsp FROM employees")?;
    let employee_iter = stmt.query_map([], |row| {
        let id: usize = row.get(0)?;
        let name: String = row.get(1)?;
        let sex_str: String = row.get(2)?;
        let role_str: String = row.get(3)?;
        let required_days: u8 = row.get(4)?;
        let fixed_days_json: String = row.get(5)?;
        let is_nsp: i32 = row.get(6)?;

        let sex = match sex_str.as_str() {
            "Male" => Sex::Male,
            "Female" => Sex::Female,
            _ => Sex::Male, // Or handle the error/unknown case appropriately
        };
        let role = match role_str.as_str() {
            "Human Resource Manager" => Role::HR,
            "AI-LLM Engineer" => Role::AiLlmEngineer,
            "Social Media Marketing" => Role::SocialMediaMarketing,
            // "Marketing Manager" => Role::MarketingManager,
            "IT Support" => Role::ITSupport,
            "Machine Learning Engineer" => Role::MLEngineer,
            "Data Scientist" => Role::DataScientist,
            "Data Analyst" => Role::DataAnalyst,
            "Full-stack Engineer" => Role::FullStackEngineer,
            "Backend Engineer" => Role::BackendEngineer,
            "Frontend Engineer" => Role::FrontendEngineer,
            "Blockchain Engineer" => Role::BlockchainEngineer,
            "QA Engineer" => Role::QaEngineer,
            "Project Manager" => Role::ProjectManager,
            "UI/UX Designer" => Role::UiUxDesigner,
            "Mobile Engineer" => Role::MobileEngineer,
            "DevOps Engineer" => Role::DevOpsEngineer,
            "Operations Manager" => Role::OperationsManager,
            _ => Role::FullStackEngineer, // Or handle the error/unknown case appropriately
        };
        let fixed_days: Vec<Weekday> = serde_json::from_str(&fixed_days_json).unwrap_or_default();

        Ok(Employee {
            id,
            name,
            sex,
            role,
            required_days,
            fixed_days,
            is_nsp: is_nsp != 0,
        })
    })?;

    let mut employees = Vec::new();
    for employee in employee_iter {
        employees.push(employee?);
    }
    Ok(employees)
}

pub fn create_schedules_table(conn: &Connection) -> SqliteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schedules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            year INTEGER NOT NULL,
            month INTEGER NOT NULL,
            schedule_data TEXT NOT NULL,  -- JSON serialized MonthlySchedule
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(year, month)
        )",
        [],
    )?;
    Ok(())
}

pub fn save_schedule_to_db(
    conn: &Connection,
    year: i32,
    month: u32,
    schedule: &MonthlySchedule,
) -> SqliteResult<()> {
    let schedule_json = serde_json::to_string(schedule).unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO schedules (year, month, schedule_data) VALUES (?1, ?2, ?3)",
        params![year, month, schedule_json],
    )?;
    Ok(())
}

pub fn load_schedule_from_db(
    conn: &Connection,
    year: i32,
    month: u32,
) -> SqliteResult<Option<MonthlySchedule>> {
    let mut stmt = conn.prepare(
        "SELECT schedule_data FROM schedules WHERE year = ?1 AND month = ?2 ORDER BY created_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query_map(params![year, month], |row| {
        let data: String = row.get(0)?;
        Ok(data)
    })?;

    if let Some(row) = rows.next() {
        let data = row?;
        let schedule: MonthlySchedule = serde_json::from_str(&data).unwrap();
        Ok(Some(schedule))
    } else {
        Ok(None)
    }
}

// RESET METHODS
pub fn delete_all_employees(conn: &Connection) -> SqliteResult<()> {
    conn.execute("DELETE FROM employees", [])?;
    Ok(())
}

pub fn delete_all_schedules(conn: &Connection) -> SqliteResult<()> {
    conn.execute("DELETE FROM schedules", [])?;
    Ok(())
}
