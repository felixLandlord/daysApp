use crate::state::AppState;
use common::Employee;
use gpui::*;
use std::sync::Arc;

pub struct EmployeesView {
    state: Arc<AppState>,
}

impl EmployeesView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl Render for EmployeesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let employees = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_filtered_employees().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Employees"),
            )
            .child(
                div().grid().gap_4().children(
                    employees
                        .iter()
                        .map(|emp| self.render_employee_card(emp, window, cx)),
                ),
            )
    }
}

impl EmployeesView {
    fn render_employee_card(
        &self,
        employee: &Employee,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .p_4()
            .rounded_lg()
            .border_1()
            .border_color(rgb(0xe0e0e0))
            .bg(rgb(0xffffff))
            .hover(|style| style.bg(rgb(0xf5f5f5)))
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(employee.name.clone()),
            )
            .child(
                div().text_sm().text_color(rgb(0x666666)).child(
                    employee
                        .role
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "No role".to_string()),
                ),
            )
            .child(
                div()
                    .text_sm()
                    .child(format!("Required days: {}", employee.required_days)),
            )
    }
}
