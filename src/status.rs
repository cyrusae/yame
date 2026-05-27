use std::time::{Duration, Instant};

/// Operating mode of the status bar.
#[derive(Debug, Default)]
pub enum StatusMode {
    #[default]
    Normal,
    TimedMessage {
        text: String,
        expires_at: Instant,
    },
    DismissibleMessage(String),
    ExitPrompt,
}

/// All mutable state for the status bar / hint line.
#[derive(Debug, Default)]
pub struct StatusLine {
    pub mode: StatusMode,
}

impl StatusLine {
    pub fn set_timed(&mut self, text: impl Into<String>, duration: Duration) {
        self.mode = StatusMode::TimedMessage {
            text: text.into(),
            expires_at: Instant::now() + duration,
        };
    }

    pub fn set_dismissible(&mut self, text: impl Into<String>) {
        self.mode = StatusMode::DismissibleMessage(text.into());
    }

    pub fn dismiss(&mut self) {
        if matches!(self.mode, StatusMode::DismissibleMessage(_)) {
            self.mode = StatusMode::Normal;
        }
    }

    /// Clear any expired timed messages; call each frame.
    pub fn tick(&mut self) {
        if matches!(&self.mode, StatusMode::TimedMessage { expires_at, .. } if Instant::now() >= *expires_at)
        {
            self.mode = StatusMode::Normal;
        }
    }

    /// Returns the current message text, if any.
    pub fn message(&self) -> Option<&str> {
        match &self.mode {
            StatusMode::TimedMessage { text, .. } => Some(text),
            StatusMode::DismissibleMessage(text) => Some(text),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_message_expires() {
        let mut s = StatusLine::default();
        s.set_timed("Saved.", Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(5));
        s.tick();
        assert!(
            matches!(s.mode, StatusMode::Normal),
            "expected Normal after expiry"
        );
    }

    #[test]
    fn timed_message_not_expired_yet() {
        let mut s = StatusLine::default();
        s.set_timed("Saving...", Duration::from_secs(60));
        s.tick();
        assert!(
            matches!(s.mode, StatusMode::TimedMessage { .. }),
            "should still be active"
        );
    }

    #[test]
    fn dismissible_clears_on_dismiss() {
        let mut s = StatusLine::default();
        s.set_dismissible("warning");
        s.dismiss();
        assert!(
            matches!(s.mode, StatusMode::Normal),
            "expected Normal after dismiss"
        );
    }

    #[test]
    fn dismiss_noop_on_normal() {
        let mut s = StatusLine::default();
        s.dismiss(); // should not panic
        assert!(matches!(s.mode, StatusMode::Normal));
    }

    #[test]
    fn message_returns_text_for_timed() {
        let mut s = StatusLine::default();
        s.set_timed("hello", Duration::from_secs(10));
        assert_eq!(s.message(), Some("hello"));
    }

    #[test]
    fn message_returns_none_for_normal() {
        let s = StatusLine::default();
        assert_eq!(s.message(), None);
    }

    #[test]
    fn message_returns_none_for_exit_prompt() {
        let s = StatusLine {
            mode: StatusMode::ExitPrompt,
        };
        assert_eq!(s.message(), None);
    }

    #[test]
    fn set_dismissible_sets_dismissible_mode() {
        let mut s = StatusLine::default();
        s.set_dismissible("oops");
        assert!(
            matches!(s.mode, StatusMode::DismissibleMessage(_)),
            "set_dismissible must set DismissibleMessage mode"
        );
    }

    #[test]
    fn message_returns_text_for_dismissible() {
        let mut s = StatusLine::default();
        s.set_dismissible("config error");
        assert_eq!(
            s.message(),
            Some("config error"),
            "message() must return the DismissibleMessage text"
        );
    }
}
