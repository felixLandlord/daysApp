use crate::state::{AppState, Tab, UndoAction};
use crate::views::{EmployeesView, SchedulesView, SettingsView};
use chrono::Datelike;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Element, Length, Task, Theme};
// use iced::{
//     alignment, executor, keyboard, Application, Command, Element, Length, Settings, Subscription,
//     Task, Theme,
// };
use scheduler_core::{AppConfig, Employee, Sex};
use scheduler_engine::{GenerationOptions, ScheduleGenerator};
use scheduler_integrations::{CsvExporter, CsvImporter, JsonExporter, JsonImporter};
use scheduler_storage::{Database, DatabaseConfig};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    // Tab switching
    SwitchTab(Tab),

    // Employee operations
    EmployeeNameChanged(usize, String),
    EmployeeRequiredDaysChanged(usize, u8),
    ToggleEmployeeEditing,
    AddEmployee,
    DeleteEmployee(usize),
    SaveEmployees,

    // Schedule operations
    GenerateSchedule,
    SaveSchedule,
    YearChanged(String),
    MonthChanged(String),

    // Search
    SearchChanged(String),

    // Undo/Redo
    Undo,
    Redo,

    // Import/Export
    ImportEmployees,
    ExportEmployees,
    ImportSchedule,
    ExportSchedule,
    FileSelected(Option<PathBuf>),

    // Config
    ToggleSexDistribution,
    ToggleRoleDistribution,
    ChangeMenteeOverlap,

    // Database
    DataLoaded(Result<(Vec<Employee>, Option<scheduler_core::Schedule>, AppConfig), String>),
    EmployeesSaved(Result<(), String>),
    ScheduleSaved(Result<(), String>),

    // Status
    ClearStatus,
}

pub struct Application {
    state: AppState,
    db: Option<Arc<Database>>,
    pending_action: Option<PendingAction>,
}

#[derive(Debug, Clone)]
enum PendingAction {
    ImportEmployees,
    ExportEmployees,
    ImportSchedule,
    ExportSchedule,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            db: None,
            pending_action: None,
        }
    }
}

impl Application {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self::default();
        (app, Task::perform(Self::load_data(), Message::DataLoaded))
    }

    async fn load_data(
    ) -> Result<(Vec<Employee>, Option<scheduler_core::Schedule>, AppConfig), String> {
        let config = DatabaseConfig::default();
        let db = Database::new(config).await.map_err(|e| e.to_string())?;

        let employees = db.get_all_employees().await.map_err(|e| e.to_string())?;
        let config = db.get_config().await.map_err(|e| e.to_string())?;

        // Try to load current schedule
        let now = chrono::Local::now();
        let schedule = db.get_schedule(now.year(), now.month()).await.ok();

        Ok((employees, schedule, config))
    }

    pub fn title(&self) -> String {
        "Office Scheduler".to_string()
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.state.current_tab = tab;
                Task::none()
            }

            Message::ToggleEmployeeEditing => {
                self.state.is_editing = !self.state.is_editing;
                Task::none()
            }

            Message::AddEmployee => {
                let id = self.state.next_employee_id();
                let employee = Employee::new(id, format!("New Employee {}", id), Sex::Male);
                self.state
                    .push_undo(UndoAction::AddEmployee(employee.clone()));
                self.state.employees.push(employee);
                Task::none()
            }

            Message::DeleteEmployee(id) => {
                if let Some(pos) = self.state.employees.iter().position(|e| e.id == id) {
                    let employee = self.state.employees.remove(pos);
                    self.state
                        .push_undo(UndoAction::DeleteEmployee(id, employee));
                }
                Task::none()
            }

            Message::EmployeeNameChanged(id, name) => {
                if let Some(emp) = self.state.employees.iter_mut().find(|e| e.id == id) {
                    emp.name = name;
                }
                Task::none()
            }

            Message::EmployeeRequiredDaysChanged(id, days) => {
                if let Some(emp) = self.state.employees.iter_mut().find(|e| e.id == id) {
                    emp.required_days = days.min(5);
                }
                Task::none()
            }

            Message::SaveEmployees => {
                if let Some(db) = &self.db {
                    let db = Arc::clone(db);
                    let employees = self.state.employees.clone();
                    Task::perform(
                        async move {
                            for emp in &employees {
                                db.save_employee(emp).await.map_err(|e| e.to_string())?;
                            }
                            Ok(())
                        },
                        Message::EmployeesSaved,
                    )
                } else {
                    Task::none()
                }
            }

            Message::GenerateSchedule => {
                let config = self.state.config.clone();
                let employees = self.state.employees.clone();
                let year = self.state.selected_year;
                let month = self.state.selected_month;

                let db = self.db.clone();

                Task::perform(
                    async move {
                        let past_schedules = if let Some(db) = db {
                            db.get_past_schedules(year, month, 3)
                                .await
                                .unwrap_or_default()
                        } else {
                            Default::default()
                        };

                        let generator = ScheduleGenerator::new(config);
                        let options = GenerationOptions {
                            year,
                            month,
                            past_schedules,
                        };

                        generator
                            .generate(&employees, options)
                            .map_err(|e| e.to_string())
                    },
                    |result| match result {
                        Ok(schedule) => {
                            // Return a message to update state with the schedule
                            Message::DataLoaded(Ok((vec![], Some(schedule), AppConfig::default())))
                        }
                        Err(e) => Message::DataLoaded(Err(e)),
                    },
                )
            }

            Message::SaveSchedule => {
                if let (Some(db), Some(schedule)) = (&self.db, &self.state.current_schedule) {
                    let db = Arc::clone(db);
                    let schedule = schedule.clone();
                    Task::perform(
                        async move { db.save_schedule(&schedule).await.map_err(|e| e.to_string()) },
                        Message::ScheduleSaved,
                    )
                } else {
                    Task::none()
                }
            }

            Message::YearChanged(year_str) => {
                if let Ok(year) = year_str.parse() {
                    self.state.selected_year = year;
                }
                Task::none()
            }

            Message::MonthChanged(month_str) => {
                if let Ok(month) = month_str.parse::<u32>() {
                    self.state.selected_month = month.clamp(1, 12);
                }
                Task::none()
            }

            Message::SearchChanged(query) => {
                self.state.search_query = query;
                Task::none()
            }

            Message::Undo => {
                self.state.undo();
                Task::none()
            }

            Message::Redo => {
                self.state.redo();
                Task::none()
            }

            Message::ImportEmployees => {
                self.pending_action = Some(PendingAction::ImportEmployees);
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("CSV", &["csv"])
                            .add_filter("JSON", &["json"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }

            Message::ExportEmployees => {
                self.pending_action = Some(PendingAction::ExportEmployees);
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("CSV", &["csv"])
                            .add_filter("JSON", &["json"])
                            .save_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }

            Message::ImportSchedule => {
                self.pending_action = Some(PendingAction::ImportSchedule);
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("CSV", &["csv"])
                            .add_filter("JSON", &["json"])
                            .pick_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }

            Message::ExportSchedule => {
                self.pending_action = Some(PendingAction::ExportSchedule);
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("CSV", &["csv"])
                            .add_filter("JSON", &["json"])
                            .save_file()
                            .await
                            .map(|f| f.path().to_path_buf())
                    },
                    Message::FileSelected,
                )
            }

            Message::FileSelected(path) => {
                if let (Some(path), Some(action)) = (path, self.pending_action.take()) {
                    match action {
                        PendingAction::ImportEmployees => {
                            let result = if path.extension().and_then(|s| s.to_str()) == Some("csv")
                            {
                                CsvImporter::import_employees(&path)
                            } else {
                                JsonImporter::import_employees(&path)
                            };

                            if let Ok(employees) = result {
                                self.state.employees = employees;
                                self.state.status_message = Some("Employees imported".to_string());
                            }
                        }
                        PendingAction::ExportEmployees => {
                            let result = if path.extension().and_then(|s| s.to_str()) == Some("csv")
                            {
                                CsvExporter::export_employees(&self.state.employees, &path)
                            } else {
                                JsonExporter::export_employees(&self.state.employees, &path)
                            };

                            if result.is_ok() {
                                self.state.status_message = Some("Employees exported".to_string());
                            }
                        }
                        PendingAction::ImportSchedule => {
                            let result = if path.extension().and_then(|s| s.to_str()) == Some("csv")
                            {
                                CsvImporter::import_schedule(&path)
                            } else {
                                JsonImporter::import_schedule(&path)
                            };

                            if let Ok(schedule) = result {
                                self.state.current_schedule = Some(schedule);
                                self.state.status_message = Some("Schedule imported".to_string());
                            }
                        }
                        PendingAction::ExportSchedule => {
                            if let Some(schedule) = &self.state.current_schedule {
                                let result =
                                    if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                                        CsvExporter::export_schedule(
                                            schedule,
                                            &self.state.employees,
                                            &path,
                                        )
                                    } else {
                                        JsonExporter::export_schedule(schedule, &path)
                                    };

                                if result.is_ok() {
                                    self.state.status_message =
                                        Some("Schedule exported".to_string());
                                }
                            }
                        }
                    }
                }
                Task::none()
            }

            Message::ToggleSexDistribution => {
                self.state.config.schedule.consider_sex_distribution =
                    !self.state.config.schedule.consider_sex_distribution;
                Task::none()
            }

            Message::ToggleRoleDistribution => {
                self.state.config.schedule.consider_role_distribution =
                    !self.state.config.schedule.consider_role_distribution;
                Task::none()
            }

            Message::ChangeMenteeOverlap => {
                use scheduler_core::MenteeOverlapMode;
                self.state.config.schedule.mentee_mentor_overlap =
                    match self.state.config.schedule.mentee_mentor_overlap {
                        MenteeOverlapMode::None => MenteeOverlapMode::AtLeastOne,
                        MenteeOverlapMode::AtLeastOne => MenteeOverlapMode::AtLeastTwo,
                        MenteeOverlapMode::AtLeastTwo => MenteeOverlapMode::None,
                    };
                Task::none()
            }

            Message::DataLoaded(result) => match result {
                Ok((employees, schedule, config)) => {
                    if !employees.is_empty() {
                        self.state.employees = employees;
                    }
                    if let Some(sched) = schedule {
                        self.state.current_schedule = Some(sched);
                    }
                    if self.state.config.schedule.consider_sex_distribution == false
                        && config.schedule.consider_sex_distribution
                    {
                        self.state.config = config;
                    }

                    // Initialize database
                    Task::perform(
                        async {
                            let config = DatabaseConfig::default();
                            Database::new(config).await.ok()
                        },
                        |db| {
                            if let Some(db) = db {
                                Message::DataLoaded(Ok((vec![], None, AppConfig::default())))
                            } else {
                                Message::ClearStatus
                            }
                        },
                    )
                }
                Err(e) => {
                    self.state.status_message = Some(format!("Error: {}", e));
                    Task::none()
                }
            },

            Message::EmployeesSaved(result) => {
                match result {
                    Ok(_) => {
                        self.state.status_message = Some("Employees saved".to_string());
                    }
                    Err(e) => {
                        self.state.status_message = Some(format!("Error: {}", e));
                    }
                }
                Task::none()
            }

            Message::ScheduleSaved(result) => {
                match result {
                    Ok(_) => {
                        self.state.status_message = Some("Schedule saved".to_string());
                    }
                    Err(e) => {
                        self.state.status_message = Some(format!("Error: {}", e));
                    }
                }
                Task::none()
            }

            Message::ClearStatus => {
                self.state.status_message = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content = column![
            self.render_menu_bar(),
            self.render_tab_bar(),
            self.render_content()
        ]
        .spacing(0);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn render_menu_bar(&self) -> Element<Message> {
        let tab = self.state.current_tab;

        let left_section = match tab {
            Tab::Employees => row![text(format!(
                "Total: {} employees",
                self.state.employees.len()
            ))]
            .spacing(10),
            Tab::Schedules => {
                if let Some(schedule) = &self.state.current_schedule {
                    row![text(format!(
                        "Mon: {} | Tue: {} | Wed: {} | Thu: {} | Fri: {}",
                        schedule.count_for_day(scheduler_core::Weekday::Monday),
                        schedule.count_for_day(scheduler_core::Weekday::Tuesday),
                        schedule.count_for_day(scheduler_core::Weekday::Wednesday),
                        schedule.count_for_day(scheduler_core::Weekday::Thursday),
                        schedule.count_for_day(scheduler_core::Weekday::Friday),
                    ))]
                    .spacing(10)
                } else {
                    row![text("No schedule generated")].spacing(10)
                }
            }
            Tab::Settings => row![text("Configuration")].spacing(10),
        };

        let right_section = row![
            button("Undo")
                .on_press_maybe((!self.state.undo_stack.is_empty()).then_some(Message::Undo)),
            button("Redo")
                .on_press_maybe((!self.state.redo_stack.is_empty()).then_some(Message::Redo)),
            match tab {
                Tab::Employees => button("Edit").on_press(Message::ToggleEmployeeEditing),
                Tab::Schedules => button("Generate").on_press(Message::GenerateSchedule),
                Tab::Settings => button("Save").on_press(Message::SaveEmployees),
            },
            text_input("Search...", &self.state.search_query)
                .on_input(Message::SearchChanged)
                .width(200),
        ]
        .spacing(10);

        let menu = row![
            left_section,
            Space::new().width(Length::Fill),
            right_section
        ]
        .padding(10)
        .spacing(20);

        container(menu)
            .width(Length::Fill)
            .style(|theme: &Theme| container::Style {
                border: iced::Border {
                    width: 0.0,
                    color: theme.palette().background,
                    radius: 0.0.into(),
                },
                background: Some(theme.palette().background.into()),
                ..Default::default()
            })
            .into()
    }

    fn render_tab_bar(&self) -> Element<Message> {
        let tabs = row![
            button("Employees")
                .on_press(Message::SwitchTab(Tab::Employees))
                .style(if self.state.current_tab == Tab::Employees {
                    button::primary
                } else {
                    button::secondary
                }),
            button("Schedules")
                .on_press(Message::SwitchTab(Tab::Schedules))
                .style(if self.state.current_tab == Tab::Schedules {
                    button::primary
                } else {
                    button::secondary
                }),
            button("Settings")
                .on_press(Message::SwitchTab(Tab::Settings))
                .style(if self.state.current_tab == Tab::Settings {
                    button::primary
                } else {
                    button::secondary
                }),
        ]
        .spacing(5)
        .padding(10);

        container(tabs).width(Length::Fill).into()
    }

    fn render_content(&self) -> Element<Message> {
        let content: Element<Message> = match self.state.current_tab {
            Tab::Employees => EmployeesView::view(&self.state),
            Tab::Schedules => SchedulesView::view(&self.state),
            Tab::Settings => SettingsView::view(&self.state),
        };

        let mut col = column![content].padding(20).spacing(10);

        if let Some(msg) = &self.state.status_message {
            col = col.push(text(msg));
        }

        container(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
