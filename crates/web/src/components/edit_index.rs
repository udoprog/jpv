use lib::config::{ConfigIndex, IndexFormat};
use url::Url;
use yew::prelude::*;

pub(crate) enum Msg {
    ChangeId(String),
    ChangeFormat(IndexFormat),
    ChangeDescription(String),
    ChangeUrl(String),
    ChangeHelp(String),
    Save,
}

#[derive(Default)]
struct Errors {
    id: Option<&'static str>,
    url: Option<String>,
    help: Option<String>,
}

impl Errors {
    fn is_empty(&self) -> bool {
        self.id.is_none() && self.url.is_none()
    }
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    #[prop_or_default]
    pub(crate) index: Option<ConfigIndex>,
    #[prop_or_default]
    pub(crate) pending: bool,
    pub(crate) oncancel: Callback<()>,
    pub(crate) onsave: Callback<(Option<String>, ConfigIndex)>,
    #[prop_or_default]
    pub(crate) ondelete: Callback<()>,
}

pub(crate) struct EditIndex {
    id: String,
    format: IndexFormat,
    description: String,
    url: String,
    help: String,
    errors: Errors,
}

impl Component for EditIndex {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let index = ctx.props().index.as_ref();

        Self {
            id: String::new(),
            format: index.map(|i| i.format).unwrap_or_default(),
            description: index
                .and_then(|i| i.description.clone())
                .unwrap_or_default(),
            url: index.map(|i| i.url.clone()).unwrap_or_default(),
            help: index.and_then(|i| i.help.clone()).unwrap_or_default(),
            errors: Errors::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeId(id) => {
                self.id = id;
                self.validate(ctx);
            }
            Msg::ChangeFormat(format) => {
                self.format = format;
            }
            Msg::ChangeDescription(description) => {
                self.description = description;
            }
            Msg::ChangeUrl(url) => {
                self.url = url;
                self.validate(ctx);
            }
            Msg::ChangeHelp(help) => {
                self.help = help;
                self.validate(ctx);
            }
            Msg::Save => {
                let id = ctx.props().index.is_none().then(|| self.id.clone());
                self.validate(ctx);

                if self.errors.is_empty() {
                    let index = ConfigIndex {
                        enabled: true,
                        format: self.format,
                        description: Some(self.description.clone()),
                        url: self.url.clone(),
                        help: if self.help.is_empty() {
                            None
                        } else {
                            Some(self.help.clone())
                        },
                    };

                    ctx.props().onsave.emit((id, index));
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchangeformat = ctx.link().batch_callback({
            move |e: Event| {
                let select = e.target_dyn_into::<web_sys::HtmlSelectElement>()?;
                let format = select.value().parse().ok()?;
                Some(Msg::ChangeFormat(format))
            }
        });

        let onchangedescription = ctx.link().batch_callback({
            move |e: Event| {
                let input = e.target_dyn_into::<web_sys::HtmlInputElement>()?;
                let description = input.value();
                Some(Msg::ChangeDescription(description))
            }
        });

        let onchangeurl = ctx.link().batch_callback({
            move |e: Event| {
                let input = e.target_dyn_into::<web_sys::HtmlInputElement>()?;
                let url = input.value();
                Some(Msg::ChangeUrl(url))
            }
        });

        let onchangehelp = ctx.link().batch_callback({
            move |e: Event| {
                let input = e.target_dyn_into::<web_sys::HtmlInputElement>()?;
                let url = input.value();
                Some(Msg::ChangeHelp(url))
            }
        });

        let oncancel = ctx.props().oncancel.reform(|_| ());

        let onsave = ctx.link().callback(move |_| Msg::Save);

        let class = classes! {
            "block",
            "index"
        };

        let id = ctx.props().index.is_none().then(|| {
            let oninput = ctx.link().batch_callback({
                move |e: InputEvent| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>()?;
                    let description = input.value();
                    Some(Msg::ChangeId(description))
                }
            });

            let error = self.errors.id.map(|error| {
                html!(<p class="form-error">{error}</p>)
            });

            let class = classes! {
                "block",
                "form",
                self.errors.id.is_some().then_some("has-errors")
            };

            html! {
                <div {class}>
                    <h6>{"Id"}</h6>
                    <p class="form-help">{"The unique identifier of the dictionary, must only contain [a-z], [A-Z], and [0-9]."}</p>
                    <input type="text" disabled={ctx.props().pending} value={self.id.clone()} {oninput} />
                    <>{error}</>
                </div>
            }
        });

        let delete = ctx.props().index.is_some().then(|| {
            let ondelete = ctx.props().ondelete.reform(|_| ());

            html! {
                <button class="btn end danger" disabled={ctx.props().pending} onclick={ondelete}>{"Delete"}</button>
            }
        });

        let save_classes = classes! {
            "btn",
            "primary",
            delete.is_none().then_some("end"),
        };

        let url_class = classes! {
            "block",
            "form",
            self.errors.url.is_some().then_some("has-errors")
        };

        let url_error = self
            .errors
            .url
            .as_ref()
            .map(|error| html!(<p class="form-error">{error.clone()}</p>));

        let help_class = classes! {
            "block",
            "form",
            self.errors.help.is_some().then_some("has-errors")
        };

        let help_error = self
            .errors
            .help
            .as_ref()
            .map(|error| html!(<p class="form-error">{error.clone()}</p>));

        let options = IndexFormat::all().into_iter().map(|format| {
            html! {
                <option value={format.id()} selected={self.format == format}>{format.description()}</option>
            }
        });

        html! {
            <div {class}>
                <div class="block form">
                    <h6>{"Format"}</h6>
                    <select onchange={onchangeformat}>
                        {for options}
                    </select>
                </div>
                {id}
                <div class={url_class}>
                    <h6>{"URL"}</h6>
                    <input type="text" disabled={ctx.props().pending} value={self.url.clone()} onchange={onchangeurl} />
                    <>{url_error}</>
                </div>
                <div class="block form">
                    <h6>{"Description"}</h6>
                    <input type="text" disabled={ctx.props().pending} value={self.description.clone()} onchange={onchangedescription} />
                </div>
                <div class={help_class}>
                    <h6>{"Help Page"}</h6>
                    <input type="text" disabled={ctx.props().pending} value={self.help.clone()} onchange={onchangehelp} />
                    <>{help_error}</>
                </div>
                <div class="block row row-spaced">
                    <button class="btn" disabled={ctx.props().pending} onclick={oncancel}>{"Cancel"}</button>
                    {delete}
                    <button class={save_classes} disabled={ctx.props().pending} onclick={onsave}>{"Save"}</button>
                </div>
            </div>
        }
    }
}

impl EditIndex {
    fn validate(&mut self, ctx: &Context<Self>) {
        if let Some(id) = ctx.props().index.is_none().then_some(&self.id) {
            if id.is_empty()
                || !id
                    .chars()
                    .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
            {
                self.errors.id = Some("Must be a non-empty sequence of [a-z], [A-Z], and [0-9].");
            } else {
                self.errors.id = None;
            }
        }

        if self.url.is_empty() {
            self.errors.url = Some("Must be non-empty".to_string());
        } else if let Err(error) = Url::parse(&self.url) {
            self.errors.url = Some(error.to_string());
        } else {
            self.errors.url = None;
        }

        if self.help.is_empty() {
            self.errors.help = None;
        } else if let Err(error) = Url::parse(&self.help) {
            self.errors.help = Some(error.to_string());
        } else {
            self.errors.help = None;
        }
    }
}
