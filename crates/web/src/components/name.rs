use lib::{jmnedict, kana};
use yew::prelude::*;

use super::{comma, ruby, seq};

pub enum Msg {}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub embed: bool,
    pub entry: jmnedict::OwnedEntry,
}

pub struct Name;

impl Component for Name {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let entry = &ctx.props().entry;

        let entries = if entry.kanji.is_empty() {
            let it = entry.reading.iter().map(|reading| {
                html! {
                    <div class="block">
                        <span class="text kanji highlight">{&reading.text}</span>
                    </div>
                }
            });

            html!({for it})
        } else {
            let it = entry.reading.iter().flat_map(|reading| {
                entry.kanji.iter().map(|kanji| {
                    let furigana = kana::Full::new(kanji, &reading.text, "").furigana();

                    html! {
                        <span class="text kanji highlight">{ruby(furigana)}</span>
                    }
                })
            });

            let it = seq(
                it,
                |it, not_last| html!(<>{it}{for not_last.then(comma)}</>),
            );

            html!({for it})
        };

        let bullets = bullets!(entry.name_types, "sm");

        html! {
            <span class="row">
                {entries}
                {for bullets}
            </span>
        }
    }
}
