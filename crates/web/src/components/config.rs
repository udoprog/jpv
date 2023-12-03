use lib::api;
use lib::config::IndexKind;
use yew::prelude::*;

use crate::error::Error;
use crate::fetch;

pub enum Msg {
    Config(lib::config::Config),
    Toggle(IndexKind),
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
            Msg::Toggle(index) => {
                if let Some(state) = self.state.as_mut() {
                    state.local.toggle(index);
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
                log::error!("Error: {}", error);
                self.pending = false;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut indexes = Vec::new();

        for &index in IndexKind::ALL {
            let onchange = ctx.link().callback(move |_| Msg::Toggle(index));

            let checked = match &self.state {
                Some(state) => state.local.is_enabled(index),
                None => false,
            };

            let class = classes! {
                "block",
                "row",
                "setting",
                checked.then_some("enabled"),
            };

            indexes.push(html! {
                <div {class}>
                    <input id={index.name()} type="checkbox" {checked} disabled={self.pending} {onchange} />
                    <label for={index.name()}>{index.description()}</label>
                </div>
            });
        }

        let config = html! {
            {for indexes}
        };

        let onsave = ctx.link().callback(|_| Msg::Save);
        let onrebuild = ctx.link().callback(|_| Msg::Rebuild);

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

        let disabled = self.pending || matches!(&self.state, Some(s) if s.local == s.remote);

        html! {
            <>
                <h5>{"Enabled sources"}</h5>

                <div class="block block-lg">{config}</div>

                <div class="block block-lg row row-spaced">
                    {back}
                    <button class="btn btn-lg end primary" {disabled} onclick={onsave}>{"Save"}</button>
                    <button class="btn btn-lg" onclick={onrebuild}>{"Rebuild database"}</button>
                </div>

                {log}
            </>
        }
    }
}
