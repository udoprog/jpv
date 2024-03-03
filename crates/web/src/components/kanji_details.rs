use std::rc::Rc;

use lib::api;
use yew::prelude::*;

use crate::c;
use crate::error::Error;
use crate::ws;

use super::{comma, seq, spacing};

pub(crate) enum Msg {
    GetKanji(Box<api::OwnedKanjiResponse>),
    Error(Error),
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    /// Whether the component is embedded or not.
    #[prop_or_default]
    pub(crate) embed: bool,
    /// The current log state.
    pub(crate) kanji: Rc<str>,
    ///  What to do when the back button has been pressed.
    pub(crate) onback: Callback<()>,
    pub(crate) ws: ws::Handle,
    pub(crate) onclick: Callback<String>,
}

pub(crate) struct KanjiDetails {
    pending: bool,
    request: ws::Request,
    kanji: Option<api::OwnedKanjiResponse>,
}

impl Component for KanjiDetails {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let request = ctx.props().ws.request(
            api::GetKanji {
                kanji: ctx.props().kanji.to_string(),
            },
            ctx.link().callback(|result| match result {
                Ok(response) => Msg::GetKanji(Box::new(response)),
                Err(error) => Msg::Error(error),
            }),
        );

        Self {
            pending: false,
            request,
            kanji: None,
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetKanji(kanji) => {
                self.pending = false;
                self.kanji = Some(*kanji);
            }
            Msg::Error(error) => {
                log::error!("{}", error);
                self.pending = false;
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if old_props.kanji == ctx.props().kanji {
            return false;
        }

        self.pending = true;

        self.request = ctx.props().ws.request(
            api::GetKanji {
                kanji: ctx.props().kanji.to_string(),
            },
            ctx.link().callback(|result| match result {
                Ok(response) => Msg::GetKanji(Box::new(response)),
                Err(error) => Msg::Error(error),
            }),
        );

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let heading = (!ctx.props().embed).then(|| {
            let pending = self.pending.then(|| {
                html! {
                    {"Loading..."}
                }
            });

            html! {
                <div class="block block-lg row row-spaced">
                    <button class="btn btn-lg" onclick={ctx.props().onback.reform(|_| ())}>{"Back"}</button>
                    {for pending}
                </div>
            }
        });

        let kanji = self.kanji.as_ref().map(|kanji| {
            let radicals = (!kanji.radicals.is_empty()).then(|| {
                let radicals = seq(&kanji.radicals, |literal, not_last| {
                    let onclick = ctx.props().onclick.reform({
                        let literal = literal.clone();
                        move |_| literal.clone()
                    });
                    html! {<><span class="text highlight"><a onclick={onclick.clone()}>{literal.clone()}</a></span>{not_last.then(comma)}</>}
                });

                html! {
                    <div class="block block-lg row">
                        <span class="highlight clickable">{"Radicals:"}{spacing()}</span>

                        {for radicals}
                    </div>
                }
            });

            let strokes = (!kanji.kanji.misc.stroke_counts.is_empty()).then(|| {
                let strokes = seq(&kanji.kanji.misc.stroke_counts, |strokes, not_last| {
                    html! {<><span class="text highlight">{strokes}</span>{not_last.then(comma)}</>}
                });

                html! {
                    <div class="block block-lg row">
                        <span class="highlight clickable">{"Strokes:"}{spacing()}</span>

                        {for strokes}
                    </div>
                }
            });

            html! {
                <>
                    <div class="block block-lg character">
                        <c::Character embed={false} character={kanji.kanji.clone()} />
                        {for strokes}
                        {for radicals}
                    </div>
                </>
            }
        });

        html! {
            <>
                {for heading}
                {for kanji}
            </>
        }
    }
}
