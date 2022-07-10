use std::borrow::Cow;

use indicatif::ProgressStyle;

/// A styled [indicatif::ProgressBar].
pub struct ProgressBar {
    bar: indicatif::ProgressBar,
}

impl ProgressBar {
    /// Creates a new ProgressBar with the specified length
    pub fn new(len: u64) -> Self {
        let bar = indicatif::ProgressBar::new(len);
        bar.set_style(
            ProgressStyle::default_bar()
                .progress_chars("=>-")
                .template("{msg:.bold.cyan/blue} [{bar:20.cyan/blue}][{percent}%] {pos}/{len}"),
        );
        Self { bar }
    }

    /// Increments the progress of the [ProgressBar] with the specified delta.
    pub fn inc(&mut self, delta: u64) {
        self.bar.inc(delta)
    }

    /// Finishes the [ProgressBar] and shows the specified message.
    pub fn finish(&self, msg: impl Into<Cow<'static, str>>) {
        self.bar.set_message(msg);
        self.bar
            .set_style(ProgressStyle::default_bar().template("{msg:.green.bold} in {elapsed}"));
        self.bar.finish_at_current_pos();
    }
}
