use iced::Font;
use scheduler_ui::Application;
use tracing_subscriber;

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Run the iced application
    iced::application(Application::new, Application::update, Application::view)
        .theme(Application::theme)
        .default_font(Font::MONOSPACE)
        .run()
}
