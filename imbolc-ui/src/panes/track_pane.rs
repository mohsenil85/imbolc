use std::any::Any;

use crate::state::arrangement::PlayMode;
use crate::state::{AppState, OwnershipDisplayStatus, SourceType};
use crate::ui::action_id::{ActionId, ModeActionId, TrackActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::performance::PerformanceController;
use crate::ui::widgets::TextInput;
use crate::ui::{
    translate_key, Action, ArrangementAction, Color, InputEvent, KeyCode, Keymap, MouseButton,
    MouseEvent, MouseEventKind, Palette, Pane, PaneIdStr, Rect, RenderBuf, Style, ToggleResult,
    TrackAction,
};
use imbolc_types::{GroupAction, GroupId, TrackId};

fn source_color(source: SourceType, p: &Palette) -> Color {
    match source {
        // Oscillators and synths
        SourceType::Saw | SourceType::Sin | SourceType::Sqr | SourceType::Tri
        | SourceType::Noise | SourceType::Pulse | SourceType::SuperSaw | SourceType::Sync
        | SourceType::Ring | SourceType::FBSin | SourceType::FM | SourceType::PhaseMod
        | SourceType::FMBell | SourceType::FMBrass
        | SourceType::Pluck | SourceType::Formant | SourceType::Gendy | SourceType::Chaos
        | SourceType::Additive | SourceType::Wavetable | SourceType::Granular
        | SourceType::Bowed | SourceType::Blown | SourceType::Membrane
        // Mallet percussion
        | SourceType::Marimba | SourceType::Vibes | SourceType::Kalimba | SourceType::SteelDrum
        | SourceType::TubularBell | SourceType::Glockenspiel
        // Plucked strings
        | SourceType::Guitar | SourceType::BassGuitar | SourceType::Harp | SourceType::Koto
        // Drums
        | SourceType::Kick | SourceType::Snare | SourceType::HihatClosed | SourceType::HihatOpen
        | SourceType::Clap | SourceType::Cowbell | SourceType::Rim | SourceType::Tom
        | SourceType::Clave | SourceType::Conga
        // Classic synths
        | SourceType::Choir | SourceType::EPiano | SourceType::Organ | SourceType::BrassStab
        | SourceType::Strings | SourceType::Acid
        | SourceType::Universe | SourceType::Dreamscape | SourceType::Soundtrack => p.osc_color,
        SourceType::AudioIn => p.audio_in_color,
        SourceType::PitchedSampler | SourceType::TimeStretch | SourceType::Kit => p.sample_color,
        SourceType::BusIn => p.bus_in_color,
        SourceType::Custom(_) => p.custom_color,
        SourceType::Vst(_) => p.vst_color,
    }
}

pub struct TrackPane {
    keymap: Keymap,
    /// Index into current instrument's clips list for placement selection
    selected_clip_index: usize,
    perf: PerformanceController,
    /// When Some, we're waiting for the user to select a target instrument to link with
    linking_from: Option<TrackId>,
    /// Text input for renaming layer groups
    edit_input: TextInput,
    /// Layer group being renamed (None = not editing)
    editing_group: Option<GroupId>,
}

impl TrackPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            selected_clip_index: 0,
            perf: PerformanceController::new(),
            linking_from: None,
            edit_input: TextInput::new(""),
            editing_group: None,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.editing_group.is_some()
    }

    pub fn set_enhanced_keyboard(&mut self, enabled: bool) {
        self.perf.set_enhanced_keyboard(enabled);
    }

    fn ticks_per_bar(&self, state: &AppState) -> u32 {
        let (beats, _) = state.session.time_signature;
        beats as u32 * 480
    }

    fn format_filter(instrument: &crate::state::track::Track) -> String {
        match instrument.filter() {
            Some(f) => format!("[{}]", f.filter_type.name()),
            None => "---".to_string(),
        }
    }

    fn format_eq(instrument: &crate::state::track::Track) -> &'static str {
        if instrument.eq().is_some() {
            "[EQ]"
        } else {
            ""
        }
    }

    fn format_effects(instrument: &crate::state::track::Track) -> String {
        let effects: Vec<_> = instrument.effects().collect();
        if effects.is_empty() {
            return "---".to_string();
        }
        effects
            .iter()
            .map(|e| e.effect_type.name())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_level(level: f32) -> String {
        let filled = (level * 5.0) as usize;
        let bar: String = (0..5)
            .map(|i| if i < filled { '\u{258a}' } else { '\u{2591}' })
            .collect();
        format!("{} {:.0}%", bar, level * 100.0)
    }
}

impl Default for TrackPane {
    fn default() -> Self {
        Self::new(Keymap::new())
    }
}

impl Pane for TrackPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("track")
    }

    fn handle_action(&mut self, action: ActionId, event: &InputEvent, state: &AppState) -> Action {
        let arr = &state.session.arrangement;
        let num_instruments = state.tracks.tracks.len();
        if num_instruments == 0 {
            return Action::None;
        }

        let lane = arr.selected_lane.min(num_instruments.saturating_sub(1));
        let instrument_id = state.tracks.tracks[lane].id;

        // Link mode intercept
        if let Some(from_id) = self.linking_from {
            match action {
                ActionId::Track(TrackActionId::LinkLayer) => {
                    self.linking_from = None;
                    if instrument_id != from_id {
                        return Action::Track(TrackAction::LinkLayer(from_id, instrument_id));
                    }
                    return Action::None;
                }
                // Navigation passes through
                ActionId::Track(TrackActionId::LaneUp)
                | ActionId::Track(TrackActionId::LaneDown) => {}
                // Any other action cancels linking mode
                _ => {
                    self.linking_from = None;
                }
            }
        }

        match action {
            // --- Track navigation ---
            ActionId::Track(TrackActionId::LaneUp) => {
                if lane > 0 {
                    Action::Arrangement(ArrangementAction::SelectLane(lane - 1))
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::LaneDown) => {
                if lane + 1 < num_instruments {
                    Action::Arrangement(ArrangementAction::SelectLane(lane + 1))
                } else {
                    Action::None
                }
            }

            // --- Arrangement timeline ---
            ActionId::Track(TrackActionId::CursorLeft) => {
                Action::Arrangement(ArrangementAction::MoveCursor(-1))
            }
            ActionId::Track(TrackActionId::CursorRight) => {
                Action::Arrangement(ArrangementAction::MoveCursor(1))
            }
            ActionId::Track(TrackActionId::CursorHome) => {
                let delta = -(arr.cursor_tick as i32 / arr.ticks_per_col.max(1) as i32);
                Action::Arrangement(ArrangementAction::MoveCursor(delta))
            }
            ActionId::Track(TrackActionId::CursorEnd) => {
                let end = arr.arrangement_length();
                if end > arr.cursor_tick {
                    let delta = (end - arr.cursor_tick) as i32 / arr.ticks_per_col.max(1) as i32;
                    Action::Arrangement(ArrangementAction::MoveCursor(delta))
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::NewClip) => {
                Action::Arrangement(ArrangementAction::CaptureClipFromPianoRoll { instrument_id })
            }
            ActionId::Track(TrackActionId::NewEmptyClip) => {
                let tpb = self.ticks_per_bar(state);
                Action::Arrangement(ArrangementAction::CreateClip {
                    instrument_id,
                    length_ticks: tpb,
                })
            }
            ActionId::Track(TrackActionId::PlaceClip) => {
                let clips = arr.clips_for_instrument(instrument_id);
                if clips.is_empty() {
                    return Action::None;
                }
                let idx = self.selected_clip_index.min(clips.len().saturating_sub(1));
                let clip_id = clips[idx].id;
                Action::Arrangement(ArrangementAction::AddClipInstance {
                    clip_id,
                    instrument_id,
                    start_tick: arr.cursor_tick,
                })
            }
            ActionId::Track(TrackActionId::EditClip) => {
                if let Some(placement) = arr.placement_at(instrument_id, arr.cursor_tick) {
                    Action::Arrangement(ArrangementAction::EnterClipEdit(placement.clip_id))
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::Delete) => {
                if let Some(placement) = arr.placement_at(instrument_id, arr.cursor_tick) {
                    Action::Arrangement(ArrangementAction::RemoveClipInstance(placement.id))
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::DeleteClip) => {
                let clips = arr.clips_for_instrument(instrument_id);
                if clips.is_empty() {
                    return Action::None;
                }
                let idx = self.selected_clip_index.min(clips.len().saturating_sub(1));
                let clip_id = clips[idx].id;
                Action::Arrangement(ArrangementAction::DeleteClip(clip_id))
            }
            ActionId::Track(TrackActionId::Duplicate) => {
                if let Some(placement) = arr.placement_at(instrument_id, arr.cursor_tick) {
                    Action::Arrangement(ArrangementAction::DuplicateClipInstance(placement.id))
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::ToggleMode) => {
                Action::Arrangement(ArrangementAction::TogglePlayMode)
            }
            ActionId::Track(TrackActionId::TogglePlayback) => {
                Action::Arrangement(ArrangementAction::TogglePlayback)
            }
            ActionId::Track(TrackActionId::MoveLeft) => {
                if let Some(placement) = arr.placement_at(instrument_id, arr.cursor_tick) {
                    let new_start = placement.start_tick.saturating_sub(arr.ticks_per_col);
                    Action::Arrangement(ArrangementAction::MoveClipInstance {
                        clip_instance_id: placement.id,
                        new_start_tick: new_start,
                    })
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::MoveRight) => {
                if let Some(placement) = arr.placement_at(instrument_id, arr.cursor_tick) {
                    let new_start = placement.start_tick + arr.ticks_per_col;
                    Action::Arrangement(ArrangementAction::MoveClipInstance {
                        clip_instance_id: placement.id,
                        new_start_tick: new_start,
                    })
                } else {
                    Action::None
                }
            }
            ActionId::Track(TrackActionId::ZoomIn) => {
                Action::Arrangement(ArrangementAction::ZoomIn)
            }
            ActionId::Track(TrackActionId::ZoomOut) => {
                Action::Arrangement(ArrangementAction::ZoomOut)
            }
            ActionId::Track(TrackActionId::SelectNextClipInstance) => {
                let placements = arr.placements_for_instrument(instrument_id);
                if placements.is_empty() {
                    return Action::None;
                }
                let next = match arr.selected_placement {
                    Some(i) => {
                        let next_idx = i + 1;
                        if next_idx < placements.len() {
                            Some(next_idx)
                        } else {
                            Some(0)
                        }
                    }
                    None => Some(0),
                };
                Action::Arrangement(ArrangementAction::SelectClipInstance(next))
            }
            ActionId::Track(TrackActionId::SelectPrevClipInstance) => {
                let placements = arr.placements_for_instrument(instrument_id);
                if placements.is_empty() {
                    return Action::None;
                }
                let prev = match arr.selected_placement {
                    Some(0) => Some(placements.len().saturating_sub(1)),
                    Some(i) => Some(i - 1),
                    None => Some(0),
                };
                Action::Arrangement(ArrangementAction::SelectClipInstance(prev))
            }
            ActionId::Track(TrackActionId::SelectNextClip) => {
                let clips = arr.clips_for_instrument(instrument_id);
                if !clips.is_empty() {
                    self.selected_clip_index = (self.selected_clip_index + 1) % clips.len();
                }
                Action::None
            }
            ActionId::Track(TrackActionId::SelectPrevClip) => {
                let clips = arr.clips_for_instrument(instrument_id);
                if !clips.is_empty() {
                    if self.selected_clip_index == 0 {
                        self.selected_clip_index = clips.len() - 1;
                    } else {
                        self.selected_clip_index -= 1;
                    }
                }
                Action::None
            }

            // --- Layer group ---
            ActionId::Track(TrackActionId::LinkLayer) => {
                self.linking_from = Some(instrument_id);
                Action::None
            }
            ActionId::Track(TrackActionId::UnlinkLayer) => {
                Action::Track(TrackAction::UnlinkLayer(instrument_id))
            }
            ActionId::Track(TrackActionId::LayerOctaveUp) => {
                Action::Track(TrackAction::AdjustLayerOctaveOffset(instrument_id, 1))
            }
            ActionId::Track(TrackActionId::LayerOctaveDown) => {
                Action::Track(TrackAction::AdjustLayerOctaveOffset(instrument_id, -1))
            }
            ActionId::Track(TrackActionId::RenameLayerGroup) => {
                let instrument = &state.tracks.tracks[lane];
                if let Some(group_id) = instrument.layer.group {
                    let current_name = state
                        .session
                        .mixer
                        .layer_group_mixer(group_id)
                        .map(|gm| gm.name.clone())
                        .unwrap_or_default();
                    self.edit_input.set_value(&current_name);
                    self.edit_input.select_all();
                    self.edit_input.set_focused(true);
                    self.editing_group = Some(group_id);
                    return Action::PushLayer("text_edit");
                }
                Action::None
            }

            // --- Text edit mode ---
            ActionId::Mode(ModeActionId::TextConfirm) => {
                if let Some(group_id) = self.editing_group.take() {
                    let name = self.edit_input.value().to_string();
                    self.edit_input.set_focused(false);
                    return Action::Group(GroupAction::Rename(group_id, name));
                }
                Action::None
            }
            ActionId::Mode(ModeActionId::TextCancel) => {
                self.editing_group = None;
                self.edit_input.set_focused(false);
                Action::None
            }

            // --- Piano layer ---
            ActionId::Mode(ModeActionId::PianoEscape) => {
                let was_active = self.perf.piano.is_active();
                self.perf.piano.handle_escape();
                if was_active && !self.perf.piano.is_active() {
                    Action::ExitPerformanceMode
                } else {
                    Action::None
                }
            }
            ActionId::Mode(ModeActionId::PianoOctaveDown) => {
                self.perf.piano.octave_down();
                Action::None
            }
            ActionId::Mode(ModeActionId::PianoOctaveUp) => {
                self.perf.piano.octave_up();
                Action::None
            }
            ActionId::Mode(ModeActionId::PianoKey) | ActionId::Mode(ModeActionId::PianoSpace) => {
                if let KeyCode::Char(c) = event.key {
                    let c = translate_key(c, state.keyboard_layout);
                    if let Some(pitches) = self.perf.piano.key_to_pitches(c) {
                        if let Some(new_pitches) = self.perf.piano.key_pressed(
                            c,
                            pitches.clone(),
                            event.timestamp,
                            event.is_repeat,
                        ) {
                            if new_pitches.len() == 1 {
                                return Action::Track(TrackAction::PlayNote(new_pitches[0], 100));
                            } else {
                                return Action::Track(TrackAction::PlayNotes(new_pitches, 100));
                            }
                        }
                    }
                }
                Action::None
            }

            // --- Pad layer ---
            ActionId::Mode(ModeActionId::PadEscape) => {
                self.perf.pad.deactivate();
                Action::ExitPerformanceMode
            }
            ActionId::Mode(ModeActionId::PadKey) => {
                if let KeyCode::Char(c) = event.key {
                    let c = translate_key(c, state.keyboard_layout);
                    if let Some(pad_idx) = self.perf.pad.key_to_pad(c) {
                        return Action::Track(TrackAction::PlayDrumPad(pad_idx, 100));
                    }
                }
                Action::None
            }

            _ => Action::None,
        }
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        if self.editing_group.is_some() {
            self.edit_input.handle_input(event);
        }
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let rect = center_rect(area, 97, 29);
        let arr = &state.session.arrangement;
        let ticks_per_col = arr.ticks_per_col.max(1);

        // Mode indicator for title
        let mode_str = match arr.play_mode {
            PlayMode::Pattern => "Pattern",
            PlayMode::Song => "Song",
        };
        let title = format!(" Track [{}] ", mode_str);

        let border_style = Style::new().fg(p.accent);
        let inner = buf.draw_block(rect, &title, border_style, border_style);

        if state.tracks.tracks.is_empty() {
            let text = "(no tracks)";
            let x = inner.x + (inner.width.saturating_sub(text.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.draw_line(
                Rect::new(x, y, text.len() as u16, 1),
                &[(text, Style::new().fg(p.dim))],
            );
            return;
        }

        // Layout: header(1) + lanes + footer(2)
        let label_width: u16 = 38;
        let timeline_x = inner.x + label_width + 1;
        let timeline_width = inner.width.saturating_sub(label_width + 2);
        let header_height: u16 = 1;
        let footer_height: u16 = 2;
        let lanes_area_y = inner.y + header_height;
        let lanes_area_height = inner.height.saturating_sub(header_height + footer_height);

        let num_instruments = state.tracks.tracks.len();
        let lane_height: u16 = 2;
        let max_visible = (lanes_area_height / lane_height) as usize;

        // Scroll to keep selected lane visible
        let selected_lane = arr.selected_lane.min(num_instruments.saturating_sub(1));
        let scroll = if selected_lane >= max_visible {
            selected_lane - max_visible + 1
        } else {
            0
        };

        let sel_bg = Style::new().bg(p.selection_bg);
        let bar_line_style = Style::new().fg(p.border);
        let separator_style = Style::new().fg(p.muted);

        // Compute bar spacing in columns
        let (beats_per_bar, _) = state.session.time_signature;
        let ticks_per_bar = beats_per_bar as u32 * 480;
        let cols_per_bar = ticks_per_bar / ticks_per_col;
        let cols_per_beat = 480 / ticks_per_col;

        // --- Header: bar numbers ---
        let header_y = inner.y;
        let header_label_style = Style::new().fg(p.dim);
        for col in 0..timeline_width as u32 {
            let tick = arr.view_start_tick + col * ticks_per_col;
            if cols_per_bar > 0 && (tick % ticks_per_bar) < ticks_per_col {
                let bar_num = tick / ticks_per_bar + 1;
                let label = format!("{}", bar_num);
                let x = timeline_x + col as u16;
                for (j, ch) in label.chars().enumerate() {
                    if x + (j as u16) < inner.x + inner.width {
                        buf.set_cell(x + j as u16, header_y, ch, header_label_style);
                    }
                }
            }
        }

        // --- Track lanes ---
        for (vi, i) in (scroll..num_instruments).enumerate() {
            if vi >= max_visible {
                break;
            }
            let instrument = &state.tracks.tracks[i];
            let is_selected = i == selected_lane;
            let lane_y = lanes_area_y + (vi as u16) * lane_height;

            if lane_y + lane_height > lanes_area_y + lanes_area_height {
                break;
            }

            let source_c = source_color(instrument.source, &p);

            let mk_style = |fg: Color| -> Style {
                if is_selected {
                    Style::new().fg(fg).bg(p.selection_bg)
                } else {
                    Style::new().fg(fg)
                }
            };

            // Fill label area bg for selected
            if is_selected {
                for row in 0..lane_height {
                    let y = lane_y + row;
                    if y >= lanes_area_y + lanes_area_height {
                        break;
                    }
                    for x in inner.x..timeline_x {
                        buf.set_cell(x, y, ' ', sel_bg);
                    }
                }
            }

            // Selection indicator
            if is_selected {
                buf.set_cell(
                    inner.x,
                    lane_y,
                    '>',
                    Style::new().fg(p.fg).bg(p.selection_bg).bold(),
                );
            }

            // Line 1: number + name + source + layer group
            let num_str = format!("{:>2} ", i + 1);
            let name_str = format!("{:11}", &instrument.name[..instrument.name.len().min(11)]);
            let src_short = format!(
                " {:7}",
                &instrument.source.name()[..instrument.source.name().len().min(7)]
            );

            let layer_str = match instrument.layer.group {
                Some(g) => {
                    let group_name = state
                        .session
                        .mixer
                        .layer_group_mixer(g)
                        .map(|gm| gm.name.as_str())
                        .unwrap_or("");
                    let label = if group_name.is_empty() {
                        format!("L{}", g)
                    } else {
                        group_name.to_string()
                    };
                    if instrument.layer.octave_offset != 0 {
                        format!(" [{}:{:+}]", label, instrument.layer.octave_offset)
                    } else {
                        format!(" [{}]", label)
                    }
                }
                None => String::new(),
            };

            // Ownership indicator
            let ownership_str = match state.ownership_status(instrument.id) {
                OwnershipDisplayStatus::OwnedByMe => " [ME]".to_string(),
                OwnershipDisplayStatus::OwnedByOther(ref name) => {
                    let short = if name.len() > 6 { &name[..6] } else { name };
                    format!(" [{}]", short)
                }
                OwnershipDisplayStatus::Unowned => String::new(),
                OwnershipDisplayStatus::Local => String::new(),
            };
            let ownership_color = match state.ownership_status(instrument.id) {
                OwnershipDisplayStatus::OwnedByMe => p.success,
                OwnershipDisplayStatus::OwnedByOther(_) => p.accent_secondary,
                _ => p.dim,
            };

            let mut line1_spans: Vec<(&str, Style)> = vec![
                (&num_str, mk_style(p.dim)),
                (
                    &name_str,
                    if is_selected {
                        mk_style(p.fg).bold()
                    } else {
                        mk_style(p.fg)
                    },
                ),
                (&src_short, mk_style(source_c)),
            ];
            if !layer_str.is_empty() {
                line1_spans.push((&layer_str, mk_style(p.accent_secondary)));
            }
            if !ownership_str.is_empty() {
                line1_spans.push((&ownership_str, mk_style(ownership_color)));
            }
            buf.draw_line(Rect::new(inner.x + 1, lane_y, label_width, 1), &line1_spans);

            // Line 2: filter + EQ + effects + level
            let filter_str = format!(
                "{:6}",
                &Self::format_filter(instrument)[..Self::format_filter(instrument).len().min(6)]
            );
            let eq_str = format!(" {:4}", Self::format_eq(instrument));
            let fx_raw = Self::format_effects(instrument);
            let fx_str = format!(" {:10}", &fx_raw[..fx_raw.len().min(10)]);
            let level_str = format!(" {}", Self::format_level(instrument.channel_strip.level));

            buf.draw_line(
                Rect::new(inner.x + 4, lane_y + 1, label_width.saturating_sub(3), 1),
                &[
                    (&filter_str, mk_style(p.filter_color)),
                    (&eq_str, mk_style(p.eq_color)),
                    (&fx_str, mk_style(p.fx_color)),
                    (&level_str, mk_style(p.success)),
                ],
            );

            // Separator between label and timeline
            for row in 0..lane_height {
                let y = lane_y + row;
                if y >= lanes_area_y + lanes_area_height {
                    break;
                }
                buf.set_cell(
                    inner.x + label_width,
                    y,
                    '\u{2502}',
                    Style::new().fg(p.border),
                );
            }

            // Timeline area: bar/beat lines + clip blocks
            let inst_id = instrument.id;

            // Draw bar/beat lines
            for col in 0..timeline_width as u32 {
                let tick = arr.view_start_tick + col * ticks_per_col;
                let x = timeline_x + col as u16;
                let is_bar = cols_per_bar > 0 && (tick % ticks_per_bar) < ticks_per_col;
                let is_beat = cols_per_beat > 0 && (tick % 480) < ticks_per_col;

                for row in 0..lane_height {
                    let y = lane_y + row;
                    if y >= lanes_area_y + lanes_area_height {
                        break;
                    }
                    if is_bar {
                        buf.set_cell(x, y, '|', bar_line_style);
                    } else if is_beat && row == 0 {
                        buf.set_cell(x, y, '.', Style::new().fg(p.muted));
                    }
                }
            }

            // Draw clip placements for this instrument
            let placements = arr.placements_for_instrument(inst_id);
            for placement in &placements {
                if let Some(clip) = arr.clip(placement.clip_id) {
                    let _eff_len = placement.effective_length(clip);
                    let start_col =
                        placement.start_tick.saturating_sub(arr.view_start_tick) / ticks_per_col;
                    let end_col = placement.end_tick(clip).saturating_sub(arr.view_start_tick)
                        / ticks_per_col;

                    // Skip if entirely off-screen
                    if placement.end_tick(clip) <= arr.view_start_tick {
                        continue;
                    }
                    if placement.start_tick
                        >= arr.view_start_tick + (timeline_width as u32) * ticks_per_col
                    {
                        continue;
                    }

                    let vis_start = if placement.start_tick < arr.view_start_tick {
                        0
                    } else {
                        start_col as u16
                    };
                    let vis_end = (end_col as u16).min(timeline_width);

                    if vis_start >= vis_end {
                        continue;
                    }

                    let clip_bg = source_c;
                    let clip_style = Style::new().fg(p.bg).bg(clip_bg);
                    let sel_clip_style = Style::new().fg(p.fg).bg(p.selection_bg).bold();

                    let is_placement_selected = arr
                        .selected_placement
                        .and_then(|idx| arr.placements.get(idx))
                        .map(|p| p.id == placement.id)
                        .unwrap_or(false);

                    let style = if is_placement_selected {
                        sel_clip_style
                    } else {
                        clip_style
                    };

                    // Render clip block
                    let block_width = vis_end - vis_start;
                    let name = &clip.name;
                    let display_name: String = if name.len() > block_width as usize {
                        name[..block_width as usize].to_string()
                    } else {
                        let padding = block_width as usize - name.len();
                        let left_pad = 0;
                        let right_pad = padding;
                        format!("{}{}{}", " ".repeat(left_pad), name, " ".repeat(right_pad))
                    };

                    // Fill both rows of the lane
                    for row in 0..lane_height {
                        let y = lane_y + row;
                        if y >= lanes_area_y + lanes_area_height {
                            break;
                        }
                        let x = timeline_x + vis_start;
                        if row == 0 {
                            for (j, ch) in display_name.chars().enumerate() {
                                if x + (j as u16) < timeline_x + timeline_width {
                                    buf.set_cell(x + j as u16, y, ch, style);
                                }
                            }
                        } else {
                            for j in 0..block_width {
                                if x + j < timeline_x + timeline_width {
                                    buf.set_cell(x + j, y, ' ', style);
                                }
                            }
                        }
                    }

                    // Clip boundary markers
                    if vis_start > 0 || placement.start_tick >= arr.view_start_tick {
                        let x = timeline_x + vis_start;
                        for row in 0..lane_height {
                            let y = lane_y + row;
                            if y >= lanes_area_y + lanes_area_height {
                                break;
                            }
                            buf.set_cell(x, y, '[', style);
                        }
                    }
                    if vis_end <= timeline_width && vis_end > vis_start {
                        let x = timeline_x + vis_end - 1;
                        for row in 0..lane_height {
                            let y = lane_y + row;
                            if y >= lanes_area_y + lanes_area_height {
                                break;
                            }
                            buf.set_cell(x, y, ']', style);
                        }
                    }
                }
            }

            // Horizontal separator below each lane
            if vi + 1 < max_visible && i + 1 < num_instruments {
                let sep_y = lane_y + lane_height;
                if sep_y < lanes_area_y + lanes_area_height {
                    for x in (inner.x + label_width + 1)..(inner.x + inner.width) {
                        buf.set_cell(x, sep_y, '-', separator_style);
                    }
                }
            }
        }

        // --- Playhead ---
        let playhead_tick = state.audio.playhead;
        if playhead_tick >= arr.view_start_tick {
            let playhead_col = (playhead_tick - arr.view_start_tick) / ticks_per_col;
            if (playhead_col as u16) < timeline_width {
                let x = timeline_x + playhead_col as u16;
                let ph_style = Style::new().fg(p.fg).bold();
                for y in lanes_area_y..(lanes_area_y + lanes_area_height) {
                    buf.set_cell(x, y, '|', ph_style);
                }
            }
        }

        // --- Cursor --- (full-height bar, uses raw_buf read-back to preserve clip visibility)
        if arr.cursor_tick >= arr.view_start_tick {
            let cursor_col = (arr.cursor_tick - arr.view_start_tick) / ticks_per_col;
            if (cursor_col as u16) < timeline_width {
                let x = timeline_x + cursor_col as u16;
                let cursor_style = Style::new().fg(p.accent);
                for y in lanes_area_y..(lanes_area_y + lanes_area_height) {
                    if let Some(cell) = buf.raw_buf().cell_mut((x, y)) {
                        if cell.symbol() == " " {
                            cell.set_char('|').set_style(cursor_style);
                        }
                    }
                }
            }
        }

        // --- Footer ---
        let footer_y = inner.y + inner.height - 2;

        let bar = arr.cursor_tick / ticks_per_bar + 1;
        let beat = (arr.cursor_tick % ticks_per_bar) / 480 + 1;
        let inst_id = state.tracks.tracks[selected_lane].id;
        let clips = arr.clips_for_instrument(inst_id);
        let clip_info = if clips.is_empty() {
            "No clips".to_string()
        } else {
            let idx = self.selected_clip_index.min(clips.len().saturating_sub(1));
            format!("Clip: {} [{}/{}]", clips[idx].name, idx + 1, clips.len())
        };

        let pos_str = format!("Bar {} Beat {}  |  {}", bar, beat, clip_info);
        buf.draw_line(
            Rect::new(inner.x + 1, footer_y + 1, inner.width.saturating_sub(2), 1),
            &[(&pos_str, Style::new().fg(p.border))],
        );

        // --- Mode indicators in title bar ---
        if self.perf.pad.is_active() {
            let pad_str = self.perf.pad.status_label();
            let pad_x = rect.x + rect.width - pad_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(pad_x, rect.y, pad_str.len() as u16, 1),
                &[(&pad_str, Style::new().fg(p.bg).bg(p.kit_color))],
            );
        } else if self.perf.piano.is_active() {
            let piano_str = self.perf.piano.status_label();
            let piano_x = rect.x + rect.width - piano_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(piano_x, rect.y, piano_str.len() as u16, 1),
                &[(&piano_str, Style::new().fg(p.bg).bg(p.fx_color))],
            );
        }

        // Link mode indicator
        if self.linking_from.is_some() {
            let link_str = " LINK: \u{2191}/\u{2193} navigate, l confirm, Esc cancel ";
            let link_x = rect.x + rect.width - link_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(link_x, rect.y, link_str.len() as u16, 1),
                &[(link_str, Style::new().fg(p.bg).bg(p.accent_secondary))],
            );
        }

        // Rename mode indicator + inline text input
        if self.editing_group.is_some() {
            let rename_str = " RENAME: Enter confirm, Esc cancel ";
            let rename_x = rect.x + rect.width - rename_str.len() as u16 - 1;
            buf.draw_line(
                Rect::new(rename_x, rect.y, rename_str.len() as u16, 1),
                &[(rename_str, Style::new().fg(p.bg).bg(p.success))],
            );

            let input_y = inner.y + inner.height.saturating_sub(2);
            let input_x = inner.x + 1;
            let input_width = inner.width.saturating_sub(2);
            self.edit_input
                .render_buf(buf.raw_buf(), input_x, input_y, input_width, &p);
            if let Some((cx, cy)) = self.edit_input.screen_cursor() {
                buf.set_cursor_position(cx, cy);
            }
        }
    }

    fn handle_mouse(&mut self, event: &MouseEvent, area: Rect, state: &AppState) -> Action {
        let rect = center_rect(area, 97, 29);
        let arr = &state.session.arrangement;
        let num_instruments = state.tracks.tracks.len();
        if num_instruments == 0 {
            return Action::None;
        }

        let header_height: u16 = 1;
        let footer_height: u16 = 2;
        // border (1) + header
        let lanes_area_y = rect.y + 1 + header_height;
        let inner_height = rect.height.saturating_sub(2);
        let lanes_area_height = inner_height.saturating_sub(header_height + footer_height);
        let lane_height: u16 = 2;
        let max_visible = (lanes_area_height / lane_height) as usize;

        let selected_lane = arr.selected_lane.min(num_instruments.saturating_sub(1));
        let scroll = if selected_lane >= max_visible {
            selected_lane - max_visible + 1
        } else {
            0
        };

        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                let row = event.row;
                if row >= lanes_area_y && row < lanes_area_y + (max_visible as u16) * lane_height {
                    let clicked_vi = (row - lanes_area_y) as usize / lane_height as usize;
                    let clicked_idx = scroll + clicked_vi;
                    if clicked_idx < num_instruments {
                        return Action::Arrangement(ArrangementAction::SelectLane(clicked_idx));
                    }
                }
                Action::None
            }
            MouseEventKind::ScrollUp => {
                if selected_lane > 0 {
                    Action::Arrangement(ArrangementAction::SelectLane(selected_lane - 1))
                } else {
                    Action::None
                }
            }
            MouseEventKind::ScrollDown => {
                if selected_lane + 1 < num_instruments {
                    Action::Arrangement(ArrangementAction::SelectLane(selected_lane + 1))
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AppState, SourceType};
    use crate::ui::action_id::{ActionId, TrackActionId};
    use crate::ui::{InputEvent, KeyCode, Modifiers};

    fn dummy_event() -> InputEvent {
        InputEvent::new(KeyCode::Char('x'), Modifiers::default())
    }

    #[test]
    fn link_mode_navigation_passes_through() {
        let mut state = AppState::new();
        let id0 = state.add_track(SourceType::Saw);
        let _id1 = state.add_track(SourceType::Sin);

        // First track selected via arrangement lane
        state.session.arrangement.selected_lane = 0;
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        // Enter linking mode
        pane.handle_action(
            ActionId::Track(TrackActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert_eq!(pane.linking_from, Some(id0));

        // LaneDown should pass through (return SelectLane), not complete the link
        let action = pane.handle_action(
            ActionId::Track(TrackActionId::LaneDown),
            &dummy_event(),
            &state,
        );
        assert!(matches!(
            action,
            Action::Arrangement(ArrangementAction::SelectLane(1))
        ));
        // Should still be in linking mode
        assert_eq!(pane.linking_from, Some(id0));
    }

    #[test]
    fn link_mode_confirm_with_different_target() {
        let mut state = AppState::new();
        let id0 = state.add_track(SourceType::Saw);
        let id1 = state.add_track(SourceType::Sin);

        state.session.arrangement.selected_lane = 0;
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        pane.handle_action(
            ActionId::Track(TrackActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert_eq!(pane.linking_from, Some(id0));

        // Move selection to second track
        state.session.arrangement.selected_lane = 1;

        // Press LinkLayer again to confirm
        let action = pane.handle_action(
            ActionId::Track(TrackActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::LinkLayer(from, to)) => {
                assert_eq!(from, id0);
                assert_eq!(to, id1);
            }
            _ => panic!("Expected LinkLayer action, got {:?}", action),
        }
        assert!(pane.linking_from.is_none());
    }

    #[test]
    fn link_mode_confirm_same_instrument_no_action() {
        let mut state = AppState::new();
        let _id0 = state.add_track(SourceType::Saw);

        state.session.arrangement.selected_lane = 0;
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        pane.handle_action(
            ActionId::Track(TrackActionId::LinkLayer),
            &dummy_event(),
            &state,
        );

        // Confirm without moving — same instrument
        let action = pane.handle_action(
            ActionId::Track(TrackActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert!(matches!(action, Action::None));
        assert!(pane.linking_from.is_none());
    }

    #[test]
    fn link_mode_cancelled_by_other_action() {
        let mut state = AppState::new();
        let _id0 = state.add_track(SourceType::Saw);

        state.session.arrangement.selected_lane = 0;
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        pane.handle_action(
            ActionId::Track(TrackActionId::LinkLayer),
            &dummy_event(),
            &state,
        );
        assert!(pane.linking_from.is_some());

        // Any non-nav, non-link action should cancel
        pane.handle_action(
            ActionId::Track(TrackActionId::Delete),
            &dummy_event(),
            &state,
        );
        assert!(pane.linking_from.is_none());
    }

    #[test]
    fn layer_octave_up_returns_adjust_action() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);

        state.session.arrangement.selected_lane = 0;
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        let action = pane.handle_action(
            ActionId::Track(TrackActionId::LayerOctaveUp),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::AdjustLayerOctaveOffset(got_id, delta)) => {
                assert_eq!(got_id, id);
                assert_eq!(delta, 1);
            }
            _ => panic!("Expected AdjustLayerOctaveOffset(+1), got {:?}", action),
        }
    }

    #[test]
    fn layer_octave_down_returns_adjust_action() {
        let mut state = AppState::new();
        let id = state.add_track(SourceType::Saw);

        state.session.arrangement.selected_lane = 0;
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        let action = pane.handle_action(
            ActionId::Track(TrackActionId::LayerOctaveDown),
            &dummy_event(),
            &state,
        );
        match action {
            Action::Track(TrackAction::AdjustLayerOctaveOffset(got_id, delta)) => {
                assert_eq!(got_id, id);
                assert_eq!(delta, -1);
            }
            _ => panic!("Expected AdjustLayerOctaveOffset(-1), got {:?}", action),
        }
    }

    #[test]
    fn performance_mode_toggle() {
        let mut state = AppState::new();
        let _id = state.add_track(SourceType::Saw);
        state.tracks.selected = Some(0);

        let mut pane = TrackPane::new(Keymap::new());
        assert!(pane.supports_performance_mode());

        let result = pane.toggle_performance_mode(&state);
        assert!(matches!(result, ToggleResult::ActivatedPiano));

        let result = pane.toggle_performance_mode(&state);
        assert!(matches!(
            result,
            ToggleResult::CycledLayout | ToggleResult::Deactivated
        ));
    }
}
