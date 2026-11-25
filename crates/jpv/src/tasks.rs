use std::collections::{HashMap, HashSet};
use std::fmt;

use anyhow::{Context, Result};
use slab::Slab;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TaskName {
    /// Rebuilding the specified database.
    Build(String),
}

impl TaskName {
    /// The stable id clients use to refer to the task.
    pub(crate) fn id(&self) -> String {
        match self {
            TaskName::Build(name) => format!("build:{name}"),
        }
    }
}

impl fmt::Display for TaskName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskName::Build(name) => write!(f, "Building {name}"),
        }
    }
}

type Completed = (usize, Option<TaskName>, bool);

pub(crate) struct Tasks {
    tasks: Slab<Option<oneshot::Sender<()>>>,
    completion: mpsc::UnboundedReceiver<Completed>,
    sender: mpsc::UnboundedSender<Completed>,
    unique: HashMap<TaskName, usize>,
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
    pub(crate) fn unique_task(
        &mut self,
        name: TaskName,
    ) -> Option<(oneshot::Receiver<()>, TaskCompletion)> {
        if self.unique.contains_key(&name) {
            return None;
        }

        let index = self.tasks.vacant_key();
        self.unique.insert(name.clone(), index);
        Some(self.task_inner(Some(name)))
    }

    /// Spawn a new task and set up a oneshot receiver.
    fn task_inner(&mut self, name: Option<TaskName>) -> (oneshot::Receiver<()>, TaskCompletion) {
        let (sender, receiver) = oneshot::channel();
        let index = self.tasks.insert(Some(sender));

        let completion = TaskCompletion {
            sender: self.sender.clone(),
            index,
            name,
            cancelled: false,
        };

        (receiver, completion)
    }

    /// Signal the unique task with the given id to stop, returning whether
    /// one was signalled.
    pub(crate) fn cancel(&mut self, id: &str) -> bool {
        let Some(index) = self
            .unique
            .iter()
            .find(|(name, _)| name.id() == id)
            .map(|(_, index)| *index)
        else {
            return false;
        };

        let Some(sender) = self.tasks.get_mut(index).and_then(Option::take) else {
            return false;
        };

        sender.send(()).is_ok()
    }

    /// Drive task completion in general.
    pub(crate) async fn wait(&mut self) -> Result<CompletedTask> {
        let (index, name, cancelled) = self
            .completion
            .recv()
            .await
            .context("Unexpected task queue end")?;

        self.tasks.try_remove(index);

        if let Some(name) = &name {
            self.unique.remove(name);
        }

        Ok(CompletedTask { name, cancelled })
    }

    /// Wait for all background tasks to finish.
    pub(crate) async fn finish(mut self) {
        let mut expect = HashSet::new();

        for (index, sender) in self.tasks {
            expect.insert(index);

            if let Some(sender) = sender {
                let _ = sender.send(());
            }
        }

        while !expect.is_empty() {
            tracing::trace!("Waiting for {} tasks: {expect:?}", expect.len());

            let Some((index, name, _)) = self.completion.recv().await else {
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
    sender: mpsc::UnboundedSender<Completed>,
    index: usize,
    name: Option<TaskName>,
    cancelled: bool,
}

impl TaskCompletion {
    /// Get the name of the task.
    pub(crate) fn name(&self) -> Option<&TaskName> {
        self.name.as_ref()
    }

    /// Mark the task as having stopped because it was cancelled.
    pub(crate) fn set_cancelled(&mut self) {
        self.cancelled = true;
    }
}

impl Drop for TaskCompletion {
    fn drop(&mut self) {
        tracing::trace!("Marking task {} as completed", self.index);
        let _ = self
            .sender
            .send((self.index, self.name.clone(), self.cancelled));
    }
}

pub(crate) struct CompletedTask {
    name: Option<TaskName>,
    cancelled: bool,
}

impl CompletedTask {
    pub(crate) fn name(&self) -> Option<&TaskName> {
        self.name.as_ref()
    }

    pub(crate) fn cancelled(&self) -> bool {
        self.cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::{TaskName, Tasks};

    #[tokio::test]
    async fn cancel_signals_task_and_reports_cancelled() {
        let mut tasks = Tasks::new();
        let name = TaskName::Build("jmdict".into());
        let (shutdown, mut completion) = tasks.unique_task(name.clone()).unwrap();

        assert!(!tasks.cancel("build:kanjidic2"));
        assert!(tasks.cancel(&name.id()));
        assert!(!tasks.cancel(&name.id()));
        shutdown.await.unwrap();

        completion.set_cancelled();
        drop(completion);

        let completed = tasks.wait().await.unwrap();
        assert_eq!(completed.name(), Some(&name));
        assert!(completed.cancelled());
        assert!(tasks.unique_task(name).is_some());
    }

    #[tokio::test]
    async fn completed_task_is_not_cancelled() {
        let mut tasks = Tasks::new();
        let (_shutdown, completion) = tasks.unique_task(TaskName::Build("jmdict".into())).unwrap();
        drop(completion);
        assert!(!tasks.wait().await.unwrap().cancelled());
    }
}
