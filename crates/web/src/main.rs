mod components;

use std::sync::Arc;

use yew::prelude::*;
use yew_router::prelude::*;

use self::components as c;

enum Msg {}

#[derive(Properties)]
struct Props {
    db: Arc<lib::database::Database<'static>>,
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

const INDEX: &[u8] = include_bytes!("../../../index.bin");
const DATABASE: &[u8] = include_bytes!("../../../database.bin");

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    let index = lib::database::IndexRef::from_bytes(INDEX.as_ref()).expect("broken index");
    let db = Arc::new(lib::database::Database::new(DATABASE.as_ref(), index));

    log::info!("Started up");

    yew::Renderer::<App>::with_props(Props { db }).render();
}
