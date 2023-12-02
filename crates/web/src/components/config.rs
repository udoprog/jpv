use lib::config::IndexKind;
use yew::prelude::*;

use crate::error::Error;
use crate::fetch;

use super::spacing;

pub enum Msg {
    Config(lib::config::Config),
    Error(Error),
}

#[derive(Properties, PartialEq)]
pub struct Props;

pub struct Config {
    config: Option<lib::config::Config>,
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

        Self { config: None }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Config(config) => {
                self.config = Some(config);
            }
            Msg::Error(error) => {
                log::error!("{}", error);
            }
        }

        true
    }

    fn view(&self, _: &Context<Self>) -> Html {
        let config = self.config.as_ref().map(|c| {
            let mut indexes = Vec::new();

            for index in IndexKind::ALL {
                indexes.push(html! {
                    <>
                        <div class="block row">
                            <input id={index.name()} type="checkbox" checked={c.is_enabled(index.name())} />
                            <label for={index.name()}>{index.description()}</label>
                        </div>
                    </>
                });
            }

            html! {
                {for indexes}
            }
        });

        html! {
            <div id="container">
                <div class="block">
                    <h5>{"Enabled sources"}</h5>
                    {config}
                </div>

                <div class="block row">
                    <button class="btn">{"Save"}</button>
                    {spacing()}
                    <button class="btn">{"Rebuild database"}</button>
                </div>
            </div>
        }
    }
}
