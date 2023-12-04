mod components;
mod error;
mod fetch;
mod query;
mod ws;

use yew::prelude::*;
use yew_router::prelude::*;

use self::components as c;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
enum Route {
    #[at("/")]
    Prompt,
    #[not_found]
    #[at("/404")]
    NotFound,
}

enum Msg {
    WebSocket(ws::Msg),
    Error(error::Error),
}

impl From<ws::Msg> for Msg {
    #[inline]
    fn from(msg: ws::Msg) -> Self {
        Msg::WebSocket(msg)
    }
}

impl From<error::Error> for Msg {
    #[inline]
    fn from(error: error::Error) -> Self {
        Msg::Error(error)
    }
}

struct App {
    ws: ws::Service<Self>,
    handle: ws::Handle,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (ws, handle) = ws::Service::new(ctx);

        let mut this = Self { ws, handle };

        if let Err(error) = this.ws.connect(ctx) {
            ctx.link().send_message(error);
        }

        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::WebSocket(msg) => {
                self.ws.update(ctx, msg);
                false
            }
            Msg::Error(error) => {
                log::error!("Failed to fetch: {error}");
                false
            }
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let ws = self.handle.clone();

        html! {
            <BrowserRouter>
                <Switch<Route> render={move |route| switch(route, &ws)} />
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route, ws: &ws::Handle) -> Html {
    match routes {
        Route::Prompt => html! {
            <c::Prompt ws={ws.clone()} />
        },
        Route::NotFound => {
            html! {
                <div id="content" class="container">{"There is nothing here"}</div>
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Started up");
    yew::Renderer::<App>::new().render();
    Ok(())
}
