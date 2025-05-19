#![warn(clippy::pedantic)]
use pages::{DashboardPage, HomePage, NotFoundPage};
use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
pub mod parser;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/dashboard/:hash")]
    Dashboard { hash: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::Dashboard { hash } => html! { <DashboardPage hash={hash} /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <main>
                <Switch<Route> render={switch} />
            </main>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
