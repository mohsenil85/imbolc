//! Pure state-mutation reducer for TagAction.

use crate::{SessionState, TagAction};

pub(super) fn reduce(action: &TagAction, session: &mut SessionState) {
    match action {
        TagAction::CreateTag(name) => {
            session
                .param_tags
                .tags
                .push(crate::ParamTag::new(name.clone()));
            // Auto-select if first tag
            if session.param_tags.tags.len() == 1 {
                session.param_tags.selected_tag = Some(0);
            }
        }
        TagAction::DeleteTag(idx) => {
            if *idx < session.param_tags.tags.len() {
                session.param_tags.tags.remove(*idx);
                // Adjust selection
                if session.param_tags.tags.is_empty() {
                    session.param_tags.selected_tag = None;
                } else if let Some(sel) = session.param_tags.selected_tag {
                    if sel >= session.param_tags.tags.len() {
                        session.param_tags.selected_tag = Some(session.param_tags.tags.len() - 1);
                    }
                }
            }
        }
        TagAction::RenameTag(idx, name) => {
            if let Some(tag) = session.param_tags.tags.get_mut(*idx) {
                tag.name = name.clone();
            }
        }
        TagAction::AddTarget(idx, target) => {
            if let Some(tag) = session.param_tags.tags.get_mut(*idx) {
                tag.add_target(target.clone());
            }
        }
        TagAction::RemoveTarget(idx, target) => {
            if let Some(tag) = session.param_tags.tags.get_mut(*idx) {
                tag.remove_target(target);
            }
        }
        TagAction::SelectTag(sel) => {
            session.param_tags.selected_tag = sel.filter(|i| *i < session.param_tags.tags.len());
        }
        TagAction::SetPendingTarget(target) => {
            session.param_tags.pending_target = Some(target.clone());
        }
        TagAction::MoveTarget(tag_idx, target_idx, delta) => {
            if let Some(tag) = session.param_tags.tags.get_mut(*tag_idx) {
                let len = tag.targets.len();
                if *target_idx < len {
                    let new_idx =
                        (*target_idx as i64 + *delta as i64).clamp(0, len as i64 - 1) as usize;
                    if new_idx != *target_idx {
                        let item = tag.targets.remove(*target_idx);
                        tag.targets.insert(new_idx, item);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AutomationTarget, InstrumentId};

    fn make_session() -> SessionState {
        SessionState::new()
    }

    #[test]
    fn create_tag_auto_selects_first() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("bass".into()), &mut s);
        assert_eq!(s.param_tags.tags.len(), 1);
        assert_eq!(s.param_tags.selected_tag, Some(0));
    }

    #[test]
    fn create_second_tag_does_not_change_selection() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("a".into()), &mut s);
        reduce(&TagAction::CreateTag("b".into()), &mut s);
        assert_eq!(s.param_tags.tags.len(), 2);
        assert_eq!(s.param_tags.selected_tag, Some(0));
    }

    #[test]
    fn delete_tag_adjusts_selection() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("a".into()), &mut s);
        reduce(&TagAction::CreateTag("b".into()), &mut s);
        s.param_tags.selected_tag = Some(1);
        reduce(&TagAction::DeleteTag(1), &mut s);
        assert_eq!(s.param_tags.tags.len(), 1);
        assert_eq!(s.param_tags.selected_tag, Some(0));
    }

    #[test]
    fn delete_last_tag_clears_selection() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("a".into()), &mut s);
        reduce(&TagAction::DeleteTag(0), &mut s);
        assert!(s.param_tags.tags.is_empty());
        assert_eq!(s.param_tags.selected_tag, None);
    }

    #[test]
    fn rename_tag() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("old".into()), &mut s);
        reduce(&TagAction::RenameTag(0, "new".into()), &mut s);
        assert_eq!(s.param_tags.tags[0].name, "new");
    }

    #[test]
    fn add_and_remove_target() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("test".into()), &mut s);
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(1));
        reduce(&TagAction::AddTarget(0, target.clone()), &mut s);
        assert_eq!(s.param_tags.tags[0].targets.len(), 1);
        reduce(&TagAction::RemoveTarget(0, target), &mut s);
        assert!(s.param_tags.tags[0].targets.is_empty());
    }

    #[test]
    fn select_tag_bounds_check() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("a".into()), &mut s);
        reduce(&TagAction::SelectTag(Some(99)), &mut s);
        assert_eq!(s.param_tags.selected_tag, None);
    }

    #[test]
    fn move_target() {
        let mut s = make_session();
        reduce(&TagAction::CreateTag("test".into()), &mut s);
        let id = InstrumentId::new(1);
        let t0 = AutomationTarget::filter_cutoff(id);
        let t1 = AutomationTarget::level(id);
        let t2 = AutomationTarget::pan(id);
        reduce(&TagAction::AddTarget(0, t0.clone()), &mut s);
        reduce(&TagAction::AddTarget(0, t1.clone()), &mut s);
        reduce(&TagAction::AddTarget(0, t2.clone()), &mut s);
        // Move first item down
        reduce(&TagAction::MoveTarget(0, 0, 1), &mut s);
        assert_eq!(s.param_tags.tags[0].targets[0], t1);
        assert_eq!(s.param_tags.tags[0].targets[1], t0);
        assert_eq!(s.param_tags.tags[0].targets[2], t2);
    }
}
