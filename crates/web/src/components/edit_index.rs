use lib::config::{ConfigIndex, IndexFormat};
use phrasebook::yew023::PhrasesContext;
use phrasebook::Phrases;
use url::Url;
use yew::prelude::*;

pub(crate) enum Msg {
    ChangeId(String),
    ChangeFormat(IndexFormat),
    ChangeDescription(String),
    ChangeUrl(String),
    ChangeHelp(String),
    Save,
    Phrases(Phrases),
}

enum UrlError {
    Empty,
    Invalid(String),
}

#[derive(Default)]
struct Errors {
    id: bool,
    url: Option<UrlError>,
    help: Option<String>,
}

impl Errors {
    fn is_empty(&self) -> bool {
        !self.id && self.url.is_none()
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
    /// Kept as is, since the form doesn't edit it.
    gloss_url: Option<String>,
    help: String,
    errors: Errors,
    l10n: PhrasesContext,
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
            gloss_url: index.and_then(|i| i.gloss_url.clone()),
            help: index.and_then(|i| i.help.clone()).unwrap_or_default(),
            errors: Errors::default(),
            l10n: PhrasesContext::new(ctx.link(), ctx.link().callback(Msg::Phrases)),
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
                        gloss_url: self.gloss_url.clone(),
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
            Msg::Phrases(phrases) => {
                return self.l10n.set(phrases);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let pending = ctx.props().pending;
        let phrases = self.l10n.phrases();

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
            let error = self
                .errors
                .id
                .then(|| html!({ phrases.text("index-id-invalid") }));
            let hint = html!({ phrases.text("index-id-hint") });
            let describedby = sockel::field_describedby("index-id", true, error.is_some());

            html! {
                <sockel::Field id="index-id" label={html!({ phrases.text("index-id") })} {hint} {error}>
                    <sockel::TextInput id="index-id" value={self.id.clone()} on_input={on_id}
                        disabled={pending} invalid={self.errors.id} {describedby} />
                </sockel::Field>
            }
        });

        let delete = ctx.props().ondelete.as_ref().map(|ondelete| {
            html! {
                <sockel::Button variant={sockel::ButtonVariant::Danger} disabled={pending} onclick={ondelete.reform(|_| ())}>{phrases.text("index-delete")}</sockel::Button>
            }
        });

        let update = ctx.props().onupdate.as_ref().map(|onupdate| {
            let (variant, label) = if ctx.props().isupdate {
                (
                    sockel::ButtonVariant::Secondary,
                    phrases.text("index-no-update"),
                )
            } else {
                (sockel::ButtonVariant::Primary, phrases.text("index-update"))
            };

            html! {
                <sockel::Button {variant} disabled={pending} onclick={onupdate.reform(|_| ())}>{label}</sockel::Button>
            }
        });

        let url_error = self.errors.url.as_ref().map(|error| match error {
            UrlError::Empty => html!({ phrases.text("index-url-empty") }),
            UrlError::Invalid(error) => html!({ error }),
        });
        let help_error = self.errors.help.clone().map(|error| html!({ error }));
        let url_describedby = sockel::field_describedby("index-url", false, url_error.is_some());
        let help_describedby = sockel::field_describedby("index-help", false, help_error.is_some());

        html! {
            <sockel::Card inset={sockel::Inset::Compact}>
                <sockel::Stack>
                    <sockel::Field id="index-format" label={html!({ phrases.text("index-format") })}>
                        <sockel::Select<IndexFormat> id="index-format" options={format_options}
                            value={Some(self.format)} on_change={on_format} disabled={pending} />
                    </sockel::Field>
                    {id}
                    <sockel::Field id="index-url" label={html!({ phrases.text("index-url") })} error={url_error.clone()}>
                        <sockel::TextInput id="index-url" value={self.url.clone()} on_input={on_url}
                            disabled={pending} invalid={url_error.is_some()} describedby={url_describedby} />
                    </sockel::Field>
                    <sockel::Field id="index-description" label={html!({ phrases.text("index-description") })}>
                        <sockel::TextInput id="index-description" value={self.description.clone()} on_input={on_description}
                            disabled={pending} />
                    </sockel::Field>
                    <sockel::Field id="index-help" label={html!({ phrases.text("index-help") })} error={help_error.clone()}>
                        <sockel::TextInput id="index-help" value={self.help.clone()} on_input={on_help}
                            disabled={pending} invalid={help_error.is_some()} describedby={help_describedby} />
                    </sockel::Field>
                    <sockel::Cluster justify={sockel::Justify::Between}>
                        <sockel::Button disabled={pending} onclick={oncancel}>{phrases.text("cancel")}</sockel::Button>
                        <sockel::Cluster>
                            {delete}
                            {update}
                            <sockel::Button variant={sockel::ButtonVariant::Primary} disabled={pending} onclick={onsave}>{phrases.text("save")}</sockel::Button>
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
            self.errors.id = id.is_empty()
                || !id
                    .chars()
                    .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'));
        }

        if self.url.is_empty() {
            self.errors.url = Some(UrlError::Empty);
        } else if let Err(error) = Url::parse(&self.url) {
            self.errors.url = Some(UrlError::Invalid(error.to_string()));
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
