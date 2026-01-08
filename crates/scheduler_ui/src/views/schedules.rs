use crate::app::Message;
use crate::state::AppState;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Element, Length};
use scheduler_core::Weekday;

pub struct SchedulesView;

impl SchedulesView {
    pub fn view(state: &AppState) -> Element<'static, Message> {
        let mut content = column![
            text("Schedules").size(24),
            row![
                text("Year:"),
                text_input("Year", &state.selected_year.to_string())
                    .on_input(Message::YearChanged)
                    .width(80),
                text("Month:"),
                text_input("Month", &state.selected_month.to_string())
                    .on_input(Message::MonthChanged)
                    .width(60),
            ]
            .spacing(10),
            row![
                button("Generate Schedule").on_press(Message::GenerateSchedule),
                button("Save Schedule").on_press(Message::SaveSchedule),
                button("Import").on_press(Message::ImportSchedule),
                button("Export").on_press(Message::ExportSchedule),
            ]
            .spacing(10),
        ]
        .spacing(20);

        if let Some(schedule) = &state.current_schedule {
            content = content
                .push(text(format!("Schedule for {}/{}", schedule.month, schedule.year)).size(20));

            // Create a table-like layout
            let days = Weekday::all();
            let mut schedule_display = column![].spacing(10);

            for day in days {
                let employee_ids = schedule.get_employees_for_day(day);
                let employee_names: Vec<String> = employee_ids
                    .iter()
                    .filter_map(|id| {
                        state
                            .employees
                            .iter()
                            .find(|e| e.id == *id)
                            .map(|e| e.name.clone())
                    })
                    .collect();

                let day_row = column![
                    text(format!("{} ({} employees)", day, employee_ids.len())).size(16),
                    text(if employee_names.is_empty() {
                        "No employees scheduled".to_string()
                    } else {
                        employee_names.join(", ")
                    }),
                ]
                .spacing(5)
                .padding(10);

                schedule_display = schedule_display.push(
                    container(day_row)
                        .style(|theme: &iced::Theme| container::Style {
                            border: iced::Border {
                                width: 1.0,
                                color: theme.palette().text,
                                radius: 5.0.into(),
                            },
                            ..Default::default()
                        })
                        .padding(10),
                );
            }

            content = content.push(scrollable(schedule_display).height(Length::Fill));
        } else {
            content = content.push(text(
                "No schedule generated. Click 'Generate Schedule' to create one.",
            ));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
