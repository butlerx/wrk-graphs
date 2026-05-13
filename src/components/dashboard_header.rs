use crate::components::CopyButton;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub hash: String,
    pub description: Option<String>,
    pub endpoint: Option<String>,
    pub tags: Vec<String>,
    pub runs: usize,
}

#[function_component(DashboardHeader)]
pub fn dashboard_header(props: &HeaderProps) -> Html {
    let HeaderProps {
        hash,
        description,
        endpoint,
        tags,
        runs,
    } = props;

    let navigator = use_navigator().unwrap();

    let on_header_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&crate::Route::Home);
        })
    };

    let window = web_sys::window().expect("window will exist");
    let url = format!("{}/dashboard#{}", window.location().origin().unwrap(), hash);
    let embed_code = format!(
        "<iframe src=\"{url}\" width=\"100%\" height=\"600px\" frameborder=\"0\"></iframe>"
    );

    html! {
        <header class="dashboard-header">
            <div class="header-content">
                <div class="header-left" onclick={on_header_click}>
                    <img src="./icon.png" alt="Logo" class="header-icon" />
                    <h1>{ "Load Test Results" }</h1>
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
                <div class="metadata-row">
                    <span class="metadata-label">{ "Number of Runs:" }</span>
                    <span class="metadata-value">{ runs }</span>
                </div>
            </div>
        </header>
    }
}
