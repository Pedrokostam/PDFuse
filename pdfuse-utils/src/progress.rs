
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
        bar.enable_steady_tick(Duration::from_millis(100));
        BusyIndicator{bar}
    }
    pub fn update(&self, index: usize) {
        self.bar.set_message(format!("Found {} items...", index));
    }
}

impl Default for BusyIndicator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn update_busy_indicator(_message: &str){
}

