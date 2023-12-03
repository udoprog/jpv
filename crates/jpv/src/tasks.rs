use std::collections::{HashMap, HashSet};
use std::fmt;

use anyhow::{Context, Result};
use slab::Slab;
use tokio::sync::{mpsc, oneshot};

pub(crate) struct Tasks {
    tasks: Slab<oneshot::Sender<()>>,
    completion: mpsc::UnboundedReceiver<(usize, Option<Box<str>>)>,
    sender: mpsc::UnboundedSender<(usize, Option<Box<str>>)>,
    unique: HashMap<Box<str>, usize>,
}

impl Tasks {
    pub(crate) fn new() -> Self {
        let (sender, completion) = mpsc::unbounded_channel();
        Self {
            tasks: Slab::new(),
            completion,
            sender,
            unique: HashMap::new(),
        }
    }

    /// Spawn a unique task with the given name.
    ///
    /// This returns a tuple of a oneshot that will be signalled if the task
    /// needs to be cancelled, or a completion handler that must be dropped once
    /// the task has completed.
    pub(crate) fn unique_task<N>(
        &mut self,
        name: N,
    ) -> Option<(oneshot::Receiver<()>, TaskCompletion)>
    where
        N: fmt::Display,
    {
        let name = Box::from(name.to_string());

        if self.unique.get(&name).is_some() {
            return None;
        }

        let index = self.tasks.vacant_key();
        self.unique.insert(name.clone(), index);
        Some(self.task_inner(Some(name)))
    }

    /// Spawn a new task and set up a oneshot receiver.
    fn task_inner(&mut self, name: Option<Box<str>>) -> (oneshot::Receiver<()>, TaskCompletion) {
        let (sender, receiver) = oneshot::channel();
        let index = self.tasks.insert(sender);

        let completion = TaskCompletion {
            sender: self.sender.clone(),
            index,
            name,
        };

        (receiver, completion)
    }

    /// Drive task completion in general.
    pub(crate) async fn wait(&mut self) -> Result<CompletedTask> {
        loop {
            let (index, name) = self
                .completion
                .recv()
                .await
                .context("Unexpected task queue end")?;

            self.tasks.remove(index);

            if let Some(name) = &name {
                self.unique.remove(name);
            }

            return Ok(CompletedTask { name });
        }
    }

    /// Wait for all background tasks to finish.
    pub(crate) async fn finish(mut self) {
        let mut expect = HashSet::new();

        for (index, sender) in self.tasks {
            expect.insert(index);
            let _ = sender.send(());
        }

        while !expect.is_empty() {
            tracing::trace!("Waiting for {} tasks: {expect:?}", expect.len());

            let Some((index, name)) = self.completion.recv().await else {
                break;
            };

            expect.remove(&index);

            if let Some(name) = &name {
                self.unique.remove(name);
            }
        }

        tracing::trace!("Done waiting!");
    }
}

pub(crate) struct TaskCompletion {
    sender: mpsc::UnboundedSender<(usize, Option<Box<str>>)>,
    index: usize,
    name: Option<Box<str>>,
}

impl TaskCompletion {
    /// Get the name of the task.
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Drop for TaskCompletion {
    fn drop(&mut self) {
        tracing::trace!("Marking task {} as completed", self.index);
        let _ = self.sender.send((self.index, self.name.clone()));
    }
}

pub(crate) struct CompletedTask {
    name: Option<Box<str>>,
}

impl CompletedTask {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
