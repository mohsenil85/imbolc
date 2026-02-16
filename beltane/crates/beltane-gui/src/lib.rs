//! GUI adapter boundary for Beltane.
//! This crate intentionally stays thin until a concrete GUI library is selected.

use beltane_ports::Command;

/// GUI-facing user intents before translation into application commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiAction {
    TogglePlay,
    Unknown,
}

/// Inbound adapter mapping from GUI action to application command.
pub fn map_gui_action(action: GuiAction) -> Option<Command> {
    match action {
        GuiAction::TogglePlay => Some(Command::TogglePlay),
        GuiAction::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use beltane_ports::Command;

    use super::{map_gui_action, GuiAction};

    #[test]
    fn maps_toggle_play_action() {
        assert_eq!(
            map_gui_action(GuiAction::TogglePlay),
            Some(Command::TogglePlay)
        );
    }

    #[test]
    fn ignores_unknown_action() {
        assert_eq!(map_gui_action(GuiAction::Unknown), None);
    }
}
