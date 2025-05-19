use crate::{
    components::{ShareModal, WrkConfig},
    parser, Route,
};
use base64::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

fn generate_hash(data: &str, description: &str, tags: Vec<String>) -> String {
    let test_data = parser::WrkMetrics::from(data);
    let data_obj = serde_json::json!({
        "test_data": test_data,
        "description": description,
        "tags": tags
    });

    let data_str = serde_json::to_string(&data_obj).unwrap_or_default();
    BASE64_URL_SAFE_NO_PAD.encode(data_str)
}

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let navigator = use_navigator().unwrap();
    let show_modal = use_state(|| false);
    let command = use_state(String::new);

    let on_command_change = {
        let command = command.clone();
        Callback::from(move |new_command: String| {
            command.set(new_command);
        })
    };

    let on_share = {
        let navigator = navigator.clone();
        Callback::from(
            move |(data, description, tags): (String, String, Vec<String>)| {
                let hash = generate_hash(&data, &description, tags);
                navigator.push(&Route::Dashboard { hash });
            },
        )
    };

    let on_show_modal = {
        let show_modal = show_modal.clone();
        Callback::from(move |_| {
            show_modal.set(true);
        })
    };

    let on_close_modal = {
        let show_modal = show_modal.clone();
        Callback::from(move |_| {
            show_modal.set(false);
        })
    };

    html! {
        <div class="container">
            <h1>{ "WRK Load Test Generator" }</h1>
            <div class="main-content">
                <WrkConfig on_command_change={on_command_change} />
                <div class="share-section">
                    <button class="share-button" onclick={on_show_modal}>
                        { "Share Your Load Test" }
                    </button>
                </div>
            </div>
            if *show_modal {
                <ShareModal on_close={on_close_modal} on_share={on_share} />
            }
        </div>
    }
}
