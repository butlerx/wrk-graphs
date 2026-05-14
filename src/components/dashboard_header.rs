use crate::components::CopyButton;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub hash: String,
    pub description: Option<String>,
    pub endpoint: Option<String>,
    pub tags: Vec<String>,
    pub tests: usize,
    pub benchmarks: usize,
}

#[function_component(DashboardHeader)]
pub fn dashboard_header(props: &HeaderProps) -> Html {
    let HeaderProps {
        hash,
        description,
        endpoint,
        tags,
        tests,
        benchmarks,
    } = props;

    let Some(navigator) = use_navigator() else {
        return html! {};
    };

    let on_header_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&crate::Route::Home);
        })
    };

    let url = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .map_or_else(
            || format!("/dashboard#{hash}"),
            |origin| format!("{origin}/dashboard#{hash}"),
        );
    let embed_code = format!(
        "<iframe src=\"{url}\" width=\"100%\" height=\"600px\" frameborder=\"0\"></iframe>"
    );

    html! {
        <header class="dashboard-header">
            <div class="header-content">
                <div class="header-left" onclick={on_header_click} role="button" tabindex="0" aria-label="Go to home page">
                    <img src="./icon.png" alt="Benchmark Results logo" class="header-icon" />
                    <h1>{ "Benchmark Results" }</h1>
                </div>
                <div class="share-buttons">
                    <CopyButton content={url.clone()} label="Copy URL" />
                    <CopyButton content={embed_code} label="Copy Embed Code" />
                </div>
            </div>
            <div class="metadata">
                if let Some(description) = description {
                    <div class="metadata-row">
                        <span class="metadata-label">{ "Description:" }</span>
                        <span class="metadata-value">{ description }</span>
                    </div>
                }
                if let Some(endpoint) = endpoint {
                    <div class="metadata-row">
                        <span class="metadata-label">{ "Endpoint:" }</span>
                        <span class="metadata-value">{ endpoint }</span>
                    </div>
                }
                if !tags.is_empty() {
                    <div class="metadata-row">
                        <span class="metadata-label">{ "Tags:" }</span>
                        <div class="tag-list">
                            { for tags.iter().map(|tag| html! { <span class="tag">{tag}</span> }) }
                        </div>
                    </div>
                }
                if *tests > 0 {
                    <div class="metadata-row">
                        <span class="metadata-label">{ "Load Tests:" }</span>
                        <span class="metadata-value">{ tests }</span>
                    </div>
                }
                if *benchmarks > 0 {
                    <div class="metadata-row">
                        <span class="metadata-label">{ "Benchmarks:" }</span>
                        <span class="metadata-value">{ benchmarks }</span>
                    </div>
                }
            </div>
        </header>
    }
}
