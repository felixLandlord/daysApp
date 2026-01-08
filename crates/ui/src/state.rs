use common::{AppConfig, Employee, EmployeeId, Schedule};
// use std::collections::HashMap;
use chrono::Datelike;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
}

struct AppStateInner {
    pub employees: Vec<Employee>,
    pub current_schedule: Option<Schedule>,
    pub config: AppConfig,
    pub current_tab: Tab,
    pub search_query: String,
    pub is_editing: bool,
    pub undo_stack: Vec<UndoAction>,
    pub redo_stack: Vec<UndoAction>,
    pub selected_year: i32,
    pub selected_month: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Employees,
    Schedules,
    Settings,
}

#[derive(Debug, Clone)]
pub enum UndoAction {
    AddEmployee(Employee),
    DeleteEmployee(EmployeeId),
    UpdateEmployee { old: Employee, new: Employee },
    UpdateSchedule { old: Schedule, new: Schedule },
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let now = chrono::Local::now();
        Self {
            inner: Arc::new(RwLock::new(AppStateInner {
                employees: Vec::new(),
                current_schedule: None,
                config,
                current_tab: Tab::Schedules,
                search_query: String::new(),
                is_editing: false,
                undo_stack: Vec::new(),
                redo_stack: Vec::new(),
                selected_year: now.year(),
                selected_month: now.month(),
            })),
        }
    }

    pub async fn get_employees(&self) -> Vec<Employee> {
        self.inner.read().await.employees.clone()
    }

    pub async fn set_employees(&self, employees: Vec<Employee>) {
        self.inner.write().await.employees = employees;
    }

    pub async fn add_employee(&self, employee: Employee) {
        let mut inner = self.inner.write().await;
        inner
            .undo_stack
            .push(UndoAction::AddEmployee(employee.clone()));
        inner.redo_stack.clear();
        inner.employees.push(employee);
    }

    pub async fn update_employee(&self, employee: Employee) {
        let mut inner = self.inner.write().await;
        if let Some(pos) = inner.employees.iter().position(|e| e.id == employee.id) {
            let old = inner.employees[pos].clone();
            inner.undo_stack.push(UndoAction::UpdateEmployee {
                old: old.clone(),
                new: employee.clone(),
            });
            inner.redo_stack.clear();
            inner.employees[pos] = employee;
        }
    }

    pub async fn delete_employee(&self, id: EmployeeId) {
        let mut inner = self.inner.write().await;
        if let Some(pos) = inner.employees.iter().position(|e| e.id == id) {
            let _employee = inner.employees.remove(pos);
            inner.undo_stack.push(UndoAction::DeleteEmployee(id));
            inner.redo_stack.clear();
        }
    }

    pub async fn get_schedule(&self) -> Option<Schedule> {
        self.inner.read().await.current_schedule.clone()
    }

    pub async fn set_schedule(&self, schedule: Option<Schedule>) {
        let mut inner = self.inner.write().await;
        if let Some(old) = inner.current_schedule.clone() {
            if let Some(new) = schedule.clone() {
                inner.undo_stack.push(UndoAction::UpdateSchedule {
                    old,
                    new: new.clone(),
                });
                inner.redo_stack.clear();
            }
        }
        inner.current_schedule = schedule;
    }

    pub async fn get_config(&self) -> AppConfig {
        self.inner.read().await.config.clone()
    }

    pub async fn set_config(&self, config: AppConfig) {
        self.inner.write().await.config = config;
    }

    pub async fn get_tab(&self) -> Tab {
        self.inner.read().await.current_tab
    }

    pub async fn set_tab(&self, tab: Tab) {
        self.inner.write().await.current_tab = tab;
    }

    pub async fn get_search_query(&self) -> String {
        self.inner.read().await.search_query.clone()
    }

    pub async fn set_search_query(&self, query: String) {
        self.inner.write().await.search_query = query;
    }

    pub async fn is_editing(&self) -> bool {
        self.inner.read().await.is_editing
    }

    pub async fn set_editing(&self, editing: bool) {
        self.inner.write().await.is_editing = editing;
    }

    pub async fn undo(&self) {
        let mut inner = self.inner.write().await;
        if let Some(action) = inner.undo_stack.pop() {
            match action.clone() {
                UndoAction::AddEmployee(emp) => {
                    inner.employees.retain(|e| e.id != emp.id);
                }
                UndoAction::DeleteEmployee(id) => {
                    // Can't restore without saved data
                }
                UndoAction::UpdateEmployee { old, new } => {
                    if let Some(pos) = inner.employees.iter().position(|e| e.id == new.id) {
                        inner.employees[pos] = old;
                    }
                }
                UndoAction::UpdateSchedule { old, new } => {
                    inner.current_schedule = Some(old);
                }
            }
            inner.redo_stack.push(action);
        }
    }

    pub async fn redo(&self) {
        let mut inner = self.inner.write().await;
        if let Some(action) = inner.redo_stack.pop() {
            match action.clone() {
                UndoAction::AddEmployee(emp) => {
                    inner.employees.push(emp);
                }
                UndoAction::DeleteEmployee(id) => {
                    inner.employees.retain(|e| e.id != id);
                }
                UndoAction::UpdateEmployee { old, new } => {
                    if let Some(pos) = inner.employees.iter().position(|e| e.id == old.id) {
                        inner.employees[pos] = new;
                    }
                }
                UndoAction::UpdateSchedule { old, new } => {
                    inner.current_schedule = Some(new);
                }
            }
            inner.undo_stack.push(action);
        }
    }

    pub async fn get_selected_date(&self) -> (i32, u32) {
        let inner = self.inner.read().await;
        (inner.selected_year, inner.selected_month)
    }

    pub async fn set_selected_date(&self, year: i32, month: u32) {
        let mut inner = self.inner.write().await;
        inner.selected_year = year;
        inner.selected_month = month;
    }

    pub async fn get_filtered_employees(&self) -> Vec<Employee> {
        let inner = self.inner.read().await;
        let query = inner.search_query.to_lowercase();

        if query.is_empty() {
            inner.employees.clone()
        } else {
            inner
                .employees
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

    pub async fn next_employee_id(&self) -> EmployeeId {
        self.inner
            .read()
            .await
            .employees
            .iter()
            .map(|e| e.id)
            .max()
            .unwrap_or(0)
            + 1
    }
}
