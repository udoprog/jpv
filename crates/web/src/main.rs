mod callbacks;
mod components;
mod error;
mod fetch;
mod query;
mod ws;

use callbacks::Callbacks;
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
    ClientEvent(lib::api::ClientEvent),
    Error(error::Error),
}

impl From<ws::Msg> for Msg {
    #[inline]
    fn from(msg: ws::Msg) -> Self {
        Msg::WebSocket(msg)
    }
}

impl From<lib::api::ClientEvent> for Msg {
    #[inline]
    fn from(msg: lib::api::ClientEvent) -> Self {
        Msg::ClientEvent(msg)
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
    callbacks: Callbacks,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            ws: ws::Service::new(),
            callbacks: Callbacks::default(),
        };

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
            Msg::ClientEvent(event) => {
                self.callbacks.emit_client_event(event);
                true
            }
            Msg::Error(error) => {
                log::error!("Failed to fetch: {error}");
                false
            }
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let callbacks = self.callbacks.clone();

        html! {
            <BrowserRouter>
                <Switch<Route> render={move |route| switch(route, &callbacks)} />
            </BrowserRouter>
        }
    }
}

fn switch(routes: Route, callbacks: &Callbacks) -> Html {
    match routes {
        Route::Prompt => html! {
            <c::Prompt callbacks={callbacks.clone()} />
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
    log::trace!("Started up");
    yew::Renderer::<App>::new().render();
    Ok(())
}
