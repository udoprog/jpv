use std::{cell::RefCell, rc::Rc};

use lib::api::ClientEvent;
use yew::Callback;

#[derive(Default)]
struct Inner {
    client_event: Option<Callback<ClientEvent>>,
}

#[derive(Default, Clone)]
pub(crate) struct Callbacks {
    inner: Rc<RefCell<Inner>>,
}

impl Callbacks {
    /// Set client event listener.
    pub(crate) fn set_client_event(&self, callback: Callback<ClientEvent>) {
        self.inner.borrow_mut().client_event = Some(callback);
    }

    /// Clear client event listener.
    pub(crate) fn clear_client_event(&self) {
        self.inner.borrow_mut().client_event = None;
    }

    /// Emit client event.
    pub(crate) fn emit_client_event(&self, event: ClientEvent) {
        let Some(client_event) = &self.inner.borrow().client_event else {
            return;
        };

        client_event.emit(event);
    }
}

impl PartialEq for Callbacks {
    #[inline]
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
