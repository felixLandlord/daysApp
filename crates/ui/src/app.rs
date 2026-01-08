use crate::components::{EmployeesView, SchedulesView, SettingsView};
use crate::state::{AppState, Tab};
use crate::text_input::TextInput;
use gpui::{prelude::FluentBuilder, *};
use gpui_component::{button::Button, Selectable};
use std::sync::Arc;
use storage::{Database, DatabaseConfig};

pub struct AppView {
    state: Arc<AppState>,
    database: Arc<Database>,
    focus_handle: FocusHandle,
}

pub enum AppEvent {
    StateChanged,
}

impl EventEmitter<AppEvent> for AppView {}

impl Focusable for AppView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// impl AppView {
//     pub fn new(cx: &mut Context<Self>) -> Self {
//         // Initialize database
//         let db_config = DatabaseConfig::default();
//         let database = cx
//             .background_executor()
//             .block(Database::new(db_config))
//             .expect("Failed to initialize database");
//         let database = Arc::new(database);

//         // Load config
//         let config = cx
//             .background_executor()
//             .block(database.get_config())
//             .unwrap_or_default();

//         // Load employees
//         let employees = cx
//             .background_executor()
//             .block(database.get_all_employees())
//             .unwrap_or_default();

//         let state = Arc::new(AppState::new(config));

//         cx.background_executor()
//             .spawn({
//                 let state = state.clone();
//                 async move {
//                     state.set_employees(employees).await;
//                 }
//             })
//             .detach();

//         let focus_handle = cx.focus_handle();

//         Self {
//             state,
//             database,
//             focus_handle,
//         }
//     }
impl AppView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Create a placeholder state first
        let state = Arc::new(AppState::new(Default::default()));
        let focus_handle = cx.focus_handle();

        // Initialize database asynchronously
        let db_config = DatabaseConfig::default();

        // Use background_executor to create the database in an async context
        let database = Arc::new(
            cx.background_executor()
                .block(async move { Database::new(db_config).await })
                .expect("Failed to initialize database"),
        );

        // Now load config and employees
        let (config, employees) = cx.background_executor().block({
            let db = database.clone();
            async move {
                let config = db.get_config().await.unwrap_or_default();
                let employees = db.get_all_employees().await.unwrap_or_default();
                (config, employees)
            }
        });

        // Update state with loaded config
        let state = Arc::new(AppState::new(config));

        // Set employees asynchronously
        cx.background_executor()
            .spawn({
                let state = state.clone();
                async move {
                    state.set_employees(employees).await;
                }
            })
            .detach();

        Self {
            state,
            database,
            focus_handle,
        }
    }

    pub fn save_employee(&mut self, employee: common::Employee, cx: &mut Context<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();

        cx.background_executor()
            .spawn(async move {
                if let Ok(_) = db.save_employee(&employee).await {
                    state.add_employee(employee).await;
                }
            })
            .detach();
    }

    pub fn generate_schedule(&mut self, cx: &mut Context<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();

        cx.background_executor()
            .spawn(async move {
                let employees = state.get_employees().await.to_vec();
                let (year, month) = state.get_selected_date().await;
                let config = state.get_config().await;

                let past = db
                    .get_past_schedules(year, month, 3)
                    .await
                    .unwrap_or_default();

                let generator = scheduler::ScheduleGenerator::new(config);
                let options = scheduler::GenerationOptions {
                    year,
                    month,
                    past_schedules: past,
                };

                if let Ok(schedule) = generator.generate(&employees, options) {
                    state.set_schedule(Some(schedule)).await;
                }
            })
            .detach();
    }

    pub fn save_schedule(&mut self, cx: &mut Context<Self>) {
        let db = self.database.clone();
        let state = self.state.clone();

        cx.background_executor()
            .spawn(async move {
                if let Some(schedule) = state.get_schedule().await {
                    let _ = db.save_schedule(&schedule).await;
                }
            })
            .detach();
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let tab = cx
            .background_executor()
            .block(async move { state.get_tab().await });

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .child(self.render_menu_bar(cx))
            .child(self.render_tab_bar(cx))
            .child(self.render_content(tab, cx))
    }
}

impl AppView {
    fn render_menu_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let tab = cx
            .background_executor()
            .block(async move { state.get_tab().await });

        let state = self.state.clone();
        let is_editing = cx
            .background_executor()
            .block(async move { state.is_editing().await });

        div()
            .flex()
            .items_center()
            .justify_between()
            .h_12()
            .px_4()
            .border_b_1()
            .border_color(rgb(0xe0e0e0))
            .child(
                div()
                    .flex()
                    .gap_4()
                    .when(tab == Tab::Employees, |this| {
                        let state = self.state.clone();
                        let count = cx
                            .background_executor()
                            .block(async move { state.get_employees().await.len() });
                        this.child(format!("Total: {}", count))
                    })
                    .when(tab == Tab::Schedules, |this| this.child("Schedule Info")),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .when(tab == Tab::Employees, |this| {
                        this.child(Button::new("edit").label("Edit").on_click(cx.listener(
                            |this, _, _, cx| {
                                let state = this.state.clone();
                                cx.background_executor()
                                    .spawn(async move {
                                        let current = state.is_editing().await;
                                        state.set_editing(!current).await;
                                    })
                                    .detach();
                                cx.notify();
                            },
                        )))
                        .child(Button::new("save").label("Save"))
                        .child(Button::new("import").label("Import"))
                    })
                    .when(tab == Tab::Schedules, |this| {
                        this.child(
                            Button::new("generate")
                                .label("Generate")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.generate_schedule(cx);
                                    cx.notify();
                                })),
                        )
                        .child(Button::new("save").label("Save").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.save_schedule(cx);
                                cx.notify();
                            },
                        )))
                        .child(Button::new("export").label("Export"))
                    }),
            )
    }

    fn render_tab_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state.clone();
        let current_tab = cx
            .background_executor()
            .block(async move { state.get_tab().await });

        div()
            .flex()
            .h_12()
            .border_b_1()
            .border_color(rgb(0xe0e0e0))
            .child(self.render_tab(Tab::Employees, "Employees", current_tab, cx))
            .child(self.render_tab(Tab::Schedules, "Schedules", current_tab, cx))
            .child(self.render_tab(Tab::Settings, "Settings", current_tab, cx))
    }

    fn render_tab(
        &mut self,
        tab: Tab,
        label: &str,
        current: Tab,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Button::new(("tab", tab as u32))
            .label(label.to_string())
            .selected(tab == current)
            .on_click(cx.listener(move |this, _event, _, cx| {
                let state = this.state.clone();
                cx.background_executor()
                    .spawn(async move {
                        state.set_tab(tab).await;
                    })
                    .detach();
                cx.notify();
            }))
    }

    fn render_content(&mut self, tab: Tab, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex_1().overflow_hidden().p_4().child(match tab {
            Tab::Employees => {
                let view = cx.new(|_| EmployeesView::new(self.state.clone()));
                div().child(view)
            }
            Tab::Schedules => {
                let view = cx.new(|_| SchedulesView::new(self.state.clone()));
                div().child(view)
            }
            Tab::Settings => {
                let view = cx.new(|_| SettingsView::new(self.state.clone()));
                div().child(view)
            }
        })
    }
}
