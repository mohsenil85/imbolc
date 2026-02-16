//! Parameter tag types for grouping tagged parameters into performance views.

use serde::{Deserialize, Serialize};

use crate::{AutomationTarget, InstrumentId};

/// A named group of tagged parameters from any instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamTag {
    pub name: String,
    pub targets: Vec<AutomationTarget>,
}

impl ParamTag {
    pub fn new(name: String) -> Self {
        Self {
            name,
            targets: Vec::new(),
        }
    }

    /// Add a target, deduplicating.
    pub fn add_target(&mut self, target: AutomationTarget) {
        if !self.targets.contains(&target) {
            self.targets.push(target);
        }
    }

    /// Remove a target by value.
    pub fn remove_target(&mut self, target: &AutomationTarget) {
        self.targets.retain(|t| t != target);
    }

    /// Remove all targets referencing a given instrument.
    pub fn remove_instrument(&mut self, id: InstrumentId) {
        self.targets.retain(|t| t.instrument_id() != Some(id));
    }
}

/// Collection of parameter tags for a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamTagState {
    pub tags: Vec<ParamTag>,
    pub selected_tag: Option<usize>,
    /// Pending target for the tag picker modal to consume.
    #[serde(skip)]
    pub pending_target: Option<AutomationTarget>,
}

impl ParamTagState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn selected(&self) -> Option<&ParamTag> {
        self.selected_tag.and_then(|i| self.tags.get(i))
    }

    pub fn selected_mut(&mut self) -> Option<&mut ParamTag> {
        self.selected_tag.and_then(|i| self.tags.get_mut(i))
    }

    /// Remove all targets referencing a given instrument across all tags.
    pub fn remove_instrument(&mut self, id: InstrumentId) {
        for tag in &mut self.tags {
            tag.remove_instrument(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ParameterTarget;

    #[test]
    fn add_target_dedup() {
        let mut tag = ParamTag::new("test".into());
        let target = AutomationTarget::filter_cutoff(InstrumentId::new(1));
        tag.add_target(target.clone());
        tag.add_target(target.clone());
        assert_eq!(tag.targets.len(), 1);
    }

    #[test]
    fn remove_target() {
        let mut tag = ParamTag::new("test".into());
        let t1 = AutomationTarget::filter_cutoff(InstrumentId::new(1));
        let t2 = AutomationTarget::level(InstrumentId::new(1));
        tag.add_target(t1.clone());
        tag.add_target(t2.clone());
        tag.remove_target(&t1);
        assert_eq!(tag.targets.len(), 1);
        assert_eq!(tag.targets[0], t2);
    }

    #[test]
    fn remove_instrument_from_tag() {
        let mut tag = ParamTag::new("test".into());
        let id1 = InstrumentId::new(1);
        let id2 = InstrumentId::new(2);
        tag.add_target(AutomationTarget::filter_cutoff(id1));
        tag.add_target(AutomationTarget::level(id2));
        tag.remove_instrument(id1);
        assert_eq!(tag.targets.len(), 1);
        assert_eq!(tag.targets[0].instrument_id(), Some(id2));
    }

    #[test]
    fn remove_instrument_from_state() {
        let mut state = ParamTagState::new();
        let id1 = InstrumentId::new(1);
        let id2 = InstrumentId::new(2);
        state.tags.push(ParamTag::new("tag1".into()));
        state.tags.push(ParamTag::new("tag2".into()));
        state.tags[0].add_target(AutomationTarget::filter_cutoff(id1));
        state.tags[0].add_target(AutomationTarget::level(id2));
        state.tags[1].add_target(AutomationTarget::instrument(id1, ParameterTarget::Pan));
        state.remove_instrument(id1);
        assert_eq!(state.tags[0].targets.len(), 1);
        assert_eq!(state.tags[1].targets.len(), 0);
    }

    #[test]
    fn selected_tag() {
        let mut state = ParamTagState::new();
        state.tags.push(ParamTag::new("a".into()));
        assert!(state.selected().is_none());
        state.selected_tag = Some(0);
        assert_eq!(state.selected().unwrap().name, "a");
    }
}
