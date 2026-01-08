use crate::app::Message;
use crate::state::AppState;
use iced::widget::{button, checkbox, column, container, text};
use iced::{Element, Length};

pub struct SettingsView;

impl SettingsView {
    pub fn view(state: &AppState) -> Element<'static, Message> {
        let content = column![
            text("Settings").size(24),
            text("Schedule Configuration").size(18),
            checkbox(state.config.schedule.consider_sex_distribution)
                .label("Consider sex distribution")
                .on_toggle(|_| Message::ToggleSexDistribution),
            checkbox(state.config.schedule.consider_role_distribution)
                .label("Consider role distribution")
                .on_toggle(|_| Message::ToggleRoleDistribution),
            column![
                text("Mentee-Mentor Overlap:"),
                button(text(
                    state.config.schedule.mentee_mentor_overlap.to_string()
                ))
                .on_press(Message::ChangeMenteeOverlap),
            ]
            .spacing(5),
            text("Keyboard Shortcuts").size(18),
            column![
                text(format!("Save: {}", state.config.shortcuts.save)),
                text(format!("Undo: {}", state.config.shortcuts.undo)),
                text(format!("Redo: {}", state.config.shortcuts.redo)),
                text(format!("Delete: {}", state.config.shortcuts.delete)),
                text(format!(
                    "Tab Employees: {}",
                    state.config.shortcuts.tab_employees
                )),
                text(format!(
                    "Tab Schedules: {}",
                    state.config.shortcuts.tab_schedules
                )),
                text(format!(
                    "Tab Settings: {}",
                    state.config.shortcuts.tab_settings
                )),
                text(format!("Search: {}", state.config.shortcuts.search)),
            ]
            .spacing(5),
        ]
        .spacing(20)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
