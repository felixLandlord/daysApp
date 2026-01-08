use chrono::Datelike;
use scheduler_core::{AppConfig, Employee, EmployeeId, Schedule};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Employees,
    Schedules,
    Settings,
}

#[derive(Debug, Clone)]
pub enum UndoAction {
    AddEmployee(Employee),
    DeleteEmployee(EmployeeId, Employee),
    UpdateEmployee { old: Employee, new: Employee },
    UpdateSchedule { old: Schedule, new: Schedule },
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub employees: Vec<Employee>,
    pub current_schedule: Option<Schedule>,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub search_query: String,
    pub is_editing: bool,
    pub undo_stack: VecDeque<UndoAction>,
    pub redo_stack: VecDeque<UndoAction>,
    pub selected_year: i32,
    pub selected_month: u32,
    pub status_message: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let now = chrono::Local::now();
        Self {
            employees: Vec::new(),
            current_schedule: None,
            config: AppConfig::default(),
            current_tab: Tab::Schedules,
            search_query: String::new(),
            is_editing: false,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            selected_year: now.year(),
            selected_month: now.month(),
            status_message: None,
        }
    }
}

impl AppState {
    pub fn get_filtered_employees(&self) -> Vec<Employee> {
        let query = self.search_query.to_lowercase();

        if query.is_empty() {
            self.employees.clone()
        } else {
            self.employees
                .iter()
                .filter(|e| {
                    e.name.to_lowercase().contains(&query)
                        || e.role
                            .map(|r| r.to_string().to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    }

    pub fn add_employee(&mut self, employee: Employee) {
        self.undo_stack
            .push_back(UndoAction::AddEmployee(employee.clone()));
        self.redo_stack.clear();
        self.employees.push(employee);
    }

    pub fn update_employee(&mut self, employee: Employee) {
        if let Some(pos) = self.employees.iter().position(|e| e.id == employee.id) {
            let old = self.employees[pos].clone();
            self.undo_stack.push_back(UndoAction::UpdateEmployee {
                old,
                new: employee.clone(),
            });
            self.redo_stack.clear();
            self.employees[pos] = employee;
        }
    }

    pub fn delete_employee(&mut self, id: EmployeeId, employee: Employee) {
        if let Some(pos) = self.employees.iter().position(|e| e.id == id) {
            let employee = self.employees.remove(pos);
            self.undo_stack
                .push_back(UndoAction::DeleteEmployee(id, employee));
            self.redo_stack.clear();
        }
    }

    pub fn set_schedule(&mut self, schedule: Schedule) {
        if let Some(old) = self.current_schedule.clone() {
            self.undo_stack.push_back(UndoAction::UpdateSchedule {
                old,
                new: schedule.clone(),
            });
            self.redo_stack.clear();
        }
        self.current_schedule = Some(schedule);
    }

    pub fn next_employee_id(&self) -> EmployeeId {
        self.employees.iter().map(|e| e.id).max().unwrap_or(0) + 1
    }

    pub fn push_undo(&mut self, action: UndoAction) {
        self.undo_stack.push_back(action);
        if self.undo_stack.len() > 50 {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.undo_stack.pop_back() {
            match &action {
                UndoAction::AddEmployee(emp) => {
                    self.employees.retain(|e| e.id != emp.id);
                }
                UndoAction::DeleteEmployee(id, emp) => {
                    self.employees.push(emp.clone());
                }
                UndoAction::UpdateEmployee { old, new } => {
                    if let Some(pos) = self.employees.iter().position(|e| e.id == new.id) {
                        self.employees[pos] = old.clone();
                    }
                }
                UndoAction::UpdateSchedule { old, .. } => {
                    self.current_schedule = Some(old.clone());
                }
            }
            self.redo_stack.push_back(action);
        }
    }

    pub fn redo(&mut self) {
        if let Some(action) = self.redo_stack.pop_back() {
            match &action {
                UndoAction::AddEmployee(emp) => {
                    self.employees.push(emp.clone());
                }
                UndoAction::DeleteEmployee(id, _) => {
                    self.employees.retain(|e| e.id != *id);
                }
                UndoAction::UpdateEmployee { new, .. } => {
                    if let Some(pos) = self.employees.iter().position(|e| e.id == new.id) {
                        self.employees[pos] = new.clone();
                    }
                }
                UndoAction::UpdateSchedule { new, .. } => {
                    self.current_schedule = Some(new.clone());
                }
            }
            self.undo_stack.push_back(action);
        }
    }
}
