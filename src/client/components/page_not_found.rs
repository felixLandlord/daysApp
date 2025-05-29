use dioxus::prelude::*;

const PAGE_NOT_FOUND_CSS: Asset = asset!("/assets/styles/page_not_found.css");

#[component]
pub fn PageNotFound(route: Vec<String>) -> Element {
    rsx! {
        document::Link {
            rel: "stylesheet",
            href: PAGE_NOT_FOUND_CSS,
        }

        div { class: "not-found-container",
            h1 { "Page not found" }
            p { "We are terribly sorry, but the page you requested doesn't exist." }
            pre { "log:\nattempted to navigate to: {route:?}" }
        }
    }
}
