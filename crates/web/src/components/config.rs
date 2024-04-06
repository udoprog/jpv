use std::collections::HashSet;

use lib::api;
use lib::config::ConfigIndex;
use yew::prelude::*;

use crate::c;
use crate::error::Error;
use crate::ws;

pub(crate) enum Msg {
    GetConfig(api::GetConfigResult),
    Toggle(String),
    ToggleOcr,
    IndexAdd,
    IndexAddSave(String, ConfigIndex),
    IndexAddCancel,
    IndexEdit(String),
    IndexCancel(String),
    IndexSave(String, ConfigIndex),
    IndexDelete(String),
    IndexUpdate(String),
    Save,
    Saved(Option<lib::config::Config>),
    InstallingAll,
    InstallAll,
    Error(Error),
}

#[derive(Properties, PartialEq)]
pub(crate) struct Props {
    /// Whether the component is embedded or not.
    #[prop_or_default]
    pub(crate) embed: bool,
    /// The current log state.
    #[prop_or_default]
    pub(crate) log: Vec<api::OwnedLogEntry>,
    ///  What to do when the back button has been pressed.
    pub(crate) onback: Callback<()>,
    pub(crate) ws: ws::Handle,
}

struct State {
    remote: lib::config::Config,
    local: lib::config::Config,
}

pub(crate) struct Config {
    pending: bool,
    state: Option<State>,
    installed: HashSet<String>,
    missing_ocr: Option<api::MissingOcr>,
    edit_index: HashSet<String>,
    update_indexes: HashSet<String>,
    index_add: bool,
    request: ws::Request,
}

impl Component for Config {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let request = ctx.props().ws.request(
            api::GetConfig,
            ctx.link().callback(|result| match result {
                Ok(config) => Msg::GetConfig(config),
                Err(error) => Msg::Error(error),
            }),
        );

        Self {
            pending: true,
            state: None,
            installed: HashSet::new(),
            missing_ocr: None,
            edit_index: HashSet::new(),
            update_indexes: HashSet::new(),
            index_add: false,
            request,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetConfig(result) => {
                self.state = Some(State {
                    remote: result.config.clone(),
                    local: result.config,
                });

                self.installed = result.installed;
                self.missing_ocr = result.missing_ocr;
                self.pending = false;
            }
            Msg::Toggle(id) => {
                if let Some(state) = self.state.as_mut() {
                    state.local.toggle(&id);
                }
            }
            Msg::ToggleOcr => {
                if let Some(state) = self.state.as_mut() {
                    state.local.ocr = !state.local.ocr;
                }
            }
            Msg::IndexAdd => {
                self.index_add = true;
            }
            Msg::IndexAddSave(id, index) => {
                if let Some(state) = &mut self.state {
                    state.local.indexes.insert(id, index);
                }

                self.index_add = false;
            }
            Msg::IndexAddCancel => {
                self.index_add = false;
            }
            Msg::IndexEdit(id) => {
                self.edit_index.insert(id);
            }
            Msg::IndexCancel(id) => {
                self.edit_index.remove(&id);
            }
            Msg::IndexSave(id, new_index) => {
                self.edit_index.remove(&id);

                if let Some(index) = self
                    .state
                    .as_mut()
                    .and_then(|s| s.local.indexes.get_mut(&id))
                {
                    *index = new_index;
                }
            }
            Msg::IndexDelete(id) => {
                self.edit_index.remove(&id);

                if let Some(state) = &mut self.state {
                    state.local.indexes.remove(&id);
                }
            }
            Msg::IndexUpdate(id) => {
                self.edit_index.remove(&id);

                if self.update_indexes.contains(&id) {
                    self.update_indexes.remove(&id);
                } else {
                    self.update_indexes.insert(id);
                }
            }
            Msg::Save => {
                if let Some(state) = &self.state {
                    let config = (state.local != state.remote).then(|| state.local.clone());
                    let update_indexes = self.update_indexes.iter().cloned().collect();

                    self.pending = true;

                    self.request = ctx.props().ws.request(
                        api::UpdateConfigRequest {
                            config,
                            update_indexes,
                        },
                        ctx.link().callback(|result| match result {
                            Ok(api::UpdateConfigResponse { config }) => Msg::Saved(config),
                            Err(error) => Msg::Error(error),
                        }),
                    );
                }
            }
            Msg::Saved(config) => {
                if let Some(config) = config {
                    if let Some(state) = &mut self.state {
                        state.remote = config.clone();
                        state.local = config;
                    }
                }

                self.update_indexes.clear();
                self.edit_index.clear();
                self.pending = false;
            }
            Msg::InstallAll => {
                self.pending = true;

                self.request = ctx.props().ws.request(
                    api::InstallAllRequest,
                    ctx.link().callback(|result| match result {
                        Ok(api::Empty) => Msg::InstallingAll,
                        Err(error) => Msg::Error(error),
                    }),
                );
            }
            Msg::InstallingAll => {
                self.pending = false;
            }
            Msg::Error(error) => {
                log::error!("{}", error);
                self.pending = false;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut indexes = Vec::new();
        let mut ocr = None;

        if let Some(state) = &self.state {
            for (id, index) in &state.local.indexes {
                let is_updated =
                    self.update_indexes.contains(id) || Some(index) != state.remote.indexes.get(id);

                let checked = match &self.state {
                    Some(state) => state.local.is_enabled(id),
                    None => false,
                };

                let installing = match &self.state {
                    Some(state) => state.local.is_installing(id),
                    None => false,
                };

                if self.edit_index.contains(id) {
                    let oncancel = ctx.link().callback({
                        let id = id.to_owned();
                        move |_| Msg::IndexCancel(id.clone())
                    });

                    let onsave = ctx.link().callback({
                        let id = id.to_owned();
                        move |index| Msg::IndexSave(id.clone(), index)
                    });

                    let ondelete = ctx.link().callback({
                        let id = id.to_owned();
                        move |()| Msg::IndexDelete(id.clone())
                    });

                    let onupdate = ctx.link().callback({
                        let id = id.to_owned();
                        move |()| Msg::IndexUpdate(id.clone())
                    });

                    let isupdate = self.update_indexes.contains(id);
                    indexes.push(html!(<c::EditIndex index={index.clone()} pending={self.pending} {oncancel} {onsave} {ondelete} {onupdate} {isupdate} />));
                } else {
                    let onchange = ctx.link().callback({
                        let id = id.to_owned();
                        move |_| Msg::Toggle(id.clone())
                    });

                    let class = classes! {
                        "block",
                        "row",
                        "row-spaced",
                        "index",
                        checked.then_some("enabled"),
                    };

                    let onclick = ctx.link().callback({
                        let id = id.to_owned();
                        move |_| Msg::IndexEdit(id.clone())
                    });

                    let help = match &index.help {
                        Some(help) => html! {
                            <button class="btn index-url" title={"Go to the help page for this dictionary"} href={help.clone()} target="_index">{"About"}</button>
                        },
                        None => {
                            html!(<button disabled={true} class="btn" title="No help page specified">{"About"}</button>)
                        }
                    };

                    let not_installed = (!self.installed.contains(id)).then(|| {
                        html! {
                            <span class="bullet bullet-danger">{"not installed"}</span>
                        }
                    });

                    let updated = is_updated.then(|| {
                        html! {
                            <span title="Has been updated and will be applied on Save">{"＊"}</span>
                        }
                    });

                    indexes.push(html! {
                        <div {class}>
                            <input id={id.to_owned()} type="checkbox" {checked} disabled={self.pending || installing} {onchange} />
                            <label for={id.to_owned()}>{id.to_owned()}</label>
                            <label for={id.to_owned()}>{index.description.clone()}</label>
                            {for updated}
                            {not_installed}
                            <button class="btn btn-primary row-end index-edit" {onclick} title={"Change this dictionary"}>{"Edit"}</button>
                            {help}
                        </div>
                    });
                }
            }

            ocr = Some({
                let checked = state.local.ocr;

                let onchange = ctx.link().callback(move |_| Msg::ToggleOcr);

                let missing_ocr = self.missing_ocr.as_ref().filter(|_| state.remote.ocr).map(|missing| {
                    let install_url = missing
                        .install_url
                        .as_ref()
                        .map(|install| {
                            let href = install.url.clone();
                            let title = install.title.clone();

                            html! {
                                <div class="block block-sm row row-spaced">
                                    <a {href} {title} class="btn btn-lg" target="_install_url">{format!("⇓ {}", install.text)}</a>
                                </div>
                            }
                        });

                    html! {
                        <div class="block block-lg block-danger">
                            <div class="block block-sm row row-spaced">
                                <span class="title">{"OCR support is not installed"}</span>
                            </div>

                            {for install_url}
                        </div>
                    }
                });

                html! {
                    <>
                        <div class="block row row-spaced">
                            <input id="ocr" type="checkbox" {checked} disabled={self.pending} {onchange} />
                            <label for="ocr">{"OCR Support"}</label>
                        </div>

                        {for missing_ocr}
                    </>
                }
            });
        }

        let add = if self.index_add {
            let oncancel = ctx.link().callback(move |_| Msg::IndexAddCancel);

            let onsave = ctx
                .link()
                .callback(move |(id, index)| Msg::IndexAddSave(id, index));

            html! {
                <c::EditIndex pending={self.pending} {oncancel} onsavenew={onsave} />
            }
        } else {
            let onclick = ctx.link().callback(|_| Msg::IndexAdd);

            let onrebuild = ctx.link().callback(|_| Msg::InstallAll);

            html! {
                <div class="block row row-spaced">
                    <button class="row-end btn btn-primary" disabled={self.pending} {onclick}>{"New dictionary"}</button>
                    <button class="btn btn-primary" disabled={self.pending} onclick={onrebuild} title="Install all missing dictionaries">{"Install all"}</button>
                </div>
            }
        };

        let dictionaries = html! {
            <>
                {for indexes}
                {add}
            </>
        };

        let onsave = ctx.link().callback(|_| Msg::Save);

        let back = (!ctx.props().embed).then(|| {
            html! {
                <button class="btn btn-lg" onclick={ctx.props().onback.reform(|_| ())}>{"Back"}</button>
            }
        });

        let log = (!ctx.props().log.is_empty()).then(|| {
            let it = ctx.props().log.iter().rev().map(|entry| {
                let class = classes! {
                    "row",
                    "log-entry",
                    format!("log-entry-{}", entry.level),
                };

                html! {
                    <div {class}>
                        <span class="log-field log-level">{&entry.level}</span>
                        <span class="log-field log-target">{&entry.target}</span>
                        <span class="log-field log-text">{&entry.text}</span>
                    </div>
                }
            });

            html! {
                <div class="block block-lg log">{for it}</div>
            }
        });

        let disabled = self.pending
            || matches!(&self.state, Some(s) if s.local == s.remote)
                && self.update_indexes.is_empty();

        let pending = self.pending.then(|| {
            html! {
                <div class="block block-lg row row-spaced">
                    <div class="spinner">{"Loading"}</div>
                </div>
            }
        });

        html! {
            <>
                <div class="block block-lg row row-spaced">
                    {back}
                    <button class="row-end btn btn-lg btn-primary" {disabled} onclick={onsave}>{"Save"}</button>
                </div>

                {pending}

                <h5>{"Dictionaries"}</h5>
                <div class="block block-lg">{dictionaries}</div>

                <h5>{"OCR"}</h5>

                <div class="block block-lg">
                    {for ocr}
                </div>

                <h5>{"Log"}</h5>
                {log}
            </>
        }
    }
}
