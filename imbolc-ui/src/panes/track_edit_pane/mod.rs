mod editing;
mod input;
mod rendering;

use std::any::Any;

use crate::state::{
    track::{track_row_count, track_row_info, track_section_for_row},
    AppState, EnvConfig, LfoConfig, Param, SourceType, TrackId, TrackSection,
};
use crate::ui::action_id::ActionId;
use crate::ui::performance::PerformanceController;
use crate::ui::widgets::TextInput;
use crate::ui::{
    Action, InputEvent, Keymap, MouseEvent, Pane, PaneIdStr, Rect, RenderBuf, ToggleResult,
};
use imbolc_types::{ChannelConfig, ProcessingStage};

pub struct TrackEditPane {
    keymap: Keymap,
    instrument_id: Option<TrackId>,
    instrument_name: String,
    source: SourceType,
    source_params: Vec<Param>,
    sample_name: Option<String>,
    processing_chain: Vec<ProcessingStage>,
    lfo: LfoConfig,
    amp_envelope: EnvConfig,
    polyphonic: bool,
    active: bool,
    channel_config: ChannelConfig,
    pub(crate) selected_row: usize,
    pub(crate) scroll_offset: usize,
    editing: bool,
    edit_input: TextInput,
    edit_backup_value: Option<String>,
    perf: PerformanceController,
}

impl TrackEditPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            instrument_id: None,
            instrument_name: String::new(),
            source: SourceType::Saw,
            source_params: Vec::new(),
            sample_name: None,
            processing_chain: Vec::new(),
            lfo: LfoConfig::default(),
            amp_envelope: EnvConfig::default(),
            polyphonic: true,
            active: true,
            channel_config: ChannelConfig::default(),
            selected_row: 0,
            scroll_offset: 0,
            editing: false,
            edit_input: TextInput::new(""),
            edit_backup_value: None,
            perf: PerformanceController::new(),
        }
    }

    pub fn set_enhanced_keyboard(&mut self, enabled: bool) {
        self.perf.set_enhanced_keyboard(enabled);
    }

    pub fn set_instrument(&mut self, instrument: &crate::state::Track) {
        self.instrument_id = Some(instrument.id);
        self.instrument_name = instrument.name.clone();
        self.source = instrument.source;
        self.source_params = instrument.source_params.clone();
        self.sample_name = instrument
            .sampler_config()
            .and_then(|c| c.sample_name.clone());
        self.processing_chain = instrument.channel_strip.processing_chain.clone();
        self.lfo = instrument.modulation.lfo.clone();
        self.amp_envelope = instrument.modulation.amp_envelope.clone();
        self.polyphonic = instrument.polyphonic;
        self.active = instrument.channel_strip.active;
        self.channel_config = instrument.channel_strip.channel_config;
        self.selected_row = 0;
        self.scroll_offset = 0;
    }

    /// Re-sync data from an instrument without resetting cursor position.
    /// Used when returning from a sub-pane (e.g. add_effect) where the same
    /// instrument may have changed.
    fn refresh_instrument(&mut self, instrument: &crate::state::Track) {
        self.instrument_id = Some(instrument.id);
        self.instrument_name = instrument.name.clone();
        self.source = instrument.source;
        self.source_params = instrument.source_params.clone();
        self.sample_name = instrument
            .sampler_config()
            .and_then(|c| c.sample_name.clone());
        self.processing_chain = instrument.channel_strip.processing_chain.clone();
        self.lfo = instrument.modulation.lfo.clone();
        self.amp_envelope = instrument.modulation.amp_envelope.clone();
        self.polyphonic = instrument.polyphonic;
        self.active = instrument.channel_strip.active;
        self.channel_config = instrument.channel_strip.channel_config;
        // Clamp selected_row to valid range (chain may have changed)
        let max = self.total_rows().saturating_sub(1);
        self.selected_row = self.selected_row.min(max);
    }

    #[allow(dead_code)]
    pub fn instrument_id(&self) -> Option<TrackId> {
        self.instrument_id
    }

    /// Get current tab as index (for view state).
    /// Dynamic encoding: 0=Source, 1..=N=Processing(0..N-1), N+1=Lfo, N+2=Envelope.
    pub fn tab_index(&self) -> u8 {
        let n = self.processing_chain.len();
        match self.current_section() {
            TrackSection::Source => 0,
            TrackSection::Processing(i) => (i + 1) as u8,
            TrackSection::Lfo => (n + 1) as u8,
            TrackSection::Envelope => (n + 2) as u8,
        }
    }

    /// Set tab from index (for view state restoration).
    /// Dynamic decoding: 0=Source, 1..=N=Processing(0..N-1), N+1=Lfo, N+2=Envelope.
    pub fn set_tab_index(&mut self, idx: u8) {
        let n = self.processing_chain.len();
        let target = if idx == 0 {
            TrackSection::Source
        } else if (idx as usize) <= n {
            TrackSection::Processing(idx as usize - 1)
        } else if idx as usize == n + 1 {
            TrackSection::Lfo
        } else {
            TrackSection::Envelope
        };
        for i in 0..self.total_rows() {
            if self.section_for_row(i) == target {
                self.selected_row = i;
                break;
            }
        }
    }

    /// Apply edits back to an instrument
    #[allow(dead_code)]
    pub fn apply_to(&self, instrument: &mut crate::state::Track) {
        instrument.source = self.source;
        instrument.source_params = self.source_params.clone();
        instrument.channel_strip.processing_chain = self.processing_chain.clone();
        instrument.modulation.lfo = self.lfo.clone();
        instrument.modulation.amp_envelope = self.amp_envelope.clone();
        instrument.polyphonic = self.polyphonic;
        instrument.channel_strip.active = self.active;
    }

    /// Total number of selectable rows across all sections
    fn total_rows(&self) -> usize {
        track_row_count(self.source, &self.source_params, &self.processing_chain)
    }

    /// Calculate non-selectable visual lines (headers + separators)
    fn visual_overhead(&self) -> usize {
        // Headers: 1 (source) + one per filter in chain + 1 (LFO) + (1 if !VST for envelope)
        let filter_count = self
            .processing_chain
            .iter()
            .filter(|s| s.is_filter())
            .count();
        let headers = 1 + filter_count + 1 + if self.source.is_vst() { 0 } else { 1 };
        // Separators: 1 (after source) + 1 (after chain) + (1 after LFO if !VST)
        let separators = 1 + 1 + if self.source.is_vst() { 0 } else { 1 };
        headers + separators
    }

    /// Calculate scroll offset to keep selected_row visible
    fn calc_scroll_offset(selected: usize, total: usize, visible: usize) -> usize {
        if visible == 0 || total <= visible {
            0
        } else if selected >= visible {
            (selected - visible + 1).min(total.saturating_sub(visible))
        } else {
            0
        }
    }

    /// Which section does a given row belong to?
    fn section_for_row(&self, row: usize) -> TrackSection {
        track_section_for_row(
            row,
            self.source,
            &self.source_params,
            &self.processing_chain,
        )
    }

    /// Get section and local index for a row
    fn row_info(&self, row: usize) -> (TrackSection, usize) {
        track_row_info(
            row,
            self.source,
            &self.source_params,
            &self.processing_chain,
        )
    }

    fn current_section(&self) -> TrackSection {
        self.section_for_row(self.selected_row)
    }

    /// Find the first row belonging to a given processing stage chain index,
    /// offset by local_idx within that stage. Used for cursor stability after MoveStage.
    fn row_for_processing_stage(&self, chain_idx: usize, local_idx: usize) -> usize {
        let source_rows = (if self.source.is_sample() || self.source.is_time_stretch() {
            1
        } else {
            0
        }) + self.source_params.len().max(1);
        let chain_rows_before: usize = self.processing_chain[..chain_idx]
            .iter()
            .map(|s| s.row_count())
            .sum();
        source_rows + chain_rows_before + local_idx
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    /// Build an AutomationTarget for the currently selected row.
    fn build_automation_target(&self, id: TrackId) -> Option<imbolc_types::AutomationTarget> {
        use imbolc_types::{
            AutomationTarget, ParamIndex, ParameterTarget, ProcessingStage, TrackParameter,
        };
        let (section, local_idx) = self.row_info(self.selected_row);
        let param = match section {
            TrackSection::Source => {
                let param_idx = if self.source.is_sample() {
                    if local_idx == 0 {
                        return None; // sample file row, not a parameter
                    }
                    local_idx - 1
                } else {
                    local_idx
                };
                ParameterTarget::SourceParam(ParamIndex::new(param_idx))
            }
            TrackSection::Processing(i) => {
                match self.processing_chain.get(i)? {
                    ProcessingStage::Filter(f) => match local_idx {
                        0 => return None, // filter type header
                        1 => ParameterTarget::FilterCutoff,
                        2 => ParameterTarget::FilterResonance,
                        idx => {
                            let extra_idx = idx - 3;
                            if extra_idx < f.extra_params.len() {
                                // Extra filter params don't have dedicated ParameterTargets
                                return None;
                            }
                            return None;
                        }
                    },
                    ProcessingStage::Eq(..) => return None,
                    ProcessingStage::Effect(e) => {
                        if local_idx == 0 {
                            return None; // effect header
                        }
                        let param_idx = local_idx - 1;
                        if param_idx < e.params.len() {
                            ParameterTarget::EffectParam(e.id, ParamIndex::new(param_idx))
                        } else {
                            return None;
                        }
                    }
                }
            }
            TrackSection::Lfo => match local_idx {
                0 => ParameterTarget::LfoRate,
                1 => ParameterTarget::LfoDepth,
                _ => return None,
            },
            TrackSection::Envelope => match local_idx {
                0 => ParameterTarget::Attack,
                1 => ParameterTarget::Decay,
                2 => ParameterTarget::Sustain,
                3 => ParameterTarget::Release,
                _ => return None,
            },
        };
        Some(AutomationTarget::Track(id, TrackParameter::Standard(param)))
    }
}

impl Pane for TrackEditPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("instrument_edit")
    }

    fn handle_action(&mut self, action: ActionId, event: &InputEvent, state: &AppState) -> Action {
        self.handle_action_impl(action, event, state)
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        self.handle_raw_input_impl(event);
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        self.render_impl(area, buf, state);
    }

    fn handle_mouse(&mut self, event: &MouseEvent, _area: Rect, _state: &AppState) -> Action {
        self.handle_mouse_impl(event)
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn tick(&mut self, state: &AppState) -> Vec<Action> {
        let instrument_id = state
            .tracks
            .selected_track()
            .map(|inst| inst.id)
            .unwrap_or(TrackId::new(0));
        self.perf.tick_releases(instrument_id)
    }

    fn on_enter(&mut self, state: &AppState) {
        if let Some(inst) = state.tracks.selected_track() {
            if self.instrument_id == Some(inst.id) {
                self.refresh_instrument(inst);
            } else {
                self.set_instrument(inst);
            }
        }
    }

    fn toggle_performance_mode(&mut self, state: &AppState) -> ToggleResult {
        let is_kit = state
            .tracks
            .selected_track()
            .is_some_and(|s| s.source.is_kit());
        self.perf.toggle(is_kit)
    }

    fn activate_piano(&mut self) {
        self.perf.activate_piano();
    }

    fn activate_pad(&mut self) {
        self.perf.activate_pad();
    }

    fn deactivate_performance(&mut self) {
        self.perf.deactivate();
    }

    fn supports_performance_mode(&self) -> bool {
        true
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Default for TrackEditPane {
    fn default() -> Self {
        Self::new(Keymap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{EffectSlot, EffectType, FilterConfig, FilterType};
    use crate::ui::action_id::{ActionId, TrackEditActionId};
    use crate::ui::input::{InputEvent, KeyCode, Modifiers};
    use imbolc_types::{EffectId, TrackId};

    fn make_pane_with_chain(chain: Vec<ProcessingStage>) -> TrackEditPane {
        TrackEditPane {
            instrument_id: Some(TrackId::new(1)),
            source: SourceType::Saw,
            source_params: SourceType::Saw.default_params(),
            processing_chain: chain,
            ..Default::default()
        }
    }

    fn dummy_event() -> InputEvent {
        InputEvent::new(KeyCode::Char('x'), Modifiers::default())
    }

    #[test]
    fn test_section_navigation_reordered_chain() {
        // Chain: [Effect, Filter] — effect comes first
        let effect = EffectSlot::new(EffectId::new(0), EffectType::Delay);
        let filter = FilterConfig::new(FilterType::Lpf);
        let pane = make_pane_with_chain(vec![
            ProcessingStage::Effect(effect),
            ProcessingStage::Filter(filter),
        ]);

        // Source params for Saw: at least 1 row
        let source_rows = pane.source_params.len().max(1);

        // First processing row should be Processing(0) = Effect
        let row = source_rows;
        assert_eq!(pane.section_for_row(row), TrackSection::Processing(0));

        // Effect header + params, then filter starts at Processing(1)
        let effect_rows = 1 + EffectType::Delay.default_params().len();
        let filter_row = source_rows + effect_rows;
        assert_eq!(
            pane.section_for_row(filter_row),
            TrackSection::Processing(1)
        );
    }

    #[test]
    fn test_tab_cycling_visits_each_stage() {
        let effect = EffectSlot::new(EffectId::new(0), EffectType::Delay);
        let filter = FilterConfig::new(FilterType::Lpf);
        let mut pane = make_pane_with_chain(vec![
            ProcessingStage::Effect(effect),
            ProcessingStage::Filter(filter),
        ]);

        let state = AppState::new();
        let event = dummy_event();

        // Start at Source
        assert_eq!(pane.current_section(), TrackSection::Source);

        // Tab → Processing(0)
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::NextSection),
            &event,
            &state,
        );
        assert_eq!(pane.current_section(), TrackSection::Processing(0));

        // Tab → Processing(1)
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::NextSection),
            &event,
            &state,
        );
        assert_eq!(pane.current_section(), TrackSection::Processing(1));

        // Tab → Lfo
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::NextSection),
            &event,
            &state,
        );
        assert_eq!(pane.current_section(), TrackSection::Lfo);

        // Tab → Envelope
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::NextSection),
            &event,
            &state,
        );
        assert_eq!(pane.current_section(), TrackSection::Envelope);

        // Tab → Source (wrap)
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::NextSection),
            &event,
            &state,
        );
        assert_eq!(pane.current_section(), TrackSection::Source);
    }

    #[test]
    fn test_cursor_stability_after_move_stage() {
        let filter = FilterConfig::new(FilterType::Lpf);
        let effect = EffectSlot::new(EffectId::new(0), EffectType::Delay);
        let mut pane = make_pane_with_chain(vec![
            ProcessingStage::Filter(filter),
            ProcessingStage::Effect(effect),
        ]);

        let state = AppState::new();
        let event = dummy_event();

        // Navigate to filter's cutoff row (local_idx=1 within filter stage)
        let source_rows = pane.source_params.len().max(1);
        pane.selected_row = source_rows + 1; // Type=0, Cutoff=1
        assert_eq!(pane.current_section(), TrackSection::Processing(0));
        let (_, local_idx) = pane.row_info(pane.selected_row);
        assert_eq!(local_idx, 1); // cutoff

        // Move stage down: filter goes from index 0 to index 1
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::MoveStageDown),
            &event,
            &state,
        );

        // Filter is now at chain index 1, cursor should be on cutoff at new position
        assert_eq!(pane.current_section(), TrackSection::Processing(1));
        let (_, new_local) = pane.row_info(pane.selected_row);
        assert_eq!(new_local, 1); // still cutoff
    }

    #[test]
    fn test_toggle_filter_with_existing_chain() {
        let effect = EffectSlot::new(EffectId::new(0), EffectType::Delay);
        let mut pane = make_pane_with_chain(vec![ProcessingStage::Effect(effect)]);

        let state = AppState::new();
        let event = dummy_event();

        // Toggle on: should insert filter at index 0
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::ToggleFilter),
            &event,
            &state,
        );
        assert_eq!(pane.processing_chain.len(), 2);
        assert!(pane.processing_chain[0].is_filter());
        assert!(pane.processing_chain[1].is_effect());

        // Toggle off: should remove the filter
        pane.handle_action_impl(
            ActionId::TrackEdit(TrackEditActionId::ToggleFilter),
            &event,
            &state,
        );
        assert_eq!(pane.processing_chain.len(), 1);
        assert!(pane.processing_chain[0].is_effect());
    }

    #[test]
    fn test_empty_chain_placeholder() {
        let pane = make_pane_with_chain(vec![]);
        let total = pane.total_rows();
        let source_rows = pane.source_params.len().max(1);
        let lfo_rows = 4;
        let env_rows = 4;
        // Empty chain should contribute 1 placeholder row
        assert_eq!(total, source_rows + 1 + lfo_rows + env_rows);
    }
}
