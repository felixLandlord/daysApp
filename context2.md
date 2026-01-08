ermm i dont think you did everything right

you manually hand-rolled everything from scratch instead of using existing implementations:

use gpui::{
    App, Application, Bounds, ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler,
    Entity, EntityInputHandler, FocusHandle, Focusable, GlobalElementId, KeyBinding, Keystroke,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, UnderlineStyle, Window, WindowBounds,
    WindowOptions, actions, black, div, fill, hsla, opaque_grey, point, prelude::*, px, relative,
    rgb, rgba, size, white, yellow,
};



response:
You're absolutely right! I apologize for that mistake. Let me properly implement this using GPUI's actual components and patterns. Let me rebuild this correctly:

# src/main.rs
use gpui::*;

actions!(scheduler, [Quit, Save, Undo, Redo, Delete, SwitchToEmployees, SwitchToSchedules, SwitchToSettings]);

fn main() {
    env_logger::init();
    
    App::new().run(|cx: &mut AppContext| {
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-s", Save, None),
            KeyBinding::new("cmd-z", Undo, None),
            KeyBinding::new("cmd-shift-z", Redo, None),
            KeyBinding::new("cmd-backspace", Delete, None),
            KeyBinding::new("cmd-1", SwitchToEmployees, None),
            KeyBinding::new("cmd-2", SwitchToSchedules, None),
            KeyBinding::new("cmd-3", SwitchToSettings, None),
        ]);

        cx.activate(true);
        
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(100.0), px(100.0)),
                    size: size(px(1200.0), px(800.0)),
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Office Scheduler".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                window_min_size: Some(size(px(800.0), px(600.0))),
                ..Default::default()
            },
            |cx| cx.new_view(|cx| ui::AppView::new(cx)),
        ).unwrap();
    });
}

# crates/ui/src/app.rs
use crate::components::{EmployeesView, SchedulesView, SettingsView};
use crate::state::{AppState, Tab};
use core::AppConfig;
use gpui::*;
use storage::{Database, DatabaseConfig};
use std::sync::Arc;

pub struct AppView {
    state: Model<AppState>,
    database: Arc<Database>,
    focus_handle: FocusHandle,
}

impl AppView {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        // Initialize database
        let db_config = DatabaseConfig::default();
        let database = cx.background_executor().block(
            Database::new(db_config)
        ).expect("Failed to initialize database");
        
        let database = Arc::new(database);

        // Load config
        let config = cx.background_executor().block(database.get_config())
            .unwrap_or_default();

        // Load employees
        let employees = cx.background_executor().block(database.get_all_employees())
            .unwrap_or_default();

        let state = cx.new_model(|_| AppState::new(config, employees));

        // Setup actions
        cx.on_action({
            let state = state.clone();
            move |_, _: &crate::Save, cx| {
                cx.emit(AppEvent::Save);
            }
        });

        cx.on_action({
            let state = state.clone();
            move |_, _: &crate::Undo, cx| {
                state.update(cx, |state, cx| state.undo(cx));
            }
        });

        cx.on_action({
            let state = state.clone();
            move |_, _: &crate::Redo, cx| {
                state.update(cx, |state, cx| state.redo(cx));
            }
        });

        cx.on_action({
            let state = state.clone();
            move |_, _: &crate::SwitchToEmployees, cx| {
                state.update(cx, |state, cx| {
                    state.set_tab(Tab::Employees, cx);
                });
            }
        });

        cx.on_action({
            let state = state.clone();
            move |_, _: &crate::SwitchToSchedules, cx| {
                state.update(cx, |state, cx| {
                    state.set_tab(Tab::Schedules, cx);
                });
            }
        });

        cx.on_action({
            let state = state.clone();
            move |_, _: &crate::SwitchToSettings, cx| {
                state.update(cx, |state, cx| {
                    state.set_tab(Tab::Settings, cx);
                });
            }
        });

        let focus_handle = cx.focus_handle();

        Self { state, database, focus_handle }
    }

    fn save_employee(&mut self, employee: core::Employee, cx: &mut ViewContext<Self>) {
        let db = self.database.clone();
        
        cx.spawn(|_, _| async move {
            let _ = db.save_employee(&employee).await;
        }).detach();
    }

    fn generate_schedule(&mut self, cx: &mut ViewContext<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();
        
        cx.spawn(|this, mut cx| async move {
            let (employees, year, month, config) = state.update(&mut cx, |state, _| {
                (state.employees.clone(), state.selected_year, state.selected_month, state.config.clone())
            }).ok()?;
            
            let past = db.get_past_schedules(year, month, 3).await.ok()?;
            
            let generator = scheduler::ScheduleGenerator::new(config);
            let options = scheduler::GenerationOptions {
                year,
                month,
                past_schedules: past,
            };
            
            if let Ok(schedule) = generator.generate(&employees, options) {
                this.update(&mut cx, |this, cx| {
                    this.state.update(cx, |state, cx| {
                        state.set_schedule(Some(schedule), cx);
                    });
                }).ok()?;
            }
            
            Some(())
        }).detach();
    }

    fn save_schedule(&mut self, cx: &mut ViewContext<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();
        
        cx.spawn(|_, mut cx| async move {
            let schedule = state.read_with(&cx, |state, _| state.current_schedule.clone()).ok()??;
            db.save_schedule(&schedule).await.ok()?;
            Some(())
        }).detach();
    }
}

pub enum AppEvent {
    Save,
}

impl EventEmitter<AppEvent> for AppView {}

impl Focusable for AppView {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AppView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        
        div()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .bg(white())
            .child(self.render_menu_bar(cx))
            .child(self.render_tab_bar(cx))
            .child(self.render_content(state.current_tab, cx))
    }
}

impl AppView {
    fn render_menu_bar(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let tab = state.current_tab;
        let is_editing = state.is_editing;
        
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(48.0))
            .px(px(16.0))
            .border_b_1()
            .border_color(rgb(0xe0e0e0))
            .child(
                div()
                    .flex()
                    .gap(px(16.0))
                    .when(tab == Tab::Employees, |this| {
                        let count = state.employees.len();
                        this.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0x666666))
                                .child(format!("Total: {} | Males: {} | Females: {}", 
                                    count,
                                    state.employees.iter().filter(|e| matches!(e.sex, core::Sex::Male)).count(),
                                    state.employees.iter().filter(|e| matches!(e.sex, core::Sex::Female)).count()
                                ))
                        )
                    })
                    .when(tab == Tab::Schedules, |this| {
                        if let Some(schedule) = &state.current_schedule {
                            this.child(
                                div()
                                    .flex()
                                    .gap(px(12.0))
                                    .text_sm()
                                    .text_color(rgb(0x666666))
                                    .children(core::Weekday::all().iter().map(|day| {
                                        format!("{}: {}", day.to_string().chars().take(3).collect::<String>(), 
                                            schedule.count_for_day(*day))
                                    }))
                            )
                        } else {
                            this
                        }
                    })
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .when(tab == Tab::Employees, |this| {
                        this.child(self.render_button("Edit", is_editing, cx, |this, cx| {
                            this.state.update(cx, |state, cx| {
                                state.toggle_editing(cx);
                            });
                        }))
                        .child(self.render_button("Save", false, cx, |this, cx| {
                            cx.emit(AppEvent::Save);
                        }))
                        .child(self.render_button("Import", false, cx, |_, _| {}))
                    })
                    .when(tab == Tab::Schedules, |this| {
                        this.child(self.render_button("Generate", false, cx, |this, cx| {
                            this.generate_schedule(cx);
                        }))
                        .child(self.render_button("Save", false, cx, |this, cx| {
                            this.save_schedule(cx);
                        }))
                        .child(self.render_button("Export", false, cx, |_, _| {}))
                    })
            )
    }

    fn render_button(
        &self,
        label: &'static str,
        selected: bool,
        cx: &mut ViewContext<Self>,
        on_click: impl Fn(&mut Self, &mut ViewContext<Self>) + 'static,
    ) -> impl IntoElement {
        div()
            .px(px(12.0))
            .py(px(6.0))
            .rounded(px(6.0))
            .bg(if selected { rgb(0x0066cc) } else { rgb(0xf0f0f0) })
            .text_color(if selected { white() } else { black() })
            .hover(|style| style.bg(if selected { rgb(0x0052a3) } else { rgb(0xe0e0e0) }))
            .cursor(CursorStyle::PointingHand)
            .on_mouse_down(MouseButton::Left, move |_, cx| {
                cx.stop_propagation();
            })
            .on_click(cx.listener(move |this, _, cx| {
                on_click(this, cx);
            }))
            .child(div().text_sm().child(label))
    }

    fn render_tab_bar(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let current_tab = self.state.read(cx).current_tab;
        
        div()
            .flex()
            .h(px(48.0))
            .border_b_1()
            .border_color(rgb(0xe0e0e0))
            .child(self.render_tab(Tab::Employees, "Employees", current_tab, cx))
            .child(self.render_tab(Tab::Schedules, "Schedules", current_tab, cx))
            .child(self.render_tab(Tab::Settings, "Settings", current_tab, cx))
    }

    fn render_tab(&self, tab: Tab, label: &'static str, current: Tab, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let selected = tab == current;
        
        div()
            .px(px(24.0))
            .py(px(12.0))
            .border_b_2()
            .border_color(if selected { rgb(0x0066cc) } else { rgba(0x00000000) })
            .text_color(if selected { rgb(0x0066cc) } else { rgb(0x666666) })
            .font_weight(if selected { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
            .hover(|style| style.text_color(rgb(0x0066cc)))
            .cursor(CursorStyle::PointingHand)
            .on_click(cx.listener(move |this, _, cx| {
                this.state.update(cx, |state, cx| {
                    state.set_tab(tab, cx);
                });
            }))
            .child(label)
    }

    fn render_content(&self, tab: Tab, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .overflow_y_scroll()
            .p(px(16.0))
            .child(match tab {
                Tab::Employees => div().child(cx.new_view(|cx| EmployeesView::new(self.state.clone(), cx))),
                Tab::Schedules => div().child(cx.new_view(|cx| SchedulesView::new(self.state.clone(), cx))),
                Tab::Settings => div().child(cx.new_view(|cx| SettingsView::new(self.state.clone(), cx))),
            })
    }
}

# crates/ui/src/state.rs
use core::{AppConfig, Employee, EmployeeId, Schedule};
use gpui::*;

pub struct AppState {
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
    pub fn new(config: AppConfig, employees: Vec<Employee>) -> Self {
        let now = chrono::Local::now();
        Self {
            employees,
            current_schedule: None,
            config,
            current_tab: Tab::Schedules,
            search_query: String::new(),
            is_editing: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            selected_year: now.year(),
            selected_month: now.month(),
        }
    }

    pub fn set_tab(&mut self, tab: Tab, cx: &mut ModelContext<Self>) {
        self.current_tab = tab;
        cx.notify();
    }

    pub fn toggle_editing(&mut self, cx: &mut ModelContext<Self>) {
        self.is_editing = !self.is_editing;
        cx.notify();
    }

    pub fn add_employee(&mut self, employee: Employee, cx: &mut ModelContext<Self>) {
        self.undo_stack.push(UndoAction::AddEmployee(employee.clone()));
        self.redo_stack.clear();
        self.employees.push(employee);
        cx.notify();
    }

    pub fn update_employee(&mut self, employee: Employee, cx: &mut ModelContext<Self>) {
        if let Some(pos) = self.employees.iter().position(|e| e.id == employee.id) {
            let old = self.employees[pos].clone();
            self.undo_stack.push(UndoAction::UpdateEmployee {
                old: old.clone(),
                new: employee.clone(),
            });
            self.redo_stack.clear();
            self.employees[pos] = employee;
            cx.notify();
        }
    }

    pub fn delete_employee(&mut self, id: EmployeeId, cx: &mut ModelContext<Self>) {
        if let Some(pos) = self.employees.iter().position(|e| e.id == id) {
            let employee = self.employees.remove(pos);
            self.undo_stack.push(UndoAction::DeleteEmployee(id));
            self.redo_stack.clear();
            cx.notify();
        }
    }

    pub fn set_schedule(&mut self, schedule: Option<Schedule>, cx: &mut ModelContext<Self>) {
        if let Some(old) = self.current_schedule.clone() {
            if let Some(new) = schedule.clone() {
                self.undo_stack.push(UndoAction::UpdateSchedule { old, new: new.clone() });
                self.redo_stack.clear();
            }
        }
        self.current_schedule = schedule;
        cx.notify();
    }

    pub fn undo(&mut self, cx: &mut ModelContext<Self>) {
        if let Some(action) = self.undo_stack.pop() {
            match action.clone() {
                UndoAction::AddEmployee(emp) => {
                    self.employees.retain(|e| e.id != emp.id);
                }
                UndoAction::DeleteEmployee(_) => {}
                UndoAction::UpdateEmployee { old, new } => {
                    if let Some(pos) = self.employees.iter().position(|e| e.id == new.id) {
                        self.employees[pos] = old;
                    }
                }
                UndoAction::UpdateSchedule { old, .. } => {
                    self.current_schedule = Some(old);
                }
            }
            self.redo_stack.push(action);
            cx.notify();
        }
    }

    pub fn redo(&mut self, cx: &mut ModelContext<Self>) {
        if let Some(action) = self.redo_stack.pop() {
            match action.clone() {
                UndoAction::AddEmployee(emp) => {
                    self.employees.push(emp);
                }
                UndoAction::DeleteEmployee(id) => {
                    self.employees.retain(|e| e.id != id);
                }
                UndoAction::UpdateEmployee { new, .. } => {
                    if let Some(pos) = self.employees.iter().position(|e| e.id == new.id) {
                        self.employees[pos] = new;
                    }
                }
                UndoAction::UpdateSchedule { new, .. } => {
                    self.current_schedule = Some(new);
                }
            }
            self.undo_stack.push(action);
            cx.notify();
        }
    }

    pub fn get_filtered_employees(&self) -> Vec<Employee> {
        let query = self.search_query.to_lowercase();
        
        if query.is_empty() {
            self.employees.clone()
        } else {
            self.employees
                .iter()
                .filter(|e| {
                    e.name.to_lowercase().contains(&query)
                        || e.role.map(|r| r.to_string().to_lowercase().contains(&query)).unwrap_or(false)
                })
                .cloned()
                .collect()
        }
    }

    pub fn next_employee_id(&self) -> EmployeeId {
        self.employees
            .iter()
            .map(|e| e.id)
            .max()
            .unwrap_or(0)
            + 1
    }
}

# crates/ui/src/components/employees.rs
use crate::state::AppState;
use core::Employee;
use gpui::*;

pub struct EmployeesView {
    state: Model<AppState>,
}

impl EmployeesView {
    pub fn new(state: Model<AppState>, _cx: &mut ViewContext<Self>) -> Self {
        Self { state }
    }
}

impl Render for EmployeesView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let employees = state.get_filtered_employees();

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Employees")
            )
            .child(
                div()
                    .grid()
                    .gap(px(16.0))
                    .grid_cols_4()
                    .children(employees.iter().map(|emp| self.render_employee_card(emp, cx)))
            )
    }
}

impl EmployeesView {
    fn render_employee_card(&self, employee: &Employee, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .p(px(16.0))
            .rounded(px(8.0))
            .border_1()
            .border_color(rgb(0xe0e0e0))
            .bg(white())
            .hover(|style| style.bg(rgb(0xf5f5f5)))
            .cursor(CursorStyle::PointingHand)
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(&employee.name)
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x666666))
                    .child(employee.role.map(|r| r.to_string()).unwrap_or_else(|| "No role".to_string()))
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x888888))
                    .child(format!("Required days: {}", employee.required_days))
            )
    }
}

# crates/ui/src/components/schedules.rs
use crate::state::AppState;
use core::Weekday;
use gpui::*;

pub struct SchedulesView {
    state: Model<AppState>,
}

impl SchedulesView {
    pub fn new(state: Model<AppState>, _cx: &mut ViewContext<Self>) -> Self {
        Self { state }
    }
}

impl Render for SchedulesView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let schedule = state.current_schedule.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Schedule")
            )
            .child(match schedule {
                Some(s) => self.render_schedule(&s, &state.employees, cx),
                None => div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h(px(200.0))
                    .text_color(rgb(0x999999))
                    .child("No schedule generated. Click Generate to create one."),
            })
    }
}

impl SchedulesView {
    fn render_schedule(&self, schedule: &core::Schedule, employees: &[core::Employee], _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap(px(16.0))
            .children(Weekday::all().iter().map(|day| {
                let emp_ids = schedule.get_employees_for_day(*day);
                
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .gap(px(8.0))
                    .p(px(16.0))
                    .border_1()
                    .border_color(rgb(0xe0e0e0))
                    .rounded(px(8.0))
                    .bg(rgb(0xfafafa))
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_sm()
                            .child(format!("{} ({})", day, emp_ids.len()))
                    )
                    .children(emp_ids.iter().filter_map(|id| {
                        employees.iter().find(|e| e.id == *id).map(|emp| {
                            div()
                                .p(px(8.0))
                                .rounded(px(6.0))
                                .bg(white())
                                .border_1()
                                .border_color(rgb(0xe0e0e0))
                                .text_sm()
                                .child(&emp.name)
                        })
                    }))
            }))
    }
}

# crates/ui/src/components/settings.rs
use crate::state::AppState;
use gpui::*;

pub struct SettingsView {
    state: Model<AppState>,
}

impl SettingsView {
    pub fn new(state: Model<AppState>, _cx: &mut ViewContext<Self>) -> Self {
        Self { state }
    }
}

impl Render for SettingsView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let state = self.state.read(cx);
        let config = &state.config;

        div()
            .flex()
            .flex_col()
            .gap(px(24.0))
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Settings")
            )
            .child(self.render_schedule_settings(config, cx))
            .child(self.render_integration_settings(config, cx))
            .child(self.render_keyboard_shortcuts(config, cx))
    }
}

impl SettingsView {
    fn render_schedule_settings(&self, config: &core::AppConfig, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(16.0))
            .border_1()
            .border_color(rgb(0xe0e0e0))
            .rounded(px(8.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Schedule Configuration")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .border_1()
                            .border_color(rgb(0xcccccc))
                            .rounded(px(3.0))
                            .when(config.schedule.consider_sex_distribution, |this| {
                                this.bg(rgb(0x0066cc))
                            })
                    )
                    .child("Consider sex distribution")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .border_1()
                            .border_color(rgb(0xcccccc))
                            .rounded(px(3.0))
                            .when(config.schedule.consider_role_distribution, |this| {
                                this.bg(rgb(0x0066cc))
                            })
                    )
                    .child("Consider role distribution")
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Mentee-Mentor Overlap")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x666666))
                            .child(config.schedule.mentee_mentor_overlap.to_string())
                    )
            )
    }

    fn render_integration_settings(&self, config: &core::AppConfig, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(16.0))
            .border_1()
            .border_color(rgb(0xe0e0e0))
            .rounded(px(8.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Integrations")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .border_1()
                            .border_color(rgb(0xcccccc))
                            .rounded(px(3.0))
                            .when(config.integrations.google_enabled, |this| {
                                this.bg(rgb(0x0066cc))
                            })
                    )
                    .child("Enable Google Sheets")
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(16.0))
                            .h(px(16.0))
                            .border_1()
                            .border_color(rgb(0xcccccc))
                            .rounded(px(3.0))
                            .when(config.integrations.discord_enabled, |this| {
                                this.bg(rgb(0x0066cc))
                            })
                    )
                    .child("Enable Discord Notifications")
            )
    }

    fn render_keyboard_shortcuts(&self, config: &core::AppConfig, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
