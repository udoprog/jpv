use lib::kanjidic2::OwnedCharacter;
use yew::prelude::*;

use super::{colon, comma, seq};

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub embed: bool,
    pub character: OwnedCharacter,
}

pub(crate) struct Character;

impl Component for Character {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let c = &ctx.props().character;

        let mut onyomi = seq(
            c.reading_meaning
                .readings
                .iter()
                .filter(|r| r.ty == "ja_on"),
            |r, not_last| {
                let sep = not_last.then(comma);
                html!(<>{r.text.clone()}{for sep}</>)
            },
        )
        .peekable();

        let onyomi = onyomi
            .peek()
            .is_some()
            .then(move || html!(<div class="readings row">{"On"}{colon()}{for onyomi}</div>));

        let mut kunyomi = seq(
            c.reading_meaning
                .readings
                .iter()
                .filter(|r| r.ty == "ja_kun"),
            |r, not_last| {
                let sep = not_last.then(comma);
                html!(<>{r.text.clone()}{for sep}</>)
            },
        )
        .peekable();

        let kunyomi = kunyomi
            .peek()
            .is_some()
            .then(move || html!(<div class="readings row">{"Kun"}{colon()}{for kunyomi}</div>));

        let meanings = seq(
            c.reading_meaning
                .meanings
                .iter()
                .filter(|r| r.lang.is_none()),
            |r, _| html!(<li>{r.text.clone()}</li>),
        );

        html! {
            <div class="character">
                <div class="literal text highlight"><a href={format!("/api/kanji/{}", c.literal)} target="_api">{c.literal.clone()}</a></div>
                {for onyomi}
                {for kunyomi}
                <div class="meanings row">{"Meanings"}{colon()}<ul>{for meanings}</ul></div>
            </div>
        }
    }
}
