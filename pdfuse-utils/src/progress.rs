use std::{borrow::Cow, time::Duration};

use indicatif::{
    ParallelProgressIterator, ProgressBar, ProgressBarIter, ProgressIterator, ProgressStyle,
};

use crate::register_progressbar;
use rayon::prelude::*;

// pub fn close_busy_indicator(){
// }

pub fn get_registered_busy_indicator(msg: &str) -> ProgressBar {
    let b = ProgressBar::new_spinner().with_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg} | {elapsed}")
            .unwrap()
            .tick_strings(&[
                "▁▃▅▇█▇▅▃",
                "▃▅▇█▇▅▃▁",
                "▅▇█▇▅▃▁▃",
                "▇█▇▅▃▁▃▅",
                "█▇▅▃▁▃▅▇",
                "▇▅▃▁▃▅▇█",
                "▅▃▁▃▅▇█▇",
                "▃▁▃▅▇█▇▅",
            ]),
    );
    b.set_message(msg.to_owned());
    b.enable_steady_tick(Duration::from_millis(50));
    register_progressbar(b)
}

pub fn get_registered_progress_iterator<T, I>(
    items: I,
    message: impl Into<Cow<'static, str>>,
) -> ProgressBarIter<I::IntoIter>
where
    I: IntoIterator<Item = T> + ExactSizeIterator,
    I::IntoIter: ExactSizeIterator,
{
    let actual_iterator = items.into_iter();
    let count = actual_iterator.len() as u64;
    let p = get_progress_indicator(count, message);
    actual_iterator.progress_with(register_progressbar(p))
}

pub fn get_registered_progress_iterator_parallel<I>(
    items: I,
    message: impl Into<Cow<'static, str>>,
) -> ProgressBarIter<<I as IntoParallelIterator>::Iter>
where
    I: IntoParallelIterator,
    I::Iter: IndexedParallelIterator,
    I::Item: Send,
{
    let actual_par_iterator = items.into_par_iter();
    let count = actual_par_iterator.len() as u64;
    let p = get_progress_indicator(count, message);
    actual_par_iterator.progress_with(register_progressbar(p))
}

fn get_progress_indicator(
    total_count: impl Into<u64>,
    message: impl Into<Cow<'static, str>>,
) -> ProgressBar {
    ProgressBar::new(total_count.into())
        .with_style(
            ProgressStyle::default_spinner()
                .template(
                    "{msg} {wide_bar:.green/dim} {pos}/{len} {elapsed_precise}/{duration_precise}",
                )
                .unwrap()
                .progress_chars("█▒░"),
        )
        .with_message(message)
}

pub struct BusyIndicator(ProgressBar);

impl BusyIndicator {
    pub fn new() -> Self {
        let bar = ProgressBar::new_spinner().with_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .unwrap(),
        );
        bar.enable_steady_tick(Duration::from_millis(150));
        BusyIndicator(bar)
    }
    pub fn new_with_message(message: &str) -> Self {
        let s = Self::new();
        s.0.set_message(message.to_owned());
        s
    }
    pub fn update(&self, new_status: &str) {
        self.0.set_message(new_status.to_owned());
    }
}

impl Default for BusyIndicator {
    fn default() -> Self {
        Self::new()
    }
}
