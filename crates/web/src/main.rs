mod components;
mod error;
mod l10n;
mod query;
mod ws;

use phrasebook::yew023::use_phrases;
use phrasebook::{browser, Lang, Phrases};
use yew::prelude::*;
use yew_router::prelude::*;

use self::components as c;

mod class {
    include!(concat!(env!("OUT_DIR"), "/classes.rs"));
}

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

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
        Route::NotFound => html!(<NotFound />),
    }
}

#[function_component]
fn NotFound() -> Html {
    let phrases = use_phrases();

    html! {
        <div class={classes!(class::CONTAINER, class::CONTENT)}>{phrases.text("not-found")}</div>
    }
}

#[function_component]
fn Root() -> Html {
    let phrases = use_state(|| {
        let phrases = browser::initial(&l10n::phrasebook());
        browser::set_document_lang(phrases.lang());
        phrases
    });

    let set_lang = use_memo((), {
        let phrases = phrases.clone();

        move |_| {
            l10n::SetLang(Callback::from(move |lang: Lang| {
                let new = phrases.with_lang(lang);
                browser::set_document_lang(lang);
                new.store();
                phrases.set(new);
            }))
        }
    });

    html! {
        <ContextProvider<Phrases> context={(*phrases).clone()}>
            <ContextProvider<l10n::SetLang> context={(*set_lang).clone()}>
                <sockel::OverlayHost>
                    <sockel::Styles css={STYLESHEET} />
                    <sockel::DataTips />
                    <App />
                </sockel::OverlayHost>
            </ContextProvider<l10n::SetLang>>
        </ContextProvider<Phrases>>
    }
}

fn main() -> anyhow::Result<()> {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Started up");
    c::theme::init();
    yew::Renderer::<Root>::new().render();
    Ok(())
}
