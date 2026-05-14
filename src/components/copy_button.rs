use gloo::timers::callback::Timeout;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CopyButtonProps {
    pub content: String,
    pub label: String,
}

#[function_component(CopyButton)]
pub fn copy_button(props: &CopyButtonProps) -> Html {
    let CopyButtonProps { content, label } = props;
    let copied = use_state(|| false);
    let timeout_handle = use_mut_ref(|| None::<Timeout>);

    let on_copy = {
        let copied = copied.clone();
        let content = content.clone();
        let timeout_handle = timeout_handle.clone();
        Callback::from(move |_| {
            let Some(window) = web_sys::window() else {
                return;
            };
            let _ = window.navigator().clipboard().write_text(&content);

            copied.set(true);

            let copied = copied.clone();
            let handle = Timeout::new(2_000, move || {
                copied.set(false);
            });
            *timeout_handle.borrow_mut() = Some(handle);
        })
    };

    html! {
        <button onclick={on_copy} class="share-button" aria-label={format!("Copy {label} to clipboard")}>
            { if *copied { "Copied!" } else { label } }
        </button>
    }
}
