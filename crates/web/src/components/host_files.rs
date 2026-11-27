//! Lists the service's directories for a [`sockel::FileBrowser`].

use std::cell::RefCell;
use std::rc::Rc;

use lib::api;
use sockel::{EntryKind, ListRequest, Listing, ListingEntry, Place, PlaceKind};
use yew::prelude::*;

use crate::ws;

/// A `list` callback for a file browser that asks the service. Only the
/// latest request is kept: an earlier one is cancelled.
pub(crate) fn lister(ws: &ws::Handle) -> Callback<ListRequest> {
    let ws = ws.clone();
    let pending = Rc::new(RefCell::new(ws::Request::empty()));

    Callback::from(move |request: ListRequest| {
        let reply = request.reply;

        let callback = Callback::from(
            move |result: crate::error::Result<api::ListDirectoryResponse>| {
                reply.emit(result.map(listing).map_err(|error| error.to_string()));
            },
        );

        let handle = ws.request(
            api::ListDirectory {
                path: request.path,
                nearest: request.nearest,
            },
            callback,
        );

        *pending.borrow_mut() = handle;
    })
}

fn listing(response: api::ListDirectoryResponse) -> Listing {
    Listing {
        path: response.path,
        parent: response.parent,
        separator: response.separator,
        entries: response
            .entries
            .into_iter()
            .map(|e| ListingEntry {
                name: e.name,
                kind: match e.kind {
                    api::DirectoryEntryKind::Directory => EntryKind::Directory,
                    api::DirectoryEntryKind::File => EntryKind::File,
                    api::DirectoryEntryKind::Other => EntryKind::Other,
                },
                size: e.size,
            })
            .collect(),
        file: response.file,
        places: response
            .places
            .into_iter()
            .map(|p| {
                let kind = match p.kind {
                    api::PlaceKind::Home => PlaceKind::Home,
                    api::PlaceKind::Documents => PlaceKind::Documents,
                    api::PlaceKind::Downloads => PlaceKind::Downloads,
                    api::PlaceKind::Root => PlaceKind::Root,
                };

                Place::new(kind, p.path)
            })
            .collect(),
        notice: None,
    }
}
