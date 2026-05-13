use wasm_bindgen::prelude::*;
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
    let window = web_sys::window().expect("window will exist");

    let on_copy = {
        let window = window.clone();
        let copied = copied.clone();
        let content = content.clone();
        Callback::from(move |_| {
            let _ = window.navigator().clipboard().write_text(&content);

            copied.set(true);

            let window = window.clone();
            let copied = copied.clone();
            let closure = Closure::once(move || {
                copied.set(false);
            });
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                2000,
            );
            closure.forget();
        })
    };

    html! {
        <button onclick={on_copy} class="share-button">
            { if *copied { "Copied!" } else { label } }
        </button>
    }
}
