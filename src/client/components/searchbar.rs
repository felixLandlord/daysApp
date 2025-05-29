use dioxus::prelude::*;

const SEARCH_BAR_CSS: Asset = asset!("/assets/styles/searchbar.css");
const SEARCH_ICON: Asset = asset!("/assets/icons/search.svg");

#[component]
pub fn SearchBar(placeholder: String, #[props(into)] on_search: Callback<String>) -> Element {
    let mut search_text = use_signal(String::new);

    let handle_input = move |evt: FormEvent| {
        let value = evt.value().clone();
        search_text.set(value.clone());
        on_search(value);
    };

    rsx! {
        document::Link {
            rel: "stylesheet",
            href: SEARCH_BAR_CSS,
        }
        div { class: "search-bar",
            img {
                src: SEARCH_ICON,
                width: "20",
                height: "20",
            }
            input {
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{search_text.read()}",
                oninput: handle_input
            }
        }
    }
}
