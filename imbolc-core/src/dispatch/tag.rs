//! Dispatch handler for TagAction.

use crate::state::AppState;
use imbolc_types::action::{DispatchResult, TagAction};
use imbolc_types::state::midi_recording::{CcMappingSource, MidiCcMapping};

pub(super) fn dispatch_tag(action: &TagAction, state: &mut AppState) -> DispatchResult {
    // Delegate pure state mutation to the shared reducer
    imbolc_types::reduce::reduce_action(
        &imbolc_types::DomainAction::Tag(action.clone()),
        &mut state.instruments,
        &mut state.session,
    );

    match action {
        // When selecting a tag, apply CC auto-mapping
        TagAction::SelectTag(_) => {
            apply_tag_cc_mapping(state);
            DispatchResult::none()
        }
        // When setting pending target, navigate to the tag picker
        TagAction::SetPendingTarget(target) => {
            use crate::action::NavIntent;
            let mut result = DispatchResult::none();
            result.nav.push(NavIntent::OpenTagPicker(target.clone()));
            result
        }
        _ => DispatchResult::none(),
    }
}

/// Clear all tag-generated CC mappings and re-apply for the currently selected tag.
fn apply_tag_cc_mapping(state: &mut AppState) {
    // Remove all tag-generated mappings
    state
        .session
        .midi_recording
        .cc_mappings
        .retain(|m| m.source != CcMappingSource::Tag);

    // If a tag is selected, map its targets to sequential CCs starting at 1
    if let Some(tag) = state.session.param_tags.selected() {
        let mut cc = 1u8;
        for target in &tag.targets {
            if cc >= 120 {
                break; // Reserve CCs 120+ for system messages
            }
            // Skip if this CC is already manually mapped
            let cc_taken = state
                .session
                .midi_recording
                .cc_mappings
                .iter()
                .any(|m| m.cc_number == cc && m.source == CcMappingSource::Manual);
            if cc_taken {
                cc += 1;
                continue;
            }
            let mut mapping = MidiCcMapping::new(cc, target.clone());
            mapping.source = CcMappingSource::Tag;
            state.session.midi_recording.cc_mappings.push(mapping);
            cc += 1;
        }
    }
}
