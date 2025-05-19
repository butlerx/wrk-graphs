use crate::{
    components::{ShareModal, WrkConfig},
    pages::dashboard::Loadtest,
    parser, Route,
};
use base64::prelude::*;
use yew::prelude::*;
use yew_router::prelude::*;

fn generate_hash(data: &str, description: String, tags: Vec<String>) -> String {
    let metrics = parser::WrkMetrics::from(data);
    let data_obj = Loadtest {
        metrics,
        description,
        tags,
    };

    let data_str = serde_json5::to_string(&data_obj).unwrap_or_default();
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
                let hash = generate_hash(&data, description, tags);
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
        Callback::from(move |()| {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hash() {
        // Test data
        let data = r"Running 10s test @ http://localhost:3000
  2 threads and 10 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     1.23ms    0.45ms   5.67ms   85.71%
    Req/Sec     4.12k   456.78     5.00k    75.00%
  82345 requests in 10.00s, 12.34MB read
Requests/sec:   8234.50
Transfer/sec:      1.23MB";
        let description = "Test load test".to_string();
        let tags = vec!["test".to_string(), "example".to_string()];

        // Generate hash
        let hash = generate_hash(data, description, tags);

        // Verify the hash is not empty and is base64 URL-safe
        assert!(!hash.is_empty());
        assert!(!hash.contains('+'));
        assert!(!hash.contains('/'));
        assert!(!hash.contains('='));

        // Verify the hash can be decoded back to valid JSON
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(&hash).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        let json: Loadtest = serde_json5::from_str(&decoded_str).unwrap();

        // Verify the decoded JSON contains our original data
        assert!(json.description.as_str() == "Test load test");
        assert!(json.tags.len() == 2);
    }
}
