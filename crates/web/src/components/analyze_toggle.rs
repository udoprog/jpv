use std::rc::Rc;

use phrasebook::yew023::PhrasesContext;
use phrasebook::{args, Phrases};
use yew::prelude::*;

use super::spacing;

use crate::class;

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub(crate) query: String,
    pub(crate) analyzed: Rc<[String]>,
    pub(crate) index: usize,
    #[prop_or_default]
    pub(crate) analyze_at: Option<usize>,
    #[prop_or_default]
    pub(crate) compact: bool,
    pub(crate) on_analyze: Callback<usize>,
    pub(crate) on_analyze_cycle: Callback<()>,
}

pub(crate) struct AnalyzeToggle {
    l10n: PhrasesContext,
}

impl Component for AnalyzeToggle {
    type Message = Phrases;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            l10n: PhrasesContext::new(ctx.link(), ctx.link().callback(|phrases| phrases)),
        }
    }

    fn update(&mut self, _: &Context<Self>, phrases: Self::Message) -> bool {
        self.l10n.set(phrases)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let phrases = self.l10n.phrases();
        let mut rem = 0usize;

        let string = ctx.props().analyzed.get(ctx.props().index);

        let query = ctx.props().query.char_indices().map(|(i, c)| {
            let sub = ctx.props().query.get(i..).unwrap_or_default();

            let event = if let (Some(analyze_at), Some(string)) = (ctx.props().analyze_at, string) {
                if i == analyze_at && rem == 0 && sub.starts_with(string.as_str()) {
                    rem = string.chars().count();
                    None
                } else {
                    Some(i)
                }
            } else {
                Some(i)
            };

            let onclick = match event {
                Some(i) => ctx.props().on_analyze.reform(move |_| i),
                None => ctx.props().on_analyze_cycle.reform(|_| ()),
            };

            let class = classes! {
                (rem > 0).then_some(class::ACTIVE),
                (!(event.is_none() && ctx.props().analyzed.len() <= 1)).then_some(class::CLICKABLE),
                class::ANALYZE_SPAN
            };

            rem = rem.saturating_sub(1);
            html!(<span {class} {onclick}>{c}</span>)
        });

        let analyze_hint = if ctx.props().analyzed.len() > 1 {
            Some(html! {
                <div class={class::HINT}>
                    {phrases.msg("analyze-cycle", &args!(index = ctx.props().index + 1, count = ctx.props().analyzed.len()))}
                </div>
            })
        } else if ctx.props().analyzed.is_empty() {
            Some(html! {
                <div class={class::HINT}>
                    <span>{phrases.text("analyze-hint")}</span>
                    {spacing()}
                    <span>{phrases.text("analyze-hint-text")}</span>
                </div>
            })
        } else {
            None
        };

        html! {
            <sockel::Stack gap={sockel::Gap::Snug}>
                <sockel::Cluster gap={sockel::Gap::None} class={classes!(class::ANALYZE_TEXT, ctx.props().compact.then_some(class::COMPACT))}>{for query}</sockel::Cluster>
                {analyze_hint}
            </sockel::Stack>
        }
    }
}
