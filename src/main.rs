// use anyhow::Result;
// use gpui::{App, *};
// // use ui::App;

// fn main() {
//     env_logger::init();

//     Application::new().run(|cx: &mut App| {
//         cx.open_window(
//             WindowOptions {
//                 window_bounds: Some(WindowBounds::Windowed(Bounds {
//                     origin: Point::new(Pixels(100.0), Pixels(100.0)),
//                     size: Size {
//                         width: Pixels(1200.0),
//                         height: Pixels(800.0),
//                     },
//                 })),
//                 titlebar: Some(TitlebarOptions {
//                     title: Some("Office Scheduler".into()),
//                     appears_transparent: false,
//                     traffic_light_position: None,
//                 }),
//                 window_min_size: Some(Size {
//                     width: Pixels(800.0),
//                     height: Pixels(600.0),
//                 }),
//                 ..Default::default()
//             },
//             |cx| cx.new_view(|cx| ui::App::new(cx)),
//         );
//     });
// }
use gpui::*;

actions!(
    scheduler,
    [
        Quit,
        Save,
        Undo,
        Redo,
        Delete,
        SwitchToEmployees,
        SwitchToSchedules,
        SwitchToSettings,
        NewEmployee,
        GenerateSchedule,
        ExportSchedule,
    ]
);

fn main() {
    env_logger::init();

    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-s", Save, None),
            KeyBinding::new("cmd-z", Undo, None),
            KeyBinding::new("cmd-shift-z", Redo, None),
            KeyBinding::new("cmd-backspace", Delete, None),
            KeyBinding::new("cmd-1", SwitchToEmployees, None),
            KeyBinding::new("cmd-2", SwitchToSchedules, None),
            KeyBinding::new("cmd-3", SwitchToSettings, None),
            KeyBinding::new("cmd-n", NewEmployee, None),
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
            |_, cx| cx.new(|cx| ui::AppView::new(cx)),
        )
        .unwrap();
    });
}
