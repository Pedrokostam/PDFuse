use std::{borrow::Cow, time::Duration};

use indicatif::{ProgressBar, ProgressStyle};

// pub fn close_busy_indicator(){
// }

pub fn get_busy_indicator() -> ProgressBar {
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
    b.enable_steady_tick(Duration::from_millis(50));
    b
}

pub fn get_progress_indicator(
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
