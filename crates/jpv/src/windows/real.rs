use std::future::Future;
use std::pin::{pin, Pin};

use anyhow::Result;
use async_fuse::Fuse;
use tokio::sync::futures::Notified;

use crate::open_uri;
use crate::system::{Setup, Start};
use crate::VERSION;

const ICON: &[u8] = include_bytes!("../../res/jpv22.ico");
const NAME: &str = "se.tedro.JapaneseDictionary";

/// Setup system integration.
pub(crate) fn setup() -> Result<Setup> {
    let Some(mutex) = winctx::NamedMutex::create_acquired(NAME)? else {
        return Ok(Setup::Busy);
    };

    Ok(Setup::Start(Some(Box::new(Windows { _mutex: mutex }))))
}

struct Windows {
    _mutex: winctx::NamedMutex,
}

impl Start for Windows {
    fn start<'a>(
        &'a mut self,
        port: u16,
        shutdown: Notified<'a>,
        _: &'a crate::system::SystemEvents,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let mut shutdown = pin!(Fuse::new(shutdown));
            let mut builder = winctx::WindowBuilder::new("jpv");

            builder.set_icon(ICON, 22, 22);
            builder.add_menu_entry(format_args!("jpv ({VERSION})"), true);
            let open = builder.add_menu_entry("Open dictionary...", false);
            let exit = builder.add_menu_entry("Shutdown...", false);

            let (sender, mut event_loop) = builder.with_class_name(NAME).build().await?;

            loop {
                tokio::select! {
                    _ = shutdown.as_mut() => {
                        sender.shutdown();
                    },
                    event = event_loop.tick() => {
                        match event? {
                            winctx::Event::MenuEntryClicked(token) => {
                                if token == open {
                                    let address = format!("http://localhost:{port}");
                                    open_uri::open(&address);
                                }

                                if token == exit {
                                    sender.shutdown();
                                }
                            },
                            winctx::Event::NotificationClicked(..) => {

                            },
                            winctx::Event::NotificationDismissed(..) => {

                            },
                            winctx::Event::Shutdown => {
                                break;
                            },
                            _ => {}
                        }
                    },
                }
            }

            Ok(())
        })
    }
}
