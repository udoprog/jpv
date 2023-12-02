use lib::api;
use lib::config::IndexKind;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::error::Error;
use crate::fetch;

pub enum Msg {
    Back,
    Config(lib::config::Config),
    Toggle(IndexKind),
    Save,
    Saved,
    Error(Error),
}

#[derive(Properties, PartialEq)]
pub struct Props;

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
            Msg::Back => {
                if let Some(navigator) = ctx.link().navigator() {
                    navigator.back();
                }
            }
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
            Msg::Error(error) => {
                log::error!("Error: {}", error);
                self.pending = false;
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let disabled = self.pending
            || match &self.state {
                Some(state) => state.local == state.remote,
                None => false,
            };

        let mut indexes = Vec::new();

        for &index in IndexKind::ALL {
            let onchange = ctx.link().callback(move |e: Event| {
                e.prevent_default();
                Msg::Toggle(index)
            });

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
                <>
                    <div {class}>
                        <input id={index.name()} type="checkbox" {checked} disabled={self.pending} {onchange} />
                        <label for={index.name()}>{index.description()}</label>
                    </div>
                </>
            });
        }

        let config = html! {
            {for indexes}
        };

        let onback = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Back
        });

        let onsave = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::Save
        });

        html! {
            <div id="container">
                <h5>{"Enabled sources"}</h5>

                <div class="block block-lg">
                    {config}
                </div>

                <div class="block block-lg row row-spaced">
                <button class="btn btn-lg" onclick={onback}>{"Back"}</button>
                    <button class="btn btn-lg end primary" {disabled} onclick={onsave}>{"Save"}</button>
                    <button class="btn btn-lg">{"Rebuild database"}</button>
                </div>
            </div>
        }
    }
}
