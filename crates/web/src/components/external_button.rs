use web_sys::window;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    pub(crate) href: Option<AttrValue>,
    pub(crate) target: AttrValue,
    #[prop_or_default]
    pub(crate) title: Option<AttrValue>,
    #[prop_or_default]
    pub(crate) variant: sockel::ButtonVariant,
    #[prop_or_default]
    pub(crate) class: Classes,
    #[prop_or_default]
    pub(crate) children: Html,
}

/// A button opening `href` in the named window, so the app stays open; with
/// no `href` it is disabled.
#[function_component]
pub(crate) fn ExternalButton(props: &Props) -> Html {
    let onclick = Callback::from({
        let href = props.href.clone();
        let target = props.target.clone();

        move |_: MouseEvent| {
            let (Some(href), Some(window)) = (&href, window()) else {
                return;
            };

            if let Err(error) = window.open_with_url_and_target(href, &target) {
                log::error!("Failed to open {href}: {error:?}");
            }
        }
    });

    html! {
        <sockel::Button variant={props.variant} class={props.class.clone()} title={props.title.clone()}
            disabled={props.href.is_none()} {onclick}>
            {props.children.clone()}
        </sockel::Button>
    }
}
