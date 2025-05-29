use crate::client::routes::Route;
use dioxus::{logger::tracing::info, prelude::*};

const GLOBAL_CSS: Asset = asset!("/assets/styles/global.css");

#[component]
pub fn App() -> Element {
    use_effect(|| {
        info!("App component rendered");
    });

    rsx! {
        document::Link { rel: "stylesheet", href: GLOBAL_CSS }
        Router::<Route> {}
    }
}
