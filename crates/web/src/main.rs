mod components;
mod error;
mod fetch;
mod ws;

use yew::prelude::*;
use yew_router::prelude::*;

use self::components as c;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Prompt,
    #[at("/config")]
    Config,
    #[not_found]
    #[at("/404")]
    NotFound,
}

enum Msg {}

struct App;

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Prompt => html! {
            <c::Prompt />
        },
        Route::Config => html! {
            <c::Config />
        },
        Route::NotFound => {
            html! {
                <div id="container">{"There is nothing here"}</div>
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Started up");
    yew::Renderer::<App>::new().render();
    Ok(())
}
