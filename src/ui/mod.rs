//! Real-time observability dashboard built with Ratatui.
//!
//! Layout follows the design brief surfaced from the project research pass:
//! depth-of-book ladder as visual anchor, trade tape in the second-loudest
//! pane, latency / throughput / engine summary as supporting cards, mid-price
//! chart, and a Helix-style left/center/right statusline at the bottom.
//!
//! Theme respect: every color is `Color::Reset` or one of the ANSI 16
//! (`Green`, `Red`, `Yellow`, `DarkGray`, `LightGreen`, `LightRed`,
//! `LightYellow`). Terminals with curated themes (Catppuccin, Solarized,
//! Gruvbox, etc.) remap these correctly.

pub mod app;
pub mod panes;
pub mod theme;

pub use app::{run, run_with_app, Action, App, Mode};
