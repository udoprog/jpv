use std::rc::Rc;

use yew::prelude::*;

use super::spacing;

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub(crate) query: String,
    pub(crate) analyzed: Rc<[String]>,
    pub(crate) index: usize,
    #[prop_or_default]
    pub(crate) analyze_at: Option<usize>,
    pub(crate) on_analyze: Callback<usize>,
    pub(crate) on_analyze_cycle: Callback<()>,
}

pub(crate) struct AnalyzeToggle;

impl Component for AnalyzeToggle {
    type Message = ();
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                (rem > 0).then_some("active"),
                (!(event.is_none() && ctx.props().analyzed.len() <= 1)).then_some("clickable"),
                "analyze-span"
            };

            rem = rem.saturating_sub(1);
            html!(<span {class} {onclick}>{c}</span>)
        });

        let analyze_hint = if ctx.props().analyzed.len() > 1 {
            Some(html! {
                <div class="block row hint">
                    {format!("{} / {} (click character to cycle)", ctx.props().index + 1, ctx.props().analyzed.len())}
                </div>
            })
        } else if ctx.props().analyzed.is_empty() {
            Some(html! {
                <div class="block row hint">
                    <span>{"Hint:"}</span>
                    {spacing()}
                    <span>{"Click character for substring search"}</span>
                </div>
            })
        } else {
            None
        };

        html! {
            <div id="analyze">
                <div class="block row analyze-text">{for query}</div>
                {analyze_hint}
            </div>
        }
    }
}
