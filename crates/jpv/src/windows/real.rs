use std::future::Future;
use std::io::Cursor;
use std::pin::{pin, Pin};

use anyhow::Result;
use async_fuse::Fuse;
use tokio::sync::futures::Notified;
use winctx::event::{ClipboardEvent, Event, MouseButton};

use crate::open_uri;
use crate::system::{self, Setup, Start, SystemEvents};
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
        system_events: &'a SystemEvents,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let mut shutdown = pin!(Fuse::new(shutdown));
            let mut window = winctx::CreateWindow::new(NAME).clipboard_events(true);

            let icon = window.icons().insert_buffer(ICON, 22, 22);

            let area = window.new_area().icon(icon);
            let menu = area.popup_menu();

            let open = menu
                .push_entry(format_args!("Japanese Dictionary ({VERSION})"))
                .id();
            let exit = menu.push_entry("Quit").id();

            let (sender, mut event_loop) = window.build().await?;

            loop {
                tokio::select! {
                    _ = shutdown.as_mut() => {
                        sender.shutdown();
                    },
                    event = event_loop.tick() => {
                        match event? {
                            Event::Clipboard { event, .. } => match event {
                                ClipboardEvent::BitMap(bitmap) => {
                                    let decoder = image::codecs::bmp::BmpDecoder::new_without_file_header(Cursor::new(& bitmap[..]))?;
                                    let image = image::DynamicImage::from_decoder(decoder)?;
                                    system_events.send(system::Event::SendDynamicImage(image.clone()));
                                }
                                ClipboardEvent::Text(text) => {
                                    system_events.send(system::Event::SendText(text.clone()));
                                }
                                _ => {}
                            },
                            Event::MenuItemClicked { item_id, .. } => {
                                if item_id == open {
                                    let address = format!("http://localhost:{port}");
                                    open_uri::open(&address);
                                }

                                if item_id == exit {
                                    sender.shutdown();
                                }
                            },
                            Event::IconClicked { event, .. } => {
                                if event.buttons.test(MouseButton::Left) {
                                    let address = format!("http://localhost:{port}");
                                    open_uri::open(&address);
                                }
                            },
                            Event::Shutdown { .. } => {
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
