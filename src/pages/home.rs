use crate::{
    components::{ShareModal, WrkConfig},
    serialzer::encode_dashboard,
    Route,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let navigator = use_navigator().unwrap();
    let show_modal = use_state(|| false);
    let command = use_state(String::new);

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

    let on_share = {
        let navigator = navigator.clone();
        Callback::from(
            move |(data, description, tags): (String, String, Vec<String>)| {
                let hash = encode_dashboard(&data, description, tags);
                navigator.push(&Route::Dashboard);
                web_sys::window()
                    .unwrap()
                    .location()
                    .set_hash(&hash)
                    .expect("Failed to set hash");
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
                    <div class="header-left" onclick={on_header_click}>
                        <img src="./icon.png" alt="Logo" class="header-icon" />
                        <h1>{ "Load Test Generator" }</h1>
                    </div>
                </div>
            </header>
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
