//! The sizes of the app's resizable regions, kept in this browser's local
//! storage so they survive a reload and are the same in every view.

const PREFIX: &str = "jpv-region-";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

/// The share stored for the region `name`, or `default` when none is or it
/// is not a share between 0 and 1.
pub(crate) fn load(name: &str, default: f64) -> f64 {
    storage()
        .and_then(|s| s.get_item(&format!("{PREFIX}{name}")).ok().flatten())
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|at| (0.0..=1.0).contains(at))
        .unwrap_or(default)
}

pub(crate) fn store(name: &str, at: f64) {
    let Some(storage) = storage() else {
        return;
    };

    if let Err(error) = storage.set_item(&format!("{PREFIX}{name}"), &at.to_string()) {
        log::warn!("Failed to store the size of region {name}: {error:?}");
    }
}
