use yew::prelude::*;

use crate::class;
use crate::regions;

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    /// The region's name, which its stored size is kept under.
    pub(crate) name: AttrValue,
    pub(crate) list: Html,
    #[prop_or_default]
    pub(crate) detail: Option<Html>,
    #[prop_or_default]
    pub(crate) empty: Html,
    #[prop_or(0.65)]
    pub(crate) default: f64,
    #[prop_or_default]
    pub(crate) label: Option<AttrValue>,
}

/// A resizable list and detail whose size is remembered by name.
#[function_component]
pub(crate) fn SplitRegion(props: &Props) -> Html {
    let at = use_state_eq({
        let name = props.name.clone();
        let default = props.default;
        move || regions::load(&name, default)
    });

    let on_settle = {
        let at = at.clone();
        let name = props.name.clone();

        Callback::from(move |value: f64| {
            regions::store(&name, value);
            at.set(value);
        })
    };

    html! {
        <sockel::SplitView class={class::FILL_REST} at={*at} {on_settle} label={props.label.clone()}
            list={props.list.clone()} detail={props.detail.clone()} empty={props.empty.clone()} />
    }
}
