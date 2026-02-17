use std::collections::VecDeque;
use std::time::Instant;

use super::instrument::Track;
use super::{SessionState, TrackState};
use crate::action::{
    BusAction, DomainAction, MixerAction, SequencerAction, SessionAction, TrackAction,
    VstParamAction,
};
use imbolc_types::TagAction;
use imbolc_types::TrackId;

/// What scope of state an undo entry covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoScope {
    /// Only one instrument changed (most common — parameter tweaks).
    SingleInstrument(TrackId),
    /// The instrument collection changed (add/remove would use Full instead,
    /// but this is available as a defensive fallback).
    Instruments,
    /// Only session state changed (piano roll, automation, mixer buses, etc.).
    Session,
    /// Both session and instruments changed (add/remove instrument, bus add/remove, etc.).
    Full,
}

/// Identifies a gesture for undo coalescing. Sequential actions with the same
/// key within `COALESCE_WINDOW` share a single undo snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalesceKey {
    /// Parameter tweaks on the same instrument (filter, LFO, envelope, effects, etc.)
    InstrumentParam(TrackId),
    /// Session-level parameter tweaks (BPM, master level, humanize, etc.)
    SessionParam,
    /// No coalescing — structural changes always get their own snapshot.
    None,
}

/// Maximum time between coalesced actions (500ms).
const COALESCE_WINDOW: std::time::Duration = std::time::Duration::from_millis(500);

/// A single undo/redo entry storing only the state that was affected.
enum UndoEntry {
    SingleInstrument {
        id: TrackId,
        instrument: Box<Track>,
    },
    Instruments(Box<TrackState>),
    Session(Box<SessionState>),
    Full {
        session: Box<SessionState>,
        instruments: Box<TrackState>,
    },
}

pub struct UndoHistory {
    undo_stack: VecDeque<UndoEntry>,
    redo_stack: VecDeque<UndoEntry>,
    max_depth: usize,
    last_coalesce_key: CoalesceKey,
    last_push_time: Instant,
}

impl UndoHistory {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_depth,
            last_coalesce_key: CoalesceKey::None,
            last_push_time: Instant::now(),
        }
    }

    /// Push a scoped snapshot before mutating state.
    pub fn push_scoped(
        &mut self,
        scope: UndoScope,
        session: &SessionState,
        instruments: &TrackState,
    ) {
        let entry = match scope {
            UndoScope::SingleInstrument(id) => {
                match instruments.track(id) {
                    Some(inst) => UndoEntry::SingleInstrument {
                        id,
                        instrument: Box::new(inst.clone()),
                    },
                    // Track not found — fall back to full instruments snapshot
                    None => UndoEntry::Instruments(Box::new(instruments.clone())),
                }
            }
            UndoScope::Instruments => UndoEntry::Instruments(Box::new(instruments.clone())),
            UndoScope::Session => UndoEntry::Session(Box::new(session.clone())),
            UndoScope::Full => UndoEntry::Full {
                session: Box::new(session.clone()),
                instruments: Box::new(instruments.clone()),
            },
        };

        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(entry);
        self.redo_stack.clear();
    }

    /// Push a snapshot from owned values (used by automation.rs when starting recording).
    pub fn push_from(&mut self, session: SessionState, instruments: TrackState) {
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(UndoEntry::Full {
            session: Box::new(session),
            instruments: Box::new(instruments),
        });
        self.redo_stack.clear();
    }

    /// Push a scoped snapshot with coalescing support. If `key` matches the
    /// previous push's key and less than `COALESCE_WINDOW` has elapsed, the
    /// push is skipped — keeping the pre-gesture snapshot already on the stack.
    pub fn push_coalesced(
        &mut self,
        scope: UndoScope,
        session: &SessionState,
        instruments: &TrackState,
        key: CoalesceKey,
    ) {
        let now = Instant::now();
        if key != CoalesceKey::None
            && key == self.last_coalesce_key
            && now.duration_since(self.last_push_time) < COALESCE_WINDOW
        {
            // Same gesture, within window — skip the push to keep the
            // original pre-gesture snapshot on the stack.
            self.last_push_time = now;
            return;
        }
        self.push_scoped(scope, session, instruments);
        self.last_coalesce_key = key;
        self.last_push_time = now;
    }

    fn clear_coalesce(&mut self) {
        self.last_coalesce_key = CoalesceKey::None;
    }

    /// Undo: pop from undo stack, create inverse from current state, apply stored entry.
    /// Returns the scope of the undone entry, or `None` if nothing to undo.
    pub fn undo(
        &mut self,
        session: &mut SessionState,
        instruments: &mut TrackState,
    ) -> Option<UndoScope> {
        self.clear_coalesce();
        let entry = self.undo_stack.pop_back()?;
        let scope = entry_scope(&entry);
        let inverse = create_inverse(&entry, session, instruments);
        apply_entry(entry, session, instruments);
        self.redo_stack.push_back(inverse);
        Some(scope)
    }

    /// Redo: pop from redo stack, create inverse from current state, apply stored entry.
    /// Returns the scope of the redone entry, or `None` if nothing to redo.
    pub fn redo(
        &mut self,
        session: &mut SessionState,
        instruments: &mut TrackState,
    ) -> Option<UndoScope> {
        self.clear_coalesce();
        let entry = self.redo_stack.pop_back()?;
        let scope = entry_scope(&entry);
        let inverse = create_inverse(&entry, session, instruments);
        apply_entry(entry, session, instruments);
        self.undo_stack.push_back(inverse);
        Some(scope)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.clear_coalesce();
    }
}

/// Map an undo entry back to its scope.
fn entry_scope(entry: &UndoEntry) -> UndoScope {
    match entry {
        UndoEntry::SingleInstrument { id, .. } => UndoScope::SingleInstrument(*id),
        UndoEntry::Instruments(_) => UndoScope::Instruments,
        UndoEntry::Session(_) => UndoScope::Session,
        UndoEntry::Full { .. } => UndoScope::Full,
    }
}

/// Create an inverse entry by snapshotting the *current* state at the same scope.
/// If the entry is SingleInstrument but the instrument no longer exists,
/// defensively escalate to Instruments scope.
fn create_inverse(
    entry: &UndoEntry,
    session: &SessionState,
    instruments: &TrackState,
) -> UndoEntry {
    match entry {
        UndoEntry::SingleInstrument { id, .. } => {
            match instruments.track(*id) {
                Some(inst) => UndoEntry::SingleInstrument {
                    id: *id,
                    instrument: Box::new(inst.clone()),
                },
                // Track was deleted between push and undo — snapshot everything
                None => UndoEntry::Instruments(Box::new(instruments.clone())),
            }
        }
        UndoEntry::Instruments(_) => UndoEntry::Instruments(Box::new(instruments.clone())),
        UndoEntry::Session(_) => UndoEntry::Session(Box::new(session.clone())),
        UndoEntry::Full { .. } => UndoEntry::Full {
            session: Box::new(session.clone()),
            instruments: Box::new(instruments.clone()),
        },
    }
}

/// Apply a stored entry onto the live state.
fn apply_entry(entry: UndoEntry, session: &mut SessionState, instruments: &mut TrackState) {
    match entry {
        UndoEntry::SingleInstrument { id, instrument } => {
            if let Some(live) = instruments.track_mut(id) {
                *live = *instrument;
            }
            // If instrument not found (shouldn't happen due to Full escalation),
            // silently skip — the state is already consistent.
        }
        UndoEntry::Instruments(stored) => {
            *instruments = *stored;
            instruments.rebuild_index();
        }
        UndoEntry::Session(stored) => {
            *session = *stored;
        }
        UndoEntry::Full {
            session: s,
            instruments: i,
        } => {
            *session = *s;
            *instruments = *i;
            instruments.rebuild_index();
        }
    }
}

/// Determine the undo scope for an action based on what state it will touch.
/// `automation_recording` should be true only when automation is actively being
/// recorded during playback — this escalates param tweaks to Full scope so the
/// automation lane changes are also captured.
pub fn undo_scope(
    action: &DomainAction,
    session: &SessionState,
    instruments: &TrackState,
    automation_recording: bool,
) -> UndoScope {
    let recording = automation_recording;

    match action {
        // Track add/delete always touch both state trees
        DomainAction::Track(TrackAction::Add(_)) | DomainAction::Track(TrackAction::Delete(_)) => {
            UndoScope::Full
        }

        // Layer link/unlink modifies instruments + session.mixer.layer_group_mixers
        DomainAction::Track(TrackAction::LinkLayer(_, _))
        | DomainAction::Track(TrackAction::UnlinkLayer(_)) => UndoScope::Full,

        // Track Update carries an explicit id
        DomainAction::Track(TrackAction::Update(update)) => {
            if recording {
                UndoScope::Full
            } else {
                UndoScope::SingleInstrument(update.id)
            }
        }

        // Other instrument actions — use target_track_id()
        DomainAction::Track(a) => {
            match a.target_track_id() {
                Some(id) => {
                    if recording {
                        UndoScope::Full
                    } else {
                        UndoScope::SingleInstrument(id)
                    }
                }
                // No target ID (shouldn't happen for undoable actions, but be safe)
                None => UndoScope::Full,
            }
        }

        // Mixer actions: depends on what's selected
        DomainAction::Mixer(a) => mixer_scope(a, session, instruments, recording),

        // Session-only domains
        DomainAction::PianoRoll(_)
        | DomainAction::Automation(_)
        | DomainAction::Arrangement(_)
        | DomainAction::Session(_)
        | DomainAction::Midi(_)
        | DomainAction::Generative(_)
        | DomainAction::Tag(_) => UndoScope::Session,

        // Bus add/remove syncs instrument sends — touches both trees
        DomainAction::Bus(BusAction::Add | BusAction::Remove(_)) => UndoScope::Full,
        DomainAction::Bus(BusAction::Rename(_, _)) => UndoScope::Session,

        // Sequencer/Chopper operate on the selected instrument's drum sequencer
        DomainAction::Sequencer(_) | DomainAction::Chopper(_) => {
            match instruments.selected_track() {
                Some(inst) => UndoScope::SingleInstrument(inst.id),
                None => UndoScope::Full,
            }
        }

        // VstParam carries TrackId as first field
        DomainAction::VstParam(a) => {
            let id = match a {
                VstParamAction::SetParam(id, _, _, _)
                | VstParamAction::AdjustParam(id, _, _, _)
                | VstParamAction::ResetParam(id, _, _)
                | VstParamAction::DiscoverParams(id, _)
                | VstParamAction::SaveState(id, _) => *id,
            };
            if recording {
                UndoScope::Full
            } else {
                UndoScope::SingleInstrument(id)
            }
        }

        // Everything else (non-undoable actions shouldn't reach here, but be safe)
        _ => UndoScope::Full,
    }
}

/// Determine mixer action scope based on selection target.
fn mixer_scope(
    _action: &MixerAction,
    session: &SessionState,
    instruments: &TrackState,
    recording: bool,
) -> UndoScope {
    match session.mixer.selection {
        super::session::MixerSelection::Track(idx) => match instruments.tracks.get(idx) {
            Some(inst) => {
                if recording {
                    UndoScope::Full
                } else {
                    UndoScope::SingleInstrument(inst.id)
                }
            }
            None => UndoScope::Full,
        },
        // Bus, LayerGroup, Master selections all live in SessionState
        super::session::MixerSelection::Bus(_)
        | super::session::MixerSelection::LayerGroup(_)
        | super::session::MixerSelection::Master => {
            if recording {
                UndoScope::Full
            } else {
                UndoScope::Session
            }
        }
    }
}

/// Map an action to a coalesce key. Actions with the same key that arrive
/// within `COALESCE_WINDOW` share a single undo snapshot.
pub fn coalesce_key(
    action: &DomainAction,
    session: &SessionState,
    instruments: &TrackState,
) -> CoalesceKey {
    match action {
        // Track parameter tweaks — coalesce by instrument ID
        DomainAction::Track(
            TrackAction::AdjustFilterCutoff(id, _)
            | TrackAction::AdjustFilterResonance(id, _)
            | TrackAction::AdjustEffectParam(id, _, _, _)
            | TrackAction::AdjustLfoRate(id, _)
            | TrackAction::AdjustLfoDepth(id, _)
            | TrackAction::AdjustEnvelopeAttack(id, _)
            | TrackAction::AdjustEnvelopeDecay(id, _)
            | TrackAction::AdjustEnvelopeSustain(id, _)
            | TrackAction::AdjustEnvelopeRelease(id, _)
            | TrackAction::AdjustArpOctaves(id, _)
            | TrackAction::AdjustArpGate(id, _)
            | TrackAction::AdjustLayerOctaveOffset(id, _)
            | TrackAction::AdjustTrackSwing(id, _)
            | TrackAction::AdjustTrackHumanizeVelocity(id, _)
            | TrackAction::AdjustTrackHumanizeTiming(id, _)
            | TrackAction::AdjustTrackTimingOffset(id, _),
        ) => CoalesceKey::InstrumentParam(*id),
        DomainAction::Track(_) => CoalesceKey::None,

        // Mixer level/pan/send — coalesce by mixer selection target
        DomainAction::Mixer(
            MixerAction::AdjustLevel(_) | MixerAction::AdjustPan(_) | MixerAction::AdjustSend(_, _),
        ) => match session.mixer.selection {
            super::session::MixerSelection::Track(idx) => match instruments.tracks.get(idx) {
                Some(inst) => CoalesceKey::InstrumentParam(inst.id),
                None => CoalesceKey::None,
            },
            _ => CoalesceKey::SessionParam,
        },
        DomainAction::Mixer(_) => CoalesceKey::None,

        // VST param tweaks
        DomainAction::VstParam(
            VstParamAction::SetParam(id, _, _, _) | VstParamAction::AdjustParam(id, _, _, _),
        ) => CoalesceKey::InstrumentParam(*id),
        DomainAction::VstParam(_) => CoalesceKey::None,

        // Sequencer continuous adjustments — operate on selected instrument
        DomainAction::Sequencer(
            SequencerAction::AdjustVelocity(_, _, _)
            | SequencerAction::AdjustPadLevel(_, _)
            | SequencerAction::AdjustSwing(_)
            | SequencerAction::AdjustProbability(_, _, _)
            | SequencerAction::AdjustPadPitch(_, _)
            | SequencerAction::AdjustStepPitch(_, _, _),
        ) => match instruments.selected_track() {
            Some(inst) => CoalesceKey::InstrumentParam(inst.id),
            None => CoalesceKey::None,
        },
        DomainAction::Sequencer(_) => CoalesceKey::None,

        // Session-level adjustments
        DomainAction::Session(
            SessionAction::AdjustHumanizeVelocity(_) | SessionAction::AdjustHumanizeTiming(_),
        ) => CoalesceKey::SessionParam,
        DomainAction::Session(_) => CoalesceKey::None,

        // Generative macro adjustments coalesce
        DomainAction::Generative(
            imbolc_types::GenerativeAction::AdjustDensity(_)
            | imbolc_types::GenerativeAction::AdjustChaos(_)
            | imbolc_types::GenerativeAction::AdjustEnergy(_)
            | imbolc_types::GenerativeAction::AdjustMotion(_)
            | imbolc_types::GenerativeAction::AdjustHumanizeTiming(_)
            | imbolc_types::GenerativeAction::AdjustHumanizeVelocity(_),
        ) => CoalesceKey::SessionParam,
        DomainAction::Generative(_) => CoalesceKey::None,

        DomainAction::Tag(_) => CoalesceKey::None,

        // Everything else — no coalescing
        _ => CoalesceKey::None,
    }
}

pub fn is_undoable(action: &DomainAction) -> bool {
    match action {
        DomainAction::Track(a) => !matches!(
            a,
            TrackAction::PlayNote(_, _)
                | TrackAction::PlayNotes(_, _)
                | TrackAction::PlayDrumPad(_, _)
                | TrackAction::Select(_)
                | TrackAction::SelectNext
                | TrackAction::SelectPrev
                | TrackAction::SelectFirst
                | TrackAction::SelectLast
                | TrackAction::Edit(_)
                | TrackAction::OpenVstEffectParams(_, _)
        ),
        DomainAction::Mixer(a) => !matches!(
            a,
            MixerAction::Move(_)
                | MixerAction::Jump(_)
                | MixerAction::SelectAt(_)
                | MixerAction::CycleSection
        ),
        DomainAction::PianoRoll(a) => matches!(
            a,
            crate::action::PianoRollAction::ToggleNote { .. }
                | crate::action::PianoRollAction::ToggleLoop
                | crate::action::PianoRollAction::SetLoopStart(_)
                | crate::action::PianoRollAction::SetLoopEnd(_)
                | crate::action::PianoRollAction::CycleTimeSig
                | crate::action::PianoRollAction::TogglePolyMode(_)
                | crate::action::PianoRollAction::AdjustSwing(_)
                | crate::action::PianoRollAction::DeleteNotesInRegion { .. }
                | crate::action::PianoRollAction::PasteNotes { .. }
        ),
        DomainAction::Session(a) => !matches!(
            a,
            SessionAction::Save
                | SessionAction::SaveAs(_)
                | SessionAction::Load
                | SessionAction::LoadFrom(_)
                | SessionAction::NewProject
                | SessionAction::OpenFileBrowser(_)
        ),
        DomainAction::Sequencer(a) => !matches!(
            a,
            SequencerAction::PlayStop
                | SequencerAction::LoadSample(_)
                | SequencerAction::LoadSampleResult(_, _)
                | SequencerAction::CopySteps { .. }
        ),
        DomainAction::Chopper(a) => !matches!(
            a,
            crate::action::ChopperAction::LoadSample
                | crate::action::ChopperAction::LoadSampleResult(_)
                | crate::action::ChopperAction::PreviewSlice
                | crate::action::ChopperAction::SelectSlice(_)
                | crate::action::ChopperAction::MoveCursor(_)
        ),
        DomainAction::Automation(a) => !matches!(
            a,
            crate::action::AutomationAction::SelectLane(_)
                | crate::action::AutomationAction::ToggleRecording
                | crate::action::AutomationAction::ToggleLaneArm(_)
                | crate::action::AutomationAction::ArmAllLanes
                | crate::action::AutomationAction::DisarmAllLanes
                | crate::action::AutomationAction::RecordValue(_, _)
                | crate::action::AutomationAction::CopyPoints(_, _, _)
        ),
        DomainAction::Midi(a) => !matches!(
            a,
            crate::action::MidiAction::ConnectPort(_)
                | crate::action::MidiAction::DisconnectPort
                | crate::action::MidiAction::StartLearn(_)
                | crate::action::MidiAction::CancelLearn
        ),
        DomainAction::Arrangement(a) => !matches!(
            a,
            crate::action::ArrangementAction::TogglePlayMode
                | crate::action::ArrangementAction::SelectPlacement(_)
                | crate::action::ArrangementAction::SelectLane(_)
                | crate::action::ArrangementAction::MoveCursor(_)
                | crate::action::ArrangementAction::ScrollView(_)
                | crate::action::ArrangementAction::PlayStop
        ),
        DomainAction::VstParam(a) => matches!(
            a,
            VstParamAction::SetParam(_, _, _, _)
                | VstParamAction::AdjustParam(_, _, _, _)
                | VstParamAction::ResetParam(_, _, _)
        ),
        DomainAction::Generative(_) => true,
        DomainAction::Tag(a) => {
            !matches!(a, TagAction::SelectTag(_) | TagAction::SetPendingTarget(_))
        }
        DomainAction::Undo | DomainAction::Redo => false,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SessionState;
    use crate::state::TrackState;
    use imbolc_types::{BusId, SourceType};

    #[test]
    fn test_undo_push_pop() {
        let mut history = UndoHistory::new(5);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        assert!(!history.can_undo());

        history.push_scoped(UndoScope::Full, &session, &instruments);
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_stack.len(), 1);

        let undone = history.undo(&mut session, &mut instruments);
        assert!(undone.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_redo() {
        let mut history = UndoHistory::new(5);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        // Initial state
        session.mixer.master_level = 1.0;
        history.push_scoped(UndoScope::Full, &session, &instruments);

        // Modify state
        session.mixer.master_level = 0.5;

        // Undo — should restore master_level to 1.0
        assert!(history.undo(&mut session, &mut instruments).is_some());
        assert_eq!(session.mixer.master_level, 1.0);

        // Redo — should restore master_level to 0.5
        assert!(history.redo(&mut session, &mut instruments).is_some());
        assert_eq!(session.mixer.master_level, 0.5);
    }

    #[test]
    fn test_max_depth() {
        let mut history = UndoHistory::new(2);
        let session = SessionState::new();
        let instruments = TrackState::new();

        history.push_scoped(UndoScope::Full, &session, &instruments);
        history.push_scoped(UndoScope::Full, &session, &instruments);
        history.push_scoped(UndoScope::Full, &session, &instruments);

        assert_eq!(history.undo_stack.len(), 2);
    }

    #[test]
    fn test_push_clears_redo() {
        let mut history = UndoHistory::new(5);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        history.push_scoped(UndoScope::Full, &session, &instruments);
        history.undo(&mut session, &mut instruments);
        assert!(history.can_redo());

        history.push_scoped(UndoScope::Full, &session, &instruments);
        assert!(!history.can_redo());
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut history = UndoHistory::new(5);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        history.push_scoped(UndoScope::Full, &session, &instruments);
        history.push_scoped(UndoScope::Full, &session, &instruments);
        history.undo(&mut session, &mut instruments);
        assert!(history.can_undo());
        assert!(history.can_redo());

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn push_from_owned_works() {
        let mut history = UndoHistory::new(5);
        let session = SessionState::new();
        let instruments = TrackState::new();

        history.push_from(session.clone(), instruments.clone());
        assert!(history.can_undo());
        assert_eq!(history.undo_stack.len(), 1);
    }

    #[test]
    fn undo_empty_returns_none() {
        let mut history = UndoHistory::new(5);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();
        assert!(history.undo(&mut session, &mut instruments).is_none());
    }

    #[test]
    fn redo_empty_returns_none() {
        let mut history = UndoHistory::new(5);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();
        assert!(history.redo(&mut session, &mut instruments).is_none());
    }

    #[test]
    fn is_undoable_instrument_add() {
        let action = DomainAction::Track(TrackAction::Add(SourceType::Saw));
        assert!(is_undoable(&action));
    }

    #[test]
    fn is_undoable_select_is_not() {
        let action = DomainAction::Track(TrackAction::Select(0));
        assert!(!is_undoable(&action));
    }

    // --- New scope-aware tests ---

    #[test]
    fn test_single_instrument_scope() {
        let mut history = UndoHistory::new(10);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        let id1 = instruments.add_track(SourceType::Saw);
        let id2 = instruments.add_track(SourceType::Sin);

        // Snapshot instrument 1 before modifying
        history.push_scoped(UndoScope::SingleInstrument(id1), &session, &instruments);

        // Modify instrument 1's level
        instruments.track_mut(id1).unwrap().channel_strip.level = 0.3;
        // Also modify instrument 2 (should NOT be reverted)
        instruments.track_mut(id2).unwrap().channel_strip.level = 0.7;

        // Undo should only revert instrument 1
        assert!(history.undo(&mut session, &mut instruments).is_some());
        // Track 1 reverted to default (0.8)
        assert!(
            (instruments.track(id1).unwrap().channel_strip.level - 0.8).abs() < f32::EPSILON,
            "instrument 1 level should be reverted to 0.8, got {}",
            instruments.track(id1).unwrap().channel_strip.level
        );
        // Track 2 unchanged
        assert!(
            (instruments.track(id2).unwrap().channel_strip.level - 0.7).abs() < f32::EPSILON,
            "instrument 2 level should remain 0.7, got {}",
            instruments.track(id2).unwrap().channel_strip.level
        );
    }

    #[test]
    fn test_session_scope() {
        let mut history = UndoHistory::new(10);
        let mut session = SessionState::new();
        let mut instruments = TrackState::new();

        let id1 = instruments.add_track(SourceType::Saw);

        // Snapshot session before modifying
        history.push_scoped(UndoScope::Session, &session, &instruments);

        // Modify session
        session.mixer.master_level = 0.3;
        // Also modify an instrument (should NOT be reverted)
        instruments.track_mut(id1).unwrap().channel_strip.level = 0.1;

        // Undo should only revert session
        assert!(history.undo(&mut session, &mut instruments).is_some());
        assert!(
            (session.mixer.master_level - 1.0).abs() < f32::EPSILON,
            "master_level should be reverted to 1.0, got {}",
            session.mixer.master_level
        );
        // Track unchanged
        assert!(
            (instruments.track(id1).unwrap().channel_strip.level - 0.1).abs() < f32::EPSILON,
            "instrument level should remain 0.1, got {}",
            instruments.track(id1).unwrap().channel_strip.level
        );
    }

    #[test]
    fn test_scope_classification() {
        let session = SessionState::new();
        let mut instruments = TrackState::new();
        let id1 = instruments.add_track(SourceType::Saw);

        // Track Add => Full
        let action = DomainAction::Track(TrackAction::Add(SourceType::Saw));
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::Full
        );

        // Track Delete => Full
        let action = DomainAction::Track(TrackAction::Delete(id1));
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::Full
        );

        // Track param tweak => SingleInstrument (no automation recording)
        let action = DomainAction::Track(TrackAction::AdjustFilterCutoff(id1, 0.1));
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::SingleInstrument(id1)
        );

        // Track param tweak => Full (automation recording active)
        assert_eq!(
            undo_scope(&action, &session, &instruments, true),
            UndoScope::Full
        );

        // PianoRoll => Session
        let action = DomainAction::PianoRoll(crate::action::PianoRollAction::ToggleLoop);
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::Session
        );

        // Bus Add => Full
        let action = DomainAction::Bus(BusAction::Add);
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::Full
        );

        // Bus Rename => Session
        let action = DomainAction::Bus(BusAction::Rename(BusId::new(1), "Test".to_string()));
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::Session
        );

        // Sequencer (with selected instrument) => SingleInstrument
        instruments.selected = Some(0);
        let action = DomainAction::Sequencer(crate::action::SequencerAction::ToggleStep(0, 0));
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::SingleInstrument(id1)
        );

        // VstParam => SingleInstrument
        let action = DomainAction::VstParam(VstParamAction::SetParam(
            id1,
            crate::action::VstTarget::Source,
            0,
            0.5,
        ));
        assert_eq!(
            undo_scope(&action, &session, &instruments, false),
            UndoScope::SingleInstrument(id1)
        );
    }
}
