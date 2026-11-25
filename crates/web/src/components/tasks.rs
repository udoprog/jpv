use std::collections::BTreeSet;

use lib::api;
use phrasebook::args;
use phrasebook::yew023::use_phrases;
use sockel::Gap;
use yew::prelude::*;

use super::icons;
use crate::class;

#[derive(Properties, PartialEq)]
pub(crate) struct TasksProps {
    pub(crate) tasks: Vec<api::OwnedTaskProgress>,
    /// Ids of tasks a cancel has been asked for.
    pub(crate) cancelling: BTreeSet<String>,
    /// Tasks that stopped because they were cancelled, until dismissed.
    pub(crate) cancelled: Vec<api::OwnedTaskCompleted>,
    /// Leave out the step text, as the embedded view is narrow.
    pub(crate) embed: bool,
    pub(crate) oncancel: Callback<String>,
    pub(crate) ondismiss: Callback<String>,
}

/// Running tasks with their progress and a cancel button each, and a notice
/// for each cancelled one.
#[function_component]
pub(crate) fn Tasks(props: &TasksProps) -> Html {
    let phrases = use_phrases();

    if props.tasks.is_empty() && props.cancelled.is_empty() {
        return Html::default();
    }

    let tasks = props.tasks.iter().map(|task| {
        let (progress, value) = match task.total {
            Some(total) => {
                let progress = html! {
                    <progress max={total.to_string()} value={task.value.to_string()} />
                };

                (progress, None)
            }
            None => {
                let progress = html!(<progress />);
                let value = html!(<div>{task.value.to_string()}</div>);
                (progress, Some(value))
            }
        };

        let cancelling = props.cancelling.contains(&task.id);

        let progress_text = if cancelling {
            phrases.text("task-cancelling").to_string()
        } else {
            format!("{} ...", task.text)
        };

        let text = (!props.embed).then(|| html!(<div>{progress_text.clone()}</div>));

        let onclick = props.oncancel.reform({
            let id = task.id.clone();
            move |_| id.clone()
        });

        let label = phrases.msg("task-cancel", &args!(task = task.name.as_str()));

        html! {
            <sockel::Cluster gap={Gap::Tight} nowrap={true} class={class::TASK}>
                <span data-tip={progress_text}>
                    <sockel::Badge tone={sockel::Tone::Accent} solid={true}>{&task.name}</sockel::Badge>
                </span>
                <div>{format!("{}/{}", task.step, task.steps)}</div>
                {text}
                <div class={class::TASK_PROGRESS}>{progress}</div>
                {value}
                <sockel::Button variant={sockel::ButtonVariant::Quiet} size={sockel::ButtonSize::Small}
                    icon={icons::glyph(sockel::Glyph::Close)} disabled={cancelling} {onclick}
                    tip={label.clone()} label={label} />
            </sockel::Cluster>
        }
    });

    let cancelled = props.cancelled.iter().map(|task| {
        let onclose = props.ondismiss.reform({
            let id = task.id.clone();
            move |_| id.clone()
        });

        html! {
            <sockel::Notice tone={sockel::Tone::Warning} {onclose}>
                {phrases.msg("task-cancelled", &args!(task = task.name.as_str()))}
            </sockel::Notice>
        }
    });

    html! {
        <sockel::Stack gap={Gap::Snug}>
            {for tasks}
            {for cancelled}
        </sockel::Stack>
    }
}
