use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use lib::api;
use parking_lot::Mutex;
use tracing::Subscriber;
use tracing_subscriber::fmt::format::{PrettyVisitor, Writer};

use crate::system::{self, Event};

const LIMIT: usize = 100;

/// Rotating statically known index of the current thread.
static THREAD_INDEX: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static THREAD_INDEX_THREAD: Cell<Option<usize>> = Cell::new(None);
}

pub fn new(system_events: system::SystemEvents) -> (Layer, Capture) {
    let threads = num_cpus::get();
    let mut log = Vec::with_capacity(threads);

    for _ in 0..threads.max(1) {
        log.push(Mutex::new(VecDeque::new()));
    }

    let inner = Arc::new(Inner {
        log,
        limit: LIMIT,
        start: Instant::now(),
        start_time: SystemTime::now(),
    });

    let layer = Layer {
        inner: inner.clone(),
        system_events,
    };

    let capturing = Capture { inner };
    (layer, capturing)
}

struct Inner {
    log: Vec<Mutex<VecDeque<api::OwnedLogEntry>>>,
    limit: usize,
    start: Instant,
    start_time: SystemTime,
}

impl Inner {
    fn timestamp(&self, instant: Instant) -> Option<u128> {
        let duration = instant.checked_duration_since(self.start)?;
        let timestamp = self.start_time.checked_add(duration)?;
        Some(
            timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?
                .as_millis(),
        )
    }
}

/// Layer that performs the capturing.
pub struct Layer {
    inner: Arc<Inner>,
    system_events: system::SystemEvents,
}

impl Layer {
    fn emit(&self, entry: api::OwnedLogEntry) {
        self.system_events.send(Event::LogEntry(entry));
    }
}

impl<S> tracing_subscriber::Layer<S> for Layer
where
    S: Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let level = *event.metadata().level();
        let module = event.metadata().module_path().map(Box::<str>::from);

        let mut text = String::with_capacity(32);
        let mut visitor = PrettyVisitor::new(Writer::new(&mut text), true);
        event.record(&mut visitor);

        let index = THREAD_INDEX_THREAD.with(|index| {
            if let Some(index) = index.get() {
                return index;
            }

            let new_index = THREAD_INDEX.fetch_add(1, Ordering::Relaxed);
            index.set(Some(new_index));
            new_index
        });

        let at = index % self.inner.log.len();

        let timestamp = Instant::now();

        let timestamp = self.inner.timestamp(timestamp).unwrap_or(u128::MIN);

        let entry = api::OwnedLogEntry {
            timestamp,
            target: module.as_deref().unwrap_or("").to_owned(),
            level: to_level_string(level).to_owned(),
            text,
        };

        self.emit(entry.clone());

        let mut log = self.inner.log[at].lock();

        log.push_back(entry);

        if log.len() > self.inner.limit {
            log.pop_front();
        }
    }
}

/// Capturing handle.
#[derive(Clone)]
pub struct Capture {
    inner: Arc<Inner>,
}

impl Capture {
    pub(crate) fn read(&self) -> Vec<api::OwnedLogEntry> {
        let mut output = Vec::new();

        for log in &self.inner.log {
            log.lock().iter().for_each(|entry| {
                output.push(entry.clone());
            });
        }

        output.sort_by_key(|entry| entry.timestamp);
        output
    }
}

fn to_level_string(level: tracing::Level) -> &'static str {
    match level {
        tracing::Level::ERROR => "error",
        tracing::Level::WARN => "warn",
        tracing::Level::INFO => "info",
        tracing::Level::DEBUG => "debug",
        tracing::Level::TRACE => "trace",
    }
}
