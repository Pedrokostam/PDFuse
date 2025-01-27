use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

// pub fn close_busy_indicator(){
// }

pub struct BusyIndicator {
    pub(crate) bar: ProgressBar,
}

impl BusyIndicator {
    pub fn new() -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .unwrap(),
        );
        bar.enable_steady_tick(Duration::from_millis(150));
        BusyIndicator { bar }
    }
    pub fn new_with_message(message:&str) -> Self {
        let s = Self::new();
        s.bar.set_message(message.to_owned());
        s
    }
    pub fn update(&self, new_status:&str) {
        self.bar.set_message(new_status.to_owned());
    }
}

impl Default for BusyIndicator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update_busy_indicator(_message: &str) {}
