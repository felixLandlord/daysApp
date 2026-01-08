use crate::state::AppState;
use gpui::*;
use gpui_component::checkbox::Checkbox;
use std::sync::Arc;

pub struct SettingsView {
    state: Arc<AppState>,
}

impl SettingsView {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let config = cx.background_executor().block({
            let state = self.state.clone();
            async move { state.get_config().await }
        });

        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                div()
                    .text_2xl()
                    .font_weight(FontWeight::BOLD)
                    .child("Settings"),
            )
            .child(self.render_schedule_settings(&config, window, cx))
            .child(self.render_integration_settings(&config, window, cx))
            .child(self.render_keyboard_shortcuts(&config, window, cx))
    }
}

impl SettingsView {
    fn render_schedule_settings(
        &self,
        config: &common::AppConfig,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_1()
            .rounded_lg()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Schedule Configuration"),
            )
            .child(
                Checkbox::new("sex-distribution")
                    .label("Consider sex distribution")
                    .checked(config.schedule.consider_sex_distribution),
            )
            .child(
                Checkbox::new("role-distribution")
                    .label("Consider role distribution")
                    .checked(config.schedule.consider_role_distribution),
            )
    }

    fn render_integration_settings(
        &self,
        config: &common::AppConfig,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_1()
            .rounded_lg()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Integrations"),
            )
            .child(
                Checkbox::new("google-enabled")
                    .label("Enable Google Sheets")
                    .checked(config.integrations.google_enabled),
            )
            .child(
                Checkbox::new("discord-enabled")
                    .label("Enable Discord Notifications")
                    .checked(config.integrations.discord_enabled),
            )
    }

    fn render_keyboard_shortcuts(
        &self,
        config: &common::AppConfig,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_1()
            .rounded_lg()
            .child(
                div()
                    .text_lg()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Keyboard Shortcuts"),
            )
            .child(format!("Save: {}", config.shortcuts.save))
            .child(format!("Undo: {}", config.shortcuts.undo))
            .child(format!("Redo: {}", config.shortcuts.redo))
    }
}
