use crate::state::AppState;
use common::Weekday;
use gpui::*;
use std::sync::Arc;

pub struct SchedulesView {
    state: Arc<AppState>,
}

impl SchedulesView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl Render for SchedulesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let schedule = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_schedule().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Schedule"),
            )
            .child(match schedule {
                Some(s) => self.render_schedule(&s, window, cx).into_any_element(),
                None => div().child("No schedule generated").into_any_element(),
            })
    }
}

impl SchedulesView {
    fn render_schedule(
        &self,
        schedule: &common::Schedule,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let employees = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_employees().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .gap_8()
                    .children(Weekday::all().iter().map(|day| {
                        let count = schedule.count_for_day(*day);
                        let emp_ids = schedule.get_employees_for_day(*day);

                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_4()
                            .border_1()
                            .rounded_lg()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(format!("{} ({})", day, count)),
                            )
                            .children(emp_ids.iter().filter_map(|id| {
                                employees.iter().find(|e| e.id == *id).map(|emp| {
                                    div()
                                        .p_2()
                                        .rounded(px(4.0))
                                        .bg(rgb(0xf0f0f0))
                                        .child(emp.name.clone())
                                })
                            }))
                    })),
            )
    }
}
