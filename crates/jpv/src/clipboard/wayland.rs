use std::collections::HashMap;
use std::io;
use std::os::fd::{AsFd, OwnedFd};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncReadExt, Interest};
use tokio::sync::watch;
use wayland_client::backend::{ObjectId, WaylandError};
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_callback, wl_registry, wl_seat};
use wayland_client::{event_created_child, Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::ext::data_control::v1::client as ext;
use wayland_protocols_wlr::data_control::v1::client as wlr;

use crate::system::{Event, SendClipboardData, SystemEvents};

/// Mime types we accept in order of preference. This is the same order as the
/// GNOME extension uses, so that the service handles them the same way.
const MIME_TYPES: &[&str] = &[
    "UTF8_STRING",
    "STRING",
    "text/plain;charset=utf-8",
    "text/plain",
    "image/png",
    "image/tiff",
    "image/webp",
    "image/jpeg",
    "image/jpg",
];

/// The largest selection we read.
const MAX_SIZE: u64 = 64 * 1024 * 1024;

/// How long we wait for the selection owner to send its data.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

/// Pick the mime type to request out of the ones offered.
fn pick_mime_type<'a, I>(offered: I) -> Option<&'static str>
where
    I: IntoIterator<Item = &'a str>,
    I::IntoIter: Clone,
{
    let offered = offered.into_iter();

    MIME_TYPES
        .iter()
        .copied()
        .find(|wanted| offered.clone().any(|m| m == *wanted))
}

/// Watch the clipboard for as long as the service runs, sending each new
/// selection as [`Event::SendClipboardData`] while `capture` is set.
///
/// Returns once if the compositor does not support data-control, after logging
/// why.
pub(crate) async fn run(capture: watch::Receiver<bool>, system_events: SystemEvents) -> Result<()> {
    let conn = match Connection::connect_to_env() {
        Ok(conn) => conn,
        Err(error) => {
            tracing::info!("Clipboard capture unavailable, no Wayland compositor: {error}");
            return Ok(());
        }
    };

    let Some(watcher) = Watcher::new(&conn)? else {
        tracing::info!(
            "Clipboard capture unavailable, the compositor supports neither ext-data-control-v1 nor zwlr-data-control-v1"
        );
        return Ok(());
    };

    tracing::info!("Clipboard capture available through {}", watcher.protocol);
    watcher.run(conn, capture, system_events).await
}

/// An offer from either protocol.
enum Offer {
    Ext(ext::ext_data_control_offer_v1::ExtDataControlOfferV1),
    Wlr(wlr::zwlr_data_control_offer_v1::ZwlrDataControlOfferV1),
}

impl Offer {
    fn id(&self) -> ObjectId {
        match self {
            Offer::Ext(offer) => offer.id(),
            Offer::Wlr(offer) => offer.id(),
        }
    }

    fn receive(&self, mime_type: &str, fd: &OwnedFd) {
        match self {
            Offer::Ext(offer) => offer.receive(mime_type.to_owned(), fd.as_fd()),
            Offer::Wlr(offer) => offer.receive(mime_type.to_owned(), fd.as_fd()),
        }
    }

    fn destroy(&self) {
        match self {
            Offer::Ext(offer) => offer.destroy(),
            Offer::Wlr(offer) => offer.destroy(),
        }
    }
}

/// A new clipboard selection.
struct Selection {
    offer: Offer,
    mime_types: Vec<String>,
}

#[derive(Default)]
struct State {
    /// Mime types of offers not yet announced as a selection.
    offers: HashMap<ObjectId, Vec<String>>,
    /// The newest selection, not yet handled.
    selection: Option<Selection>,
    /// The selection that is current, which is destroyed once replaced.
    current: Option<Offer>,
    /// Set once the compositor has sent the selection that was current when we
    /// connected, which is not captured.
    synced: bool,
    /// The data device is gone.
    finished: bool,
}

impl State {
    fn data_offer(&mut self, id: ObjectId) {
        self.offers.insert(id, Vec::new());
    }

    fn offer(&mut self, id: ObjectId, mime_type: String) {
        if let Some(mime_types) = self.offers.get_mut(&id) {
            mime_types.push(mime_type);
        }
    }

    fn selection(&mut self, offer: Option<Offer>) {
        if let Some(old) = self.selection.take() {
            old.offer.destroy();
        }

        if let Some(old) = self.current.take() {
            old.destroy();
        }

        let Some(offer) = offer else {
            return;
        };

        let mime_types = self.offers.remove(&offer.id()).unwrap_or_default();

        if self.synced {
            self.selection = Some(Selection { offer, mime_types });
        } else {
            self.current = Some(offer);
        }
    }

    /// The primary selection is not captured.
    fn primary_selection(&mut self, offer: Option<Offer>) {
        if let Some(offer) = offer {
            self.offers.remove(&offer.id());
            offer.destroy();
        }
    }
}

struct Watcher {
    queue: wayland_client::EventQueue<State>,
    state: State,
    protocol: &'static str,
}

impl Watcher {
    fn new(conn: &Connection) -> Result<Option<Self>> {
        let (globals, queue) =
            registry_queue_init::<State>(conn).context("Listing Wayland globals")?;
        let qh = queue.handle();

        let Ok(seat) = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=1, ()) else {
            tracing::info!("Clipboard capture unavailable, the compositor has no seat");
            return Ok(None);
        };

        // The data device lives for as long as the connection does.
        let protocol = if let Ok(manager) =
            globals.bind::<ext::ext_data_control_manager_v1::ExtDataControlManagerV1, _, _>(
                &qh,
                1..=1,
                (),
            ) {
            manager.get_data_device(&seat, &qh, ());
            "ext-data-control-v1"
        } else if let Ok(manager) = globals
            .bind::<wlr::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1, _, _>(
            &qh,
            1..=2,
            (),
        ) {
            manager.get_data_device(&seat, &qh, ());
            "zwlr-data-control-v1"
        } else {
            return Ok(None);
        };

        // The selection which is current when the device is created is sent
        // before this callback fires, and is not captured.
        conn.display().sync(&qh, ());

        Ok(Some(Self {
            queue,
            state: State::default(),
            protocol,
        }))
    }

    async fn run(
        mut self,
        conn: Connection,
        mut capture: watch::Receiver<bool>,
        system_events: SystemEvents,
    ) -> Result<()> {
        let poll_fd = conn
            .backend()
            .poll_fd()
            .try_clone_to_owned()
            .context("Duplicating the Wayland connection")?;

        // SAFETY: The descriptor is owned by the AsyncFd, so it stays open and
        // refers to the same file description for as long as it is registered.
        let fd = unsafe { AsyncFd::register_with_interest(poll_fd, Interest::READABLE) }
            .map_err(|error| error.into_parts().1)?;

        let mut capture_closed = false;

        loop {
            self.queue
                .dispatch_pending(&mut self.state)
                .context("Dispatching Wayland events")?;

            if self.state.finished {
                bail!("The compositor removed the clipboard data device");
            }

            if let Some(selection) = self.state.selection.take() {
                if *capture.borrow() {
                    match self.read(&conn, &selection).await {
                        Ok(Some(data)) => system_events.send(Event::SendClipboardData(data)),
                        Ok(None) => {}
                        Err(error) => tracing::warn!("Reading the clipboard: {error:#}"),
                    }
                }

                self.state.current = Some(selection.offer);
                // Something might have arrived while we were reading.
                continue;
            }

            self.queue.flush().context("Flushing Wayland requests")?;

            let Some(guard) = self.queue.prepare_read() else {
                continue;
            };

            tokio::select! {
                ready = fd.readable() => {
                    let mut ready = ready?;

                    match guard.read() {
                        Ok(_) => {}
                        Err(WaylandError::Io(error)) if error.kind() == io::ErrorKind::WouldBlock => {
                            ready.clear_ready();
                        }
                        Err(error) => {
                            return Err(error).context("Reading Wayland events");
                        }
                    }
                }
                changed = capture.changed(), if !capture_closed => {
                    // A selection made while capture was off is not captured
                    // when it is turned on.
                    capture_closed = changed.is_err();
                }
            }
        }
    }

    /// Read the given selection in the preferred mime type.
    async fn read(
        &self,
        conn: &Connection,
        selection: &Selection,
    ) -> Result<Option<SendClipboardData>> {
        let Some(mime_type) = pick_mime_type(selection.mime_types.iter().map(String::as_str))
        else {
            tracing::debug!(mime_types = ?selection.mime_types, "No supported mime type in selection");
            return Ok(None);
        };

        let (reader, writer) = io::pipe()?;
        let writer = OwnedFd::from(writer);
        selection.offer.receive(mime_type, &writer);
        conn.flush().context("Flushing Wayland requests")?;
        // Our copy of the write end must be closed for the read to end.
        drop(writer);

        let mut reader = tokio::net::unix::pipe::Receiver::from_owned_fd(OwnedFd::from(reader))?;
        let mut data = Vec::new();

        let read = async {
            (&mut reader)
                .take(MAX_SIZE + 1)
                .read_to_end(&mut data)
                .await
        };

        match tokio::time::timeout(READ_TIMEOUT, read).await {
            Ok(result) => {
                result?;
            }
            Err(..) => bail!("Timed out reading {mime_type}"),
        }

        if data.len() as u64 > MAX_SIZE {
            bail!("Selection is larger than {MAX_SIZE} bytes");
        }

        if data.is_empty() {
            return Ok(None);
        }

        tracing::trace!(?mime_type, len = data.len(), "Captured clipboard");

        Ok(Some(SendClipboardData {
            mimetype: mime_type.to_owned(),
            data,
        }))
    }
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for State {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_callback::WlCallback, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_callback::WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.synced = true;
        }
    }
}

/// Implement dispatch for one of the data-control protocols, which have the
/// same shape.
macro_rules! data_control {
    ($variant:ident, $manager:ty, $device_mod:ident, $device:ty, $offer_mod:ident, $offer:ty) => {
        impl Dispatch<$manager, ()> for State {
            fn event(
                _: &mut Self,
                _: &$manager,
                _: <$manager as Proxy>::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
            }
        }

        impl Dispatch<$device, ()> for State {
            fn event(
                state: &mut Self,
                _: &$device,
                event: $device_mod::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
                match event {
                    $device_mod::Event::DataOffer { id } => {
                        state.data_offer(id.id());
                    }
                    $device_mod::Event::Selection { id } => {
                        state.selection(id.map(Offer::$variant));
                    }
                    $device_mod::Event::PrimarySelection { id } => {
                        state.primary_selection(id.map(Offer::$variant));
                    }
                    $device_mod::Event::Finished => {
                        state.finished = true;
                    }
                    _ => {}
                }
            }

            event_created_child!(State, $device, [
                $device_mod::EVT_DATA_OFFER_OPCODE => ($offer, ()),
            ]);
        }

        impl Dispatch<$offer, ()> for State {
            fn event(
                state: &mut Self,
                offer: &$offer,
                event: $offer_mod::Event,
                _: &(),
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
                if let $offer_mod::Event::Offer { mime_type } = event {
                    state.offer(offer.id(), mime_type);
                }
            }
        }
    };
}

use ext::ext_data_control_device_v1 as ext_device;
use ext::ext_data_control_offer_v1 as ext_offer;
use wlr::zwlr_data_control_device_v1 as wlr_device;
use wlr::zwlr_data_control_offer_v1 as wlr_offer;

data_control!(
    Ext,
    ext::ext_data_control_manager_v1::ExtDataControlManagerV1,
    ext_device,
    ext_device::ExtDataControlDeviceV1,
    ext_offer,
    ext_offer::ExtDataControlOfferV1
);

data_control!(
    Wlr,
    wlr::zwlr_data_control_manager_v1::ZwlrDataControlManagerV1,
    wlr_device,
    wlr_device::ZwlrDataControlDeviceV1,
    wlr_offer,
    wlr_offer::ZwlrDataControlOfferV1
);

#[cfg(test)]
mod tests {
    use super::pick_mime_type;

    #[test]
    fn prefers_mime_types_in_extension_order() {
        assert_eq!(
            pick_mime_type(["text/plain", "UTF8_STRING", "TEXT"]),
            Some("UTF8_STRING")
        );
        assert_eq!(
            pick_mime_type(["text/plain", "text/plain;charset=utf-8"]),
            Some("text/plain;charset=utf-8")
        );
        assert_eq!(
            pick_mime_type(["image/jpeg", "image/png", "text/html"]),
            Some("image/png")
        );
        assert_eq!(pick_mime_type(["text/html", "TEXT"]), None);
        assert_eq!(pick_mime_type([]), None);
    }
}
