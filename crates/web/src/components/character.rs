use lib::kanjidic2::OwnedCharacter;
use yew::prelude::*;

use super::{colon, comma, romaji, row, ruby, seq, spacing};

use crate::class;

const ONYOMI: lib::Furigana<'static> = lib::Furigana::new("音読み", "おんよみ", "");
const KUNYOMI: lib::Furigana<'static> = lib::Furigana::new("訓読み", "くんよみ", "");

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub embed: bool,
    pub character: OwnedCharacter,
    ///  What to do when the back button has been pressed.
    #[prop_or_default]
    pub(crate) onclick: Callback<()>,
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
            .then(move || row(html!(<><span data-tip={romaji(ONYOMI)}>{ruby(ONYOMI)}</span>{colon()}{spacing()}{for onyomi}</>)));

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
            .then(move || row(html!(<><span data-tip={romaji(KUNYOMI)}>{ruby(KUNYOMI)}</span>{colon()}{spacing()}{for kunyomi}</>)));

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
            .then(move || row(html!({for meanings})));

        let onclick = ctx.props().onclick.reform(|_| ());

        html! {
            <sockel::Stack gap={sockel::Gap::Tight}>
                <div class={classes!(class::LITERAL, class::TEXT)}><a {onclick}>{c.literal.clone()}</a></div>
                {for meanings}
                {for onyomi}
                {for kunyomi}
            </sockel::Stack>
        }
    }
}
