use crate::action::DispatchResult;
use crate::state::AppState;
use imbolc_types::{DomainAction, InstrumentAction};

pub(super) fn handle_select(state: &mut AppState, action: &InstrumentAction) -> DispatchResult {
    imbolc_types::reduce::reduce_action(
        &DomainAction::Track(action.clone()),
        &mut state.tracks,
        &mut state.session,
    );
    DispatchResult::none()
}
