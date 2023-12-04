use std::collections::HashSet;

use lib::api;
use lib::config::ConfigIndex;
use yew::prelude::*;

use crate::error::Error;
use crate::{c, fetch};

pub enum Msg {
    Config(lib::config::Config),
    Toggle(String),
    IndexAdd,
    IndexAddSave(String, ConfigIndex),
    IndexAddCancel,
    IndexEdit(String),
    IndexCancel(String),
    IndexSave(String, ConfigIndex),
    Save,
    Saved,
    Rebuild,
    Rebuilt,
    Error(Error),
}

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Whether the component is embedded or not.
    #[prop_or_default]
    pub embed: bool,
    /// The current log state.
    #[prop_or_default]
    pub log: Vec<api::LogEntry>,
    ///  What to do when the back button has been pressed.
    pub onback: Callback<()>,
}

struct State {
    remote: lib::config::Config,
    local: lib::config::Config,
}

pub struct Config {
    pending: bool,
    state: Option<State>,
    edit_index: HashSet<String>,
    index_add: bool,
}

impl Component for Config {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async move {
            match fetch::config().await {
                Ok(entries) => Msg::Config(entries),
                Err(error) => Msg::Error(error),
            }
        });

        Self {
            pending: true,
            state: None,
            edit_index: HashSet::new(),
            index_add: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Config(config) => {
                self.state = Some(State {
                    remote: config.clone(),
                    local: config,
                });

                self.pending = false;
            }
            Msg::Toggle(id) => {
                if let Some(state) = self.state.as_mut() {
                    state.local.toggle(&id);
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
            Msg::Save => {
                if let Some(state) = &self.state {
                    let local = state.local.clone();
                    self.pending = true;

                    ctx.link().send_future(async move {
                        match fetch::update_config(local).await {
                            Ok(api::Empty) => Msg::Saved,
                            Err(error) => Msg::Error(error),
                        }
                    });
                }
            }
            Msg::Saved => {
                if let Some(state) = &mut self.state {
                    state.remote = state.local.clone();
                }

                self.pending = false;
            }
            Msg::Rebuild => {
                ctx.link().send_future(async move {
                    match fetch::rebuild().await {
                        Ok(api::Empty) => Msg::Rebuilt,
                        Err(error) => Msg::Error(error),
                    }
                });
            }
            Msg::Rebuilt => {
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

        if let Some(state) = &self.state {
            for (id, index) in &state.local.indexes {
                let checked = match &self.state {
                    Some(state) => state.local.is_enabled(id),
                    None => false,
                };

                if self.edit_index.contains(id) {
                    let oncancel = ctx.link().callback({
                        let id = id.to_owned();
                        move |_| Msg::IndexCancel(id.clone())
                    });

                    let onsave = ctx.link().callback({
                        let id = id.to_owned();
                        move |(_, index)| Msg::IndexSave(id.clone(), index)
                    });

                    indexes.push(html!(<c::EditIndex index={index.clone()} pending={self.pending} {oncancel} {onsave} />));
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

                    let help = index.help.as_ref().map(|help| html! {
                        <a class="index-url" title={"Go to the help page for this dictionary"} href={help.clone()} target="_index">{"Help"}</a>
                    });

                    indexes.push(html! {
                        <div {class}>
                            <input id={id.to_owned()} type="checkbox" {checked} disabled={self.pending} {onchange} />
                            <label for={id.to_owned()} class="index-id">{id.to_owned()}</label>
                            <label for={id.to_owned()}>{index.description.clone()}</label>
                            <div class="end index-edit clickable" {onclick} title={"Change this dictionary"}>{"Edit"}</div>
                            {help}
                        </div>
                    });
                }
            }
        }

        let add = if self.index_add {
            let oncancel = ctx.link().callback({ move |_| Msg::IndexAddCancel });

            let onsave = ctx
                .link()
                .batch_callback({ move |(id, index)| Some(Msg::IndexAddSave(id?, index)) });

            html! {
                <c::EditIndex pending={self.pending} {oncancel} {onsave} />
            }
        } else {
            let onclick = ctx.link().callback(|_| Msg::IndexAdd);

            html! {
                <div class="block">
                    <button class="btn primary centered" disabled={self.pending} {onclick}>{"+ Add dictionary"}</button>
                </div>
            }
        };

        let config = html! {
            <>
                {for indexes}
                {add}
            </>
        };

        let onsave = ctx.link().callback(|_| Msg::Save);
        let onrebuild = ctx.link().callback(|_| Msg::Rebuild);

        let back = (!ctx.props().embed).then(|| {
            html! {
                <button class="btn" onclick={ctx.props().onback.reform(|_| ())}>{"Back"}</button>
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

        let disabled = self.pending || matches!(&self.state, Some(s) if s.local == s.remote);

        html! {
            <>
                <h5>{"Dictionaries"}</h5>

                <div class="block block-lg">{config}</div>

                <div class="block block-lg row row-spaced">
                    {back}
                    <button class="btn end primary" {disabled} onclick={onsave}>{"Save"}</button>
                    <button class="btn" onclick={onrebuild} title="Build the search index used by this dictionary">{"Rebuild"}</button>
                </div>

                {log}
            </>
        }
    }
}
