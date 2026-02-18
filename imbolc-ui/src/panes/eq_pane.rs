use std::any::Any;

use crate::state::{AppState, EqBandType, EqConfig, TrackId};
use crate::ui::action_id::{ActionId, EqActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::{
    Action, BusAction, Color, GroupAction, InputEvent, Keymap, LayerName, MasterAction, Palette,
    Pane, PaneIdStr, Rect, RenderBuf, Style, TrackAction,
};
use imbolc_types::FilterSlope;
use imbolc_types::{BusId, EffectId, EqualizerParamKind, GroupId, MixerSelection};

use crate::state::track::EqBand;

/// Whether the EQ pane targets the current mixer channel or always the master output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqMode {
    /// Follows the mixer selection (track, group, bus, or master).
    Channel,
    /// Always targets the master output EQ.
    Master,
}

/// What the EQ pane is currently targeting.
#[derive(Debug, Clone, Copy)]
enum EqTarget {
    Track(TrackId),
    Group(GroupId),
    Bus(BusId),
    Master,
}

pub struct EqPane {
    mode: EqMode,
    keymap: Keymap,
    selected_band: usize,  // 0-5
    selected_param: usize, // context-dependent (see param_count_for_band)
}

impl EqPane {
    pub fn new(mode: EqMode, keymap: Keymap) -> Self {
        Self {
            mode,
            keymap,
            selected_band: 0,
            selected_param: 0, // default to freq
        }
    }
}

impl Default for EqPane {
    fn default() -> Self {
        Self::new(EqMode::Channel, Keymap::new())
    }
}

/// Resolved EQ target info.
struct ResolvedEq<'a> {
    target: EqTarget,
    eq: Option<&'a EqConfig>,
    /// EffectId of the first (or targeted) EQ stage.
    effect_id: Option<EffectId>,
    title: String,
}

/// Resolve the EQ target, returning target info with the first EQ's EffectId.
fn resolve_target(mode: EqMode, state: &AppState) -> Option<ResolvedEq<'_>> {
    match mode {
        EqMode::Master => {
            let eq = state.session.mixer.master_eq();
            let eid = state.session.mixer.master_channel_strip.first_eq_id();
            Some(ResolvedEq {
                target: EqTarget::Master,
                eq,
                effect_id: eid,
                title: "Master".to_string(),
            })
        }
        EqMode::Channel => match state.session.mixer.selection {
            MixerSelection::Track(id) => {
                let track = state.tracks.track(id)?;
                let eq = track.eq();
                let eid = track.first_eq_id();
                Some(ResolvedEq {
                    target: EqTarget::Track(track.id),
                    eq,
                    effect_id: eid,
                    title: track.name.clone(),
                })
            }
            MixerSelection::Group(group_id) => {
                let gm = state.session.mixer.layer_group_mixer(group_id)?;
                let eq = gm.eq();
                let eid = gm.channel_strip.first_eq_id();
                let name = gm.name.clone();
                Some(ResolvedEq {
                    target: EqTarget::Group(group_id),
                    eq,
                    effect_id: eid,
                    title: name,
                })
            }
            MixerSelection::Bus(bus_id) => {
                let bus = state.session.bus(bus_id)?;
                let eq = bus.channel_strip.eq();
                let eid = bus.channel_strip.first_eq_id();
                Some(ResolvedEq {
                    target: EqTarget::Bus(bus_id),
                    eq,
                    effect_id: eid,
                    title: bus.name.clone(),
                })
            }
            MixerSelection::Master => {
                let eq = state.session.mixer.master_eq();
                let eid = state.session.mixer.master_channel_strip.first_eq_id();
                Some(ResolvedEq {
                    target: EqTarget::Master,
                    eq,
                    effect_id: eid,
                    title: "Master".to_string(),
                })
            }
        },
    }
}

fn make_toggle_action(target: EqTarget) -> Action {
    match target {
        EqTarget::Track(id) => Action::Track(TrackAction::ToggleEqualizer(id)),
        EqTarget::Group(id) => Action::Group(GroupAction::ToggleEqualizer(id)),
        EqTarget::Bus(id) => Action::Bus(BusAction::ToggleEqualizer(id)),
        EqTarget::Master => Action::Master(MasterAction::ToggleEqualizer),
    }
}

fn make_set_action(
    target: EqTarget,
    effect_id: EffectId,
    band_idx: usize,
    param: EqualizerParamKind,
    value: f32,
) -> Action {
    match target {
        EqTarget::Track(id) => Action::Track(TrackAction::SetEqualizerParam(
            id, effect_id, band_idx, param, value,
        )),
        EqTarget::Group(id) => Action::Group(GroupAction::SetEqualizerParam(
            id, effect_id, band_idx, param, value,
        )),
        EqTarget::Bus(id) => Action::Bus(BusAction::SetEqualizerParam(
            id, effect_id, band_idx, param, value,
        )),
        EqTarget::Master => Action::Master(MasterAction::SetEqualizerParam(
            effect_id, band_idx, param, value,
        )),
    }
}

/// How many navigable params does this band type have?
/// HPF/LPF: freq, slope, enabled (3)
/// Bell/Shelf: freq, gain, Q, type, enabled (5)
fn param_count_for_band(band: &EqBand) -> usize {
    if band.band_type.has_slope() {
        3 // freq, slope, enabled
    } else {
        5 // freq, gain, Q, type, enabled
    }
}

/// Map (band_type, param_idx) to the logical parameter.
/// Returns (EqualizerParamKind, is_toggle).
fn param_for_index(band: &EqBand, param_idx: usize) -> Option<(EqualizerParamKind, bool)> {
    if band.band_type.has_slope() {
        // HPF/LPF: 0=freq, 1=slope, 2=enabled
        match param_idx {
            0 => Some((EqualizerParamKind::Freq, false)),
            1 => Some((EqualizerParamKind::Slope, false)),
            2 => Some((EqualizerParamKind::Enabled, true)),
            _ => None,
        }
    } else {
        // Bell/Shelf: 0=freq, 1=gain, 2=Q, 3=type, 4=enabled
        match param_idx {
            0 => Some((EqualizerParamKind::Freq, false)),
            1 => Some((EqualizerParamKind::Gain, false)),
            2 => Some((EqualizerParamKind::Q, false)),
            3 => Some((EqualizerParamKind::BandType, false)),
            4 => Some((EqualizerParamKind::Enabled, true)),
            _ => None,
        }
    }
}

impl Pane for EqPane {
    fn id(&self) -> PaneIdStr {
        match self.mode {
            EqMode::Channel => PaneIdStr("eq"),
            EqMode::Master => PaneIdStr("master_eq"),
        }
    }

    fn layer_name(&self) -> LayerName {
        LayerName("eq")
    }

    fn handle_action(&mut self, action: ActionId, _event: &InputEvent, state: &AppState) -> Action {
        let resolved = match resolve_target(self.mode, state) {
            Some(r) => r,
            None => return Action::None,
        };
        let ResolvedEq {
            target,
            eq,
            effect_id,
            ..
        } = resolved;
        // If no EQ exists yet, only toggle is valid
        let eid = match effect_id {
            Some(id) => id,
            None => {
                return match action {
                    ActionId::Eq(EqActionId::ToggleEq) => make_toggle_action(target),
                    _ => Action::None,
                };
            }
        };

        match action {
            ActionId::Eq(EqActionId::PrevBand) => {
                self.selected_band = self.selected_band.saturating_sub(1);
                if let Some(eq) = eq {
                    let max_param =
                        param_count_for_band(&eq.bands[self.selected_band]).saturating_sub(1);
                    self.selected_param = self.selected_param.min(max_param);
                }
                Action::None
            }
            ActionId::Eq(EqActionId::NextBand) => {
                self.selected_band = (self.selected_band + 1).min(5);
                if let Some(eq) = eq {
                    let max_param =
                        param_count_for_band(&eq.bands[self.selected_band]).saturating_sub(1);
                    self.selected_param = self.selected_param.min(max_param);
                }
                Action::None
            }
            ActionId::Eq(EqActionId::PrevParam) => {
                self.selected_param = self.selected_param.saturating_sub(1);
                Action::None
            }
            ActionId::Eq(EqActionId::NextParam) => {
                if let Some(eq) = eq {
                    let max = param_count_for_band(&eq.bands[self.selected_band]).saturating_sub(1);
                    self.selected_param = (self.selected_param + 1).min(max);
                }
                Action::None
            }
            ActionId::Eq(EqActionId::Increase)
            | ActionId::Eq(EqActionId::IncreaseBig)
            | ActionId::Eq(EqActionId::IncreaseTiny) => adjust_param(
                target,
                eid,
                eq,
                self.selected_band,
                self.selected_param,
                true,
                action,
            ),
            ActionId::Eq(EqActionId::Decrease)
            | ActionId::Eq(EqActionId::DecreaseBig)
            | ActionId::Eq(EqActionId::DecreaseTiny) => adjust_param(
                target,
                eid,
                eq,
                self.selected_band,
                self.selected_param,
                false,
                action,
            ),
            ActionId::Eq(EqActionId::ToggleEq) => make_toggle_action(target),
            ActionId::Eq(EqActionId::ToggleBand) => {
                if let Some(eq) = eq {
                    let band = &eq.bands[self.selected_band];
                    let new_val = if band.enabled { 0.0 } else { 1.0 };
                    make_set_action(
                        target,
                        eid,
                        self.selected_band,
                        EqualizerParamKind::Enabled,
                        new_val,
                    )
                } else {
                    Action::None
                }
            }
            ActionId::Eq(EqActionId::CycleType) => {
                if let Some(eq) = eq {
                    let band = &eq.bands[self.selected_band];
                    let allowed = EqBandType::allowed_types(self.selected_band);
                    if allowed.len() > 1 {
                        let cur_idx = allowed
                            .iter()
                            .position(|t| *t == band.band_type)
                            .unwrap_or(0);
                        let next_idx = (cur_idx + 1) % allowed.len();
                        make_set_action(
                            target,
                            eid,
                            self.selected_band,
                            EqualizerParamKind::BandType,
                            next_idx as f32,
                        )
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                }
            }
            ActionId::Eq(EqActionId::CycleSlope) => {
                if let Some(eq) = eq {
                    let band = &eq.bands[self.selected_band];
                    if band.band_type.has_slope() {
                        let next = band.slope.next();
                        make_set_action(
                            target,
                            eid,
                            self.selected_band,
                            EqualizerParamKind::Slope,
                            next.to_normalized(),
                        )
                    } else {
                        Action::None
                    }
                } else {
                    Action::None
                }
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let rect = center_rect(area, 78, 24);

        let (target_name, eq) = match resolve_target(self.mode, state) {
            Some(r) => (r.title, r.eq),
            None => {
                let border_style = Style::new().fg(p.eq_color);
                let inner = buf.draw_block(rect, " EQ: (none) ", border_style, border_style);
                render_centered_text(inner, buf, "(no target selected)", p.dim);
                return;
            }
        };

        let title = match self.mode {
            EqMode::Channel => format!(" Channel EQ: {} ", target_name),
            EqMode::Master => format!(" Master EQ: {} ", target_name),
        };

        let border_style = Style::new().fg(p.eq_color);
        let inner = buf.draw_block(rect, &title, border_style, border_style);

        let eq = match eq {
            Some(eq) => eq,
            None => {
                render_centered_text(inner, buf, "EQ off — press 'e' to enable", p.dim);
                return;
            }
        };

        // -- Frequency response curve --
        let curve_y = inner.y;
        let curve_height = inner.height.saturating_sub(10).max(4);
        let curve_width = inner.width.saturating_sub(6);
        let curve_x = inner.x + 5;

        render_frequency_curve(
            curve_x,
            curve_y,
            curve_width,
            curve_height,
            eq,
            self.selected_band,
            buf,
            &p,
        );

        // dB axis labels
        let db_labels = ["+24", "+12", "  0", "-12", "-24"];
        let db_style = Style::new().fg(p.dim);
        for (i, label) in db_labels.iter().enumerate() {
            let y = curve_y + (i as u16) * (curve_height.saturating_sub(1)) / 4;
            if y < inner.y + inner.height {
                for (ci, ch) in label.chars().enumerate() {
                    let x = inner.x + ci as u16;
                    if x < inner.x + 4 {
                        buf.set_cell(x, y, ch, db_style);
                    }
                }
            }
        }

        // Frequency axis labels
        let freq_labels = ["20", "100", "500", "1k", "5k", "10k", "20k"];
        let freq_axis_y = curve_y + curve_height;
        let freq_style = Style::new().fg(p.dim);
        if freq_axis_y < inner.y + inner.height {
            for (i, label) in freq_labels.iter().enumerate() {
                let frac = i as f32 / (freq_labels.len() - 1) as f32;
                let x = curve_x + (frac * (curve_width.saturating_sub(1) as f32)) as u16;
                for (ci, ch) in label.chars().enumerate() {
                    let px = x + ci as u16;
                    if px < inner.x + inner.width {
                        buf.set_cell(px, freq_axis_y, ch, freq_style);
                    }
                }
            }
        }

        // -- Band info (single row of 6 bands) --
        let info_y = freq_axis_y + 2;
        render_band_info(
            inner.x,
            info_y,
            inner.width,
            eq,
            self.selected_band,
            self.selected_param,
            buf,
            &p,
        );
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

// -- Helpers --

fn render_centered_text(area: Rect, buf: &mut RenderBuf, text: &str, color: Color) {
    let x = area.x + (area.width.saturating_sub(text.len() as u16)) / 2;
    let y = area.y + area.height / 2;
    let style = Style::new().fg(color);
    buf.draw_line(Rect::new(x, y, text.len() as u16, 1), &[(text, style)]);
}

fn adjust_param(
    target: EqTarget,
    effect_id: EffectId,
    eq: Option<&EqConfig>,
    band_idx: usize,
    param_idx: usize,
    increase: bool,
    action: ActionId,
) -> Action {
    let eq = match eq {
        Some(eq) => eq,
        None => return Action::None,
    };
    let band = &eq.bands[band_idx];

    let (param_kind, is_toggle) = match param_for_index(band, param_idx) {
        Some(p) => p,
        None => return Action::None,
    };

    // Toggle params (enabled) handled via ToggleBand, not adjust
    if is_toggle {
        return Action::None;
    }

    match param_kind {
        EqualizerParamKind::Freq => {
            let delta =
                match action {
                    ActionId::Eq(EqActionId::IncreaseBig)
                    | ActionId::Eq(EqActionId::DecreaseBig) => band.freq * 0.2,
                    ActionId::Eq(EqActionId::IncreaseTiny)
                    | ActionId::Eq(EqActionId::DecreaseTiny) => band.freq * 0.01,
                    _ => band.freq * 0.05,
                };
            let new_val = if increase {
                (band.freq + delta).min(20000.0)
            } else {
                (band.freq - delta).max(20.0)
            };
            make_set_action(
                target,
                effect_id,
                band_idx,
                EqualizerParamKind::Freq,
                new_val,
            )
        }
        EqualizerParamKind::Gain => {
            let delta =
                match action {
                    ActionId::Eq(EqActionId::IncreaseBig)
                    | ActionId::Eq(EqActionId::DecreaseBig) => 3.0,
                    ActionId::Eq(EqActionId::IncreaseTiny)
                    | ActionId::Eq(EqActionId::DecreaseTiny) => 0.1,
                    _ => 0.5,
                };
            let new_val = if increase {
                (band.gain + delta).min(24.0)
            } else {
                (band.gain - delta).max(-24.0)
            };
            make_set_action(
                target,
                effect_id,
                band_idx,
                EqualizerParamKind::Gain,
                new_val,
            )
        }
        EqualizerParamKind::Q => {
            let delta =
                match action {
                    ActionId::Eq(EqActionId::IncreaseBig)
                    | ActionId::Eq(EqActionId::DecreaseBig) => 1.0,
                    ActionId::Eq(EqActionId::IncreaseTiny)
                    | ActionId::Eq(EqActionId::DecreaseTiny) => 0.05,
                    _ => 0.1,
                };
            let new_val = if increase {
                (band.q + delta).min(10.0)
            } else {
                (band.q - delta).max(0.1)
            };
            make_set_action(target, effect_id, band_idx, EqualizerParamKind::Q, new_val)
        }
        EqualizerParamKind::Slope => {
            // Cycle slope: increase goes next, decrease goes prev
            let new_slope = if increase {
                band.slope.next()
            } else {
                band.slope.prev()
            };
            make_set_action(
                target,
                effect_id,
                band_idx,
                EqualizerParamKind::Slope,
                new_slope.to_normalized(),
            )
        }
        EqualizerParamKind::BandType => {
            // Cycle band type
            let allowed = EqBandType::allowed_types(band_idx);
            if allowed.len() <= 1 {
                return Action::None;
            }
            let cur_idx = allowed
                .iter()
                .position(|t| *t == band.band_type)
                .unwrap_or(0);
            let next_idx = if increase {
                (cur_idx + 1) % allowed.len()
            } else {
                (cur_idx + allowed.len() - 1) % allowed.len()
            };
            make_set_action(
                target,
                effect_id,
                band_idx,
                EqualizerParamKind::BandType,
                next_idx as f32,
            )
        }
        EqualizerParamKind::Enabled => Action::None,
    }
}

/// Compute biquad magnitude response at a given frequency for one EQ band.
fn band_response_db(band: &EqBand, freq: f32) -> f32 {
    if !band.enabled {
        return 0.0;
    }

    let w = freq / band.freq;
    let q = band.q.max(0.1);

    match band.band_type {
        EqBandType::HighPass => {
            // High-pass: attenuates below cutoff
            let w2 = w * w;
            let order = match band.slope {
                FilterSlope::Db12 => 1,
                FilterSlope::Db24 => 2,
                FilterSlope::Db48 => 4,
            };
            // Each biquad stage: H(w) = w^2 / sqrt((1-w^2)^2 + (w/Q)^2)
            let mag2_single = if w2 < 0.0001 {
                0.0001 // avoid log(0)
            } else {
                let num = w2 * w2;
                let den = (1.0 - w2).powi(2) + (w / q).powi(2);
                (num / den).max(0.0001)
            };
            10.0 * mag2_single.log10() * order as f32
        }
        EqBandType::LowPass => {
            // Low-pass: attenuates above cutoff
            let w2 = w * w;
            let order = match band.slope {
                FilterSlope::Db12 => 1,
                FilterSlope::Db24 => 2,
                FilterSlope::Db48 => 4,
            };
            let mag2_single = {
                let den = (1.0 - w2).powi(2) + (w / q).powi(2);
                (1.0 / den).max(0.0001)
            };
            10.0 * mag2_single.log10() * order as f32
        }
        EqBandType::Bell => {
            if band.gain.abs() < 0.001 {
                return 0.0;
            }
            let a = 10.0_f32.powf(band.gain / 40.0);
            let w2 = w * w;
            let num = (w2 - 1.0).powi(2) + (w * a / q).powi(2);
            let den = (w2 - 1.0).powi(2) + (w / (a * q)).powi(2);
            10.0 * (num / den).log10()
        }
        EqBandType::LowShelf => {
            if band.gain.abs() < 0.001 {
                return 0.0;
            }
            let a = 10.0_f32.powf(band.gain / 20.0);
            let w2 = w * w;
            let blend = 1.0 / (1.0 + w2);
            let lin = a * blend + (1.0 - blend);
            20.0 * lin.log10()
        }
        EqBandType::HighShelf => {
            if band.gain.abs() < 0.001 {
                return 0.0;
            }
            let a = 10.0_f32.powf(band.gain / 20.0);
            let w2 = w * w;
            let blend = w2 / (1.0 + w2);
            let lin = a * blend + (1.0 - blend);
            20.0 * lin.log10()
        }
    }
}

/// Compute composite EQ magnitude response at a given frequency.
fn composite_response_db(eq: &EqConfig, freq: f32) -> f32 {
    eq.bands.iter().map(|b| band_response_db(b, freq)).sum()
}

/// Render the frequency response curve.
#[allow(clippy::too_many_arguments)]
fn render_frequency_curve(
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    eq: &EqConfig,
    selected_band: usize,
    buf: &mut RenderBuf,
    p: &Palette,
) {
    if width < 2 || height < 2 {
        return;
    }

    // Zero-line
    let grid_style = Style::new().fg(Color::new(40, 40, 40));
    let zero_row = y + height / 2;
    for col in x..x + width {
        buf.set_cell(col, zero_row, '-', grid_style);
    }

    // Compute response at each column (log-spaced 20Hz..20kHz)
    let curve_color = Color::new(100, 200, 255);
    let curve_style = Style::new().fg(curve_color);

    let db_range = 24.0_f32;
    let mut responses = Vec::with_capacity(width as usize);
    for col in 0..width {
        let frac = col as f32 / (width - 1) as f32;
        let freq = 20.0 * (1000.0_f32).powf(frac);
        let db = composite_response_db(eq, freq);
        responses.push(db);
    }

    // Map dB to row
    for (col, &db) in responses.iter().enumerate() {
        let frac = (-db / db_range + 1.0) / 2.0;
        let row_f = frac * (height - 1) as f32;
        let row = (row_f.round() as u16).min(height - 1);
        let px = x + col as u16;
        let py = y + row;
        if py >= y && py < y + height {
            let sub = (row_f - row_f.floor()) * 2.0;
            let ch = if sub < 1.0 { '\u{2584}' } else { '\u{2588}' };
            buf.set_cell(px, py, ch, curve_style);
        }
    }

    // Band markers
    for (i, band) in eq.bands.iter().enumerate() {
        let freq_frac = (band.freq / 20.0).log10() / 3.0;
        let col = (freq_frac * (width - 1) as f32).round() as u16;
        if col < width {
            let db = composite_response_db(eq, band.freq);
            let frac = (-db / db_range + 1.0) / 2.0;
            let row = (frac * (height - 1) as f32).round() as u16;
            let px = x + col;
            let py = y + row.min(height - 1);

            let marker_color = if i == selected_band {
                Color::new(255, 200, 50)
            } else if !band.enabled {
                p.dim
            } else {
                p.fg
            };
            buf.set_cell(px, py, '\u{25cf}', Style::new().fg(marker_color));
        }
    }
}

/// Render band info in a single row of 6 bands.
#[allow(clippy::too_many_arguments)]
fn render_band_info(
    x: u16,
    y: u16,
    width: u16,
    eq: &EqConfig,
    selected_band: usize,
    selected_param: usize,
    buf: &mut RenderBuf,
    p: &Palette,
) {
    let band_width = (width / 6).max(10);

    for (i, band) in eq.bands.iter().enumerate() {
        let bx = x + (i as u16) * band_width;
        let is_selected = i == selected_band;

        let type_color = if is_selected {
            Color::new(255, 200, 50)
        } else {
            p.fg
        };

        // Row 0: type + freq (param 0 = freq)
        let label = format!("{} {}", band.band_type.name(), format_freq(band.freq));
        let label_style = Style::new().fg(if is_selected && selected_param == 0 {
            Color::new(255, 200, 50)
        } else {
            type_color
        });
        render_text_at(bx, y, &label, label_style, width, buf);

        if band.band_type.has_slope() {
            // HPF/LPF layout: row 1 = slope, row 2 = enabled
            let slope_str = format!("{}dB/oct", band.slope.name());
            let slope_style = Style::new().fg(if is_selected && selected_param == 1 {
                Color::new(255, 200, 50)
            } else {
                p.fg
            });
            render_text_at(bx, y + 1, &slope_str, slope_style, width, buf);

            let on_str = if band.enabled { "[ON]" } else { "[OFF]" };
            let on_color = if !band.enabled {
                p.dim
            } else if is_selected && selected_param == 2 {
                Color::new(255, 200, 50)
            } else {
                Color::new(80, 200, 80)
            };
            render_text_at(bx, y + 2, on_str, Style::new().fg(on_color), width, buf);
        } else {
            // Bell/Shelf layout: row 1 = gain, row 2 = Q, row 3 = type, row 4 = enabled
            let gain_str = format!("{:+.1}dB", band.gain);
            let gain_style = Style::new().fg(if is_selected && selected_param == 1 {
                Color::new(255, 200, 50)
            } else {
                p.fg
            });
            render_text_at(bx, y + 1, &gain_str, gain_style, width, buf);

            let q_str = format!("Q:{:.2}", band.q);
            let q_style = Style::new().fg(if is_selected && selected_param == 2 {
                Color::new(255, 200, 50)
            } else {
                p.fg
            });
            render_text_at(bx, y + 2, &q_str, q_style, width, buf);

            // Type (only meaningful for bands 1-4 which can switch)
            let allowed = EqBandType::allowed_types(i);
            if allowed.len() > 1 {
                let type_str = format!("[{}]", band.band_type.name());
                let type_style = Style::new().fg(if is_selected && selected_param == 3 {
                    Color::new(255, 200, 50)
                } else {
                    Color::new(120, 160, 200)
                });
                render_text_at(bx, y + 3, &type_str, type_style, width, buf);
            }

            let on_str = if band.enabled { "[ON]" } else { "[OFF]" };
            let on_color = if !band.enabled {
                p.dim
            } else if is_selected && selected_param == 4 {
                Color::new(255, 200, 50)
            } else {
                Color::new(80, 200, 80)
            };
            render_text_at(bx, y + 4, on_str, Style::new().fg(on_color), width, buf);
        }
    }
}

fn render_text_at(x: u16, y: u16, text: &str, style: Style, max_width: u16, buf: &mut RenderBuf) {
    for (i, ch) in text.chars().enumerate() {
        let px = x + i as u16;
        if px < x + max_width {
            buf.set_cell(px, y, ch, style);
        }
    }
}

fn format_freq(freq: f32) -> String {
    if freq >= 1000.0 {
        format!("{:.1}k", freq / 1000.0)
    } else {
        format!("{:.0}", freq)
    }
}
