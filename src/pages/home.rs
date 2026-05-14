use crate::{
    components::{ShareModal, WrkConfig},
    serializer::encode_dashboard,
    Route,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let Some(navigator) = use_navigator() else {
        return html! {};
    };
    let show_modal = use_state(|| false);
    let command = use_state(String::new);
    let error_msg = use_state(|| Option::<String>::None);

    let on_header_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    let on_command_change = {
        let command = command.clone();
        Callback::from(move |new_command: String| {
            command.set(new_command);
        })
    };

    let on_share =
        {
            let navigator = navigator.clone();
            let error_msg = error_msg.clone();
            Callback::from(
                move |(data, description, tags): (String, String, Vec<String>)| {
                    match encode_dashboard(&data, description, tags) {
                        Ok(hash) => {
                            error_msg.set(None);
                            navigator.push(&Route::Dashboard);
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_hash(&hash);
                            }
                        }
                        Err(e) => {
                            error_msg.set(Some(e.to_string()));
                        }
                    }
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
        Callback::from(move |()| {
            show_modal.set(false);
        })
    };

    html! {
        <div class="container">
            <header class="dashboard-header">
                <div class="header-content">
                    <div class="header-left" onclick={on_header_click} role="button" tabindex="0" aria-label="Go to home page">
                        <img src="./icon.png" alt="Load Test Generator logo" class="header-icon" />
                        <h1>{ "Load Test Generator" }</h1>
                    </div>
                </div>
            </header>
            <div class="main-content">
                <WrkConfig on_command_change={on_command_change} />
                if let Some(ref err) = *error_msg {
                    <div class="error-message">{ err }</div>
                }
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
