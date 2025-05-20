#![warn(clippy::pedantic)]
use pages::{DashboardPage, HomePage, NotFoundPage};
use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod parser;
mod serialzer;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/dashboard")]
    Dashboard,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <main>
                <Switch<Route> render={|routes| match routes {
                    Route::Home => html! { <HomePage /> },
                    Route::Dashboard => html! { <DashboardPage /> },
                    Route::NotFound => html! { <NotFoundPage /> },
                }} />
            </main>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
