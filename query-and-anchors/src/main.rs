use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(Nav)]
    #[route("/")]
    Home {},
    #[route("/search?:q")]
    SearchQuery { q: String },
    #[route("/search")]
    SearchBlank {},
    #[route("/anchors")]
    Anchors {},
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Nav() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        div {
            "Home"
        }

        div {
            Link {
                to: Route::SearchBlank {},
                "Search Blank"
            }
        }

        div {
            Link {
                to: Route::SearchQuery { q: "t ext".into() },
                "Search Query"
            }
        }

        div {
            Link {
                to: Route::Anchors {},
                "Anchors Empty"
            }
        }

        div {
            Link {
                to: "/anchors#h6".to_owned(),
                // to: Route::Anchors {}, // how to add a hash string to the link??
                "Anchors H6"
            }
        }
    }
}

#[component]
fn SearchBlank() -> Element {
    rsx! {
        SearchOption {
        }
    }
}

#[component]
fn SearchQuery(q: String) -> Element {
    rsx! {
        SearchOption {
            q,
        }
    }
}

#[component]
fn SearchOption(q: Option<String>) -> Element {
    let navigator = navigator();
    let q = q.unwrap_or_default();

    rsx! {
        div {
            Link {
                to: Route::Home {},
                "Home"
            }
        }

        div {
            "Search:"
            input {
                value: "{q}",
                onchange: move |evt| {
                    let q = evt.value();
                    let _ = if q.is_empty() {
                        navigator.push(Route::SearchBlank {})
                    } else {
                        navigator.push(Route::SearchQuery { q })
                    };
                },
            }
        }
    }
}

#[component]
fn Anchors() -> Element {
    rsx! {
        div {
            Link {
                to: Route::Home {},
                "Home"
            }
        }

        div {
            Link {
                to: "#h6",
                "H6"
            }
        }

        div {
            "Anchors:"
        }

        div {
            h1 {
                id: "h1",
                "Header 1"
            }

            p {
                "1 Lorem ipsum"
            }
        }

        div {
            h1 {
                id: "h2",
                "Header 2"
            }

            p {
                "2 Lorem ipsum"
            }
        }

        div {
            h1 {
                id: "h3",
                "Header 3"
            }

            p {
                "3 Lorem ipsum"
            }
        }

        div {
            h1 {
                id: "h4",
                "Header 4"
            }

            p {
                "4 Lorem ipsum"
            }
        }

        div {
            h1 {
                id: "h5",
                "Header 5"
            }

            p {
                "5 Lorem ipsum"
            }
        }

        div {
            h1 {
                id: "h6",
                "Header 6"
            }

            p {
                "6 Lorem ipsum"
            }
        }
    }
}
