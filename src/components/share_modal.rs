use gloo_file::{callbacks::read_as_text, File};
use std::collections::HashMap;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct ShareModalProps {
    pub on_close: Callback<()>,
    pub on_share: Callback<(String, String, Vec<String>)>,
}

pub enum Msg {
    Files(Vec<File>),
    LoadedText(String, String),
    DescriptionChanged(String),
    TagsChanged(String),
    Submit,
}

pub struct ShareModal {
    readers: HashMap<String, gloo_file::callbacks::FileReader>,
    files_content: Vec<String>,
    description: String,
    tags: String,
}

impl Component for ShareModal {
    type Message = Msg;
    type Properties = ShareModalProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::new(),
            files_content: Vec::new(),
            description: String::new(),
            tags: String::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Files(files) => {
                for file in files {
                    let file_name = file.name();
                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        read_as_text(&file, move |res| {
                            link.send_message(Msg::LoadedText(
                                file_name.clone(),
                                res.expect("Failed to read file"),
                            ));
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
            Msg::LoadedText(file_name, content) => {
                self.files_content.push(content);
                self.readers.remove(&file_name);
                true
            }
            Msg::DescriptionChanged(value) => {
                self.description = value;
                true
            }
            Msg::TagsChanged(value) => {
                self.tags = value;
                true
            }
            Msg::Submit => {
                if let Some(content) = self.files_content.first() {
                    let tags = self
                        .tags
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>();
                    ctx.props()
                        .on_share
                        .emit((content.clone(), self.description.clone(), tags));
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_close = ctx.props().on_close.clone();
        let on_file_change = ctx.link().callback(move |e: Event| {
            let mut selected_files = Vec::new();
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                let files = js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from);
                selected_files.extend(files);
            }
            Msg::Files(selected_files)
        });

        let on_description_change = ctx.link().callback(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::DescriptionChanged(input.value())
        });

        let on_tags_change = ctx.link().callback(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::TagsChanged(input.value())
        });

        let on_submit = ctx.link().callback(|_| Msg::Submit);

        html! {
            <div class="modal-overlay">
                <div class="modal-content">
                    <div class="modal-header">
                        <h2>{ "Share Load Test Results" }</h2>
                        <button
                            class="close-button"
                            onclick={let on_close = on_close.clone(); Callback::from(move |_| on_close.emit(()))}
                        >
                            { "×" }
                        </button>
                    </div>
                    <div class="modal-body">
                        <div class="form-group">
                            <label for="test-data">
                                { "Paste WRK test results or upload a file:" }
                            </label>
                            <textarea
                                id="test-data"
                                rows="10"
                                value={self.files_content.first().unwrap_or(&String::new()).clone()}
                                onchange={&on_file_change}
                            />
                            <input type="file" onchange={on_file_change} multiple=false />
                        </div>
                        <div class="form-group">
                            <label for="description">{ "Description:" }</label>
                            <input
                                type="text"
                                id="description"
                                value={self.description.clone()}
                                onchange={on_description_change}
                            />
                        </div>
                        <div class="form-group">
                            <label for="tags">{ "Tags (comma separated):" }</label>
                            <input
                                type="text"
                                id="tags"
                                value={self.tags.clone()}
                                onchange={on_tags_change}
                            />
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button class="submit-button" onclick={on_submit}>{ "Share" }</button>
                    </div>
                </div>
            </div>
        }
    }
}
