mod components;
mod error;
mod fetch;
mod ws;

use std::sync::Arc;

use anyhow::Context as _;
use yew::prelude::*;
use yew_router::prelude::*;

use self::components as c;

enum Msg {}

#[derive(Properties)]
struct Props {
    db: Arc<Option<lib::database::Database>>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.db, &other.db)
    }
}

struct App;

impl Component for App {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <c::Prompt db={ctx.props().db.clone()} />
            </BrowserRouter>
        }
    }
}

fn load_database() -> anyhow::Result<Option<lib::database::Database>> {
    Ok(None)
}

fn main() -> anyhow::Result<()> {
    wasm_logger::init(wasm_logger::Config::default());
    let db = Arc::new(load_database().context("loading database")?);
    log::info!("Started up");
    yew::Renderer::<App>::with_props(Props { db }).render();
    Ok(())
}
