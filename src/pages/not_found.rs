use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    html! {
        <div class="container">
            <div class="error-container">
                <h1>{ "404 - Page Not Found" }</h1>
                <div class="error-content">
                    <p class="error-message">
                        { "The page you're looking for doesn't exist or the load test data could not be found." }
                    </p>
                    <div class="error-actions">
                        <Link<Route> to={Route::Home} classes="home-link">
                            <button class="primary-button">
                                { "Return to Home" }
                            </button>
                        </Link<Route>>
                    </div>
                </div>
            </div>
        </div>
    }
}
