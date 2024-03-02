use lib::kanjidic2::OwnedCharacter;
use yew::prelude::*;

use super::{colon, comma, romaji, ruby, seq};

const ONYOMI: lib::Furigana<'static, 1, 1> = lib::Furigana::new("音読み", "おんよみ", "");
const KUNYOMI: lib::Furigana<'static, 1, 1> = lib::Furigana::new("訓読み", "くんよみ", "");

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
            c.readings.iter().filter(|r| r.ty == "ja_on"),
            |r, not_last| {
                let sep = not_last.then(comma);
                html!(<><span>{r.text.clone()}</span>{for sep}</>)
            },
        )
        .peekable();

        let onyomi = onyomi
            .peek()
            .is_some()
            .then(move || html!(<div class="readings row row-bottom"><span class="highlight clickable" title={romaji(ONYOMI)}>{ruby(ONYOMI)}</span>{colon()}{for onyomi}</div>));

        let mut kunyomi = seq(
            c.readings.iter().filter(|r| r.ty == "ja_kun"),
            |r, not_last| {
                let sep = not_last.then(comma);
                html!(<><span>{r.text.clone()}</span>{for sep}</>)
            },
        )
        .peekable();

        let kunyomi = kunyomi
            .peek()
            .is_some()
            .then(move || html!(<div class="readings row row-bottom"><span class="highlight clickable" title={romaji(ONYOMI)}>{ruby(KUNYOMI)}</span>{colon()}{for kunyomi}</div>));

        let mut meanings = seq(
            c.meanings.iter().filter(|r| r.lang.is_none()),
            |r, not_last| {
                let sep = not_last.then(comma);
                html!(<>{r.text.clone()}{for sep}</>)
            },
        )
        .peekable();

        let meanings = meanings
            .peek()
            .is_some()
            .then(move || html!(<div class="readings row">{for meanings}</div>));

        html! {
            <div class="character">
                <div class="literal text highlight"><a href={format!("/api/kanji/{}", c.literal)} target="_api">{c.literal.clone()}</a></div>
                {for meanings}
                {for onyomi}
                {for kunyomi}
            </div>
        }
    }
}
