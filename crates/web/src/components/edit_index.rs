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
    #[prop_or_default]
    pub(crate) onsavenew: Option<Callback<(String, ConfigIndex)>>,
    #[prop_or_default]
    pub(crate) onsave: Callback<ConfigIndex>,
    #[prop_or_default]
    pub(crate) ondelete: Option<Callback<()>>,
    #[prop_or_default]
    pub(crate) isupdate: bool,
    #[prop_or_default]
    pub(crate) onupdate: Option<Callback<()>>,
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
                self.validate(ctx);

                if self.errors.is_empty() {
                    let index = ConfigIndex {
                        enabled: true,
                        installing: false,
                        format: self.format,
                        description: Some(self.description.clone()),
                        url: self.url.clone(),
                        help: if self.help.is_empty() {
                            None
                        } else {
                            Some(self.help.clone())
                        },
                    };

                    if let Some(onsave) = &ctx.props().onsavenew {
                        onsave.emit((self.id.clone(), index));
                    } else {
                        ctx.props().onsave.emit(index);
                    }
                }
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let pending = ctx.props().pending;

        let format_options = IndexFormat::all()
            .into_iter()
            .map(|format| sockel::SelectOption::new(format, format.description()))
            .collect::<Vec<_>>();

        let on_format = ctx.link().callback(Msg::ChangeFormat);
        let on_id = ctx.link().callback(Msg::ChangeId);
        let on_url = ctx.link().callback(Msg::ChangeUrl);
        let on_description = ctx.link().callback(Msg::ChangeDescription);
        let on_help = ctx.link().callback(Msg::ChangeHelp);

        let oncancel = ctx.props().oncancel.reform(|_| ());
        let onsave = ctx.link().callback(move |_| Msg::Save);

        let id = ctx.props().onsavenew.is_some().then(|| {
            let error = self.errors.id.map(|error| html!({ error }));
            let hint = html!({ "The unique identifier of the dictionary, must only contain [a-z], [A-Z], and [0-9]." });
            let describedby = sockel::field_describedby("index-id", true, error.is_some());

            html! {
                <sockel::Field id="index-id" label={html!("Id")} {hint} {error}>
                    <sockel::TextInput id="index-id" value={self.id.clone()} on_input={on_id}
                        disabled={pending} invalid={self.errors.id.is_some()} {describedby} />
                </sockel::Field>
            }
        });

        let delete = ctx.props().ondelete.as_ref().map(|ondelete| {
            html! {
                <sockel::Button variant={sockel::ButtonVariant::Danger} disabled={pending} onclick={ondelete.reform(|_| ())}>{"Delete"}</sockel::Button>
            }
        });

        let update = ctx.props().onupdate.as_ref().map(|onupdate| {
            let (variant, label) = if ctx.props().isupdate {
                (sockel::ButtonVariant::Secondary, "Do not update")
            } else {
                (sockel::ButtonVariant::Primary, "Update")
            };

            html! {
                <sockel::Button {variant} disabled={pending} onclick={onupdate.reform(|_| ())}>{label}</sockel::Button>
            }
        });

        let url_error = self.errors.url.clone().map(|error| html!({ error }));
        let help_error = self.errors.help.clone().map(|error| html!({ error }));
        let url_describedby = sockel::field_describedby("index-url", false, url_error.is_some());
        let help_describedby = sockel::field_describedby("index-help", false, help_error.is_some());

        html! {
            <sockel::Card inset={sockel::Inset::Compact}>
                <sockel::Stack>
                    <sockel::Field id="index-format" label={html!("Format")}>
                        <sockel::Select<IndexFormat> id="index-format" options={format_options}
                            value={Some(self.format)} on_change={on_format} disabled={pending} />
                    </sockel::Field>
                    {id}
                    <sockel::Field id="index-url" label={html!("URL")} error={url_error.clone()}>
                        <sockel::TextInput id="index-url" value={self.url.clone()} on_input={on_url}
                            disabled={pending} invalid={url_error.is_some()} describedby={url_describedby} />
                    </sockel::Field>
                    <sockel::Field id="index-description" label={html!("Description")}>
                        <sockel::TextInput id="index-description" value={self.description.clone()} on_input={on_description}
                            disabled={pending} />
                    </sockel::Field>
                    <sockel::Field id="index-help" label={html!("Help Page")} error={help_error.clone()}>
                        <sockel::TextInput id="index-help" value={self.help.clone()} on_input={on_help}
                            disabled={pending} invalid={help_error.is_some()} describedby={help_describedby} />
                    </sockel::Field>
                    <sockel::Cluster justify={sockel::Justify::Between}>
                        <sockel::Button disabled={pending} onclick={oncancel}>{"Cancel"}</sockel::Button>
                        <sockel::Cluster>
                            {delete}
                            {update}
                            <sockel::Button variant={sockel::ButtonVariant::Primary} disabled={pending} onclick={onsave}>{"Save"}</sockel::Button>
                        </sockel::Cluster>
                    </sockel::Cluster>
                </sockel::Stack>
            </sockel::Card>
        }
    }
}

impl EditIndex {
    fn validate(&mut self, ctx: &Context<Self>) {
        if let Some(id) = ctx.props().onsavenew.is_some().then_some(&self.id) {
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
