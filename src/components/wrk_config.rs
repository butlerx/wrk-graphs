use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct WrkConfigProps {
    pub on_command_change: Callback<String>,
}

#[function_component(WrkConfig)]
pub fn wrk_config(props: &WrkConfigProps) -> Html {
    let url = use_state(|| String::from("http://localhost:3000"));
    let threads = use_state(|| 4);
    let connections = use_state(|| 100);
    let duration = use_state(|| 30);
    let latency = use_state(|| true);
    let timeout = use_state(|| 2);
    let script = use_state(String::new);
    let copied = use_state(|| false);

    let command = {
        let url = url.clone();
        let threads = threads.clone();
        let connections = connections.clone();
        let duration = duration.clone();
        let latency = latency.clone();
        let timeout = timeout.clone();
        let script = script.clone();

        format!(
            "wrk -t{} -c{} -d{}s {}{}{}{}",
            *threads,
            *connections,
            *duration,
            if *latency { "--latency " } else { "" },
            if script.is_empty() {
                String::new()
            } else {
                format!("-s {} ", *script)
            },
            if *timeout > 0 {
                format!("--timeout {}s ", *timeout)
            } else {
                String::new()
            },
            *url
        )
    };

    {
        let on_command_change = props.on_command_change.clone();
        use_effect_with(command.clone(), move |command| {
            on_command_change.emit(command.clone());
            || ()
        });
    }

    let on_url_change = {
        let url = url.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            url.set(input.value());
        })
    };

    let on_threads_change = {
        let threads = threads.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<u32>() {
                threads.set(value);
            }
        })
    };

    let on_connections_change = {
        let connections = connections.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<u32>() {
                connections.set(value);
            }
        })
    };

    let on_duration_change = {
        let duration = duration.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<u32>() {
                duration.set(value);
            }
        })
    };

    let on_latency_change = {
        let latency = latency.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            latency.set(input.checked());
        })
    };

    let on_timeout_change = {
        let timeout = timeout.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<u32>() {
                timeout.set(value);
            }
        })
    };

    let on_script_change = {
        let script = script.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            script.set(input.value());
        })
    };

    html! {
        <div class="wrk-config">
            <h2>{ "WRK Configuration" }</h2>
            <div class="config-grid">
                <div class="config-row">
                    <label for="url">{ "Target URL:" }</label>
                    <input type="text" id="url" value={(*url).clone()} onchange={on_url_change} />
                </div>
                <div class="config-row">
                    <label for="threads">{ "Threads (-t):" }</label>
                    <input
                        type="number"
                        id="threads"
                        min="1"
                        max="64"
                        value={(*threads).to_string()}
                        onchange={on_threads_change}
                    />
                    <span class="help-text">{ "Number of threads to use" }</span>
                </div>
                <div class="config-row">
                    <label for="connections">{ "Connections (-c):" }</label>
                    <input
                        type="number"
                        id="connections"
                        min="1"
                        max="10000"
                        value={(*connections).to_string()}
                        onchange={on_connections_change}
                    />
                    <span class="help-text">{ "Number of HTTP connections to keep open" }</span>
                </div>
                <div class="config-row">
                    <label for="duration">{ "Duration (-d):" }</label>
                    <input
                        type="number"
                        id="duration"
                        min="1"
                        max="3600"
                        value={(*duration).to_string()}
                        onchange={on_duration_change}
                    />
                    <span class="help-text">{ "Duration of test in seconds" }</span>
                </div>
                <div class="config-row">
                    <label for="latency">{ "Latency (--latency):" }</label>
                    <input
                        type="checkbox"
                        id="latency"
                        checked={*latency}
                        onchange={on_latency_change}
                    />
                    <span class="help-text">{ "Print latency statistics" }</span>
                </div>
                <div class="config-row">
                    <label for="timeout">{ "Timeout (--timeout):" }</label>
                    <input
                        type="number"
                        id="timeout"
                        min="0"
                        max="300"
                        value={(*timeout).to_string()}
                        onchange={on_timeout_change}
                    />
                    <span class="help-text">
                        { "Socket/request timeout in seconds (0 for no timeout)" }
                    </span>
                </div>
                <div class="config-row">
                    <label for="script">{ "Lua Script (-s):" }</label>
                    <input
                        type="text"
                        id="script"
                        value={(*script).clone()}
                        onchange={on_script_change}
                        placeholder="Path to Lua script (optional)"
                    />
                    <span class="help-text">{ "Path to Lua script for custom requests" }</span>
                </div>
            </div>
            <div class="command-preview">
                <h3>{ "Generated Command:" }</h3>
                <pre>{ command.clone() }</pre>
                <button
                    class={classes!("copy-button", if *copied { "copied" } else { "" })}
                    onclick={let command = command.clone();
                    let copied = copied.clone();
                    Callback::from(move |_| {
                        let window = web_sys::window().unwrap();
                        let navigator = window.navigator();
                        let clipboard = navigator.clipboard();
                        let command = command.clone();
                        let copied = copied.clone();
                        spawn_local(async move {
                            let _ = clipboard.write_text(&command);
                            copied.set(true);
                            // Reset the copied state after 2 seconds
                            let window = web_sys::window().unwrap();
                            let closure = Closure::once(move || {
                                copied.set(false);
                            });
                            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                closure.as_ref().unchecked_ref(),
                                2000,
                            );
                            closure.forget(); // Prevent the closure from being dropped
                        });
                    })}
                >
                    { if *copied { "Copied!" } else { "Copy to Clipboard" } }
                </button>
            </div>
        </div>
    }
}
