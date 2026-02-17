use std::path::PathBuf;

use imbolc_types::*;

use super::macro_def::repl_actions;

repl_actions! {
    group "instrument" => DomainAction::Instrument, InstrumentAction {
        "add"                      => Add(source: SourceType);                                         "Add instrument with given source type"
        "delete"                   => Delete(id: InstrumentId);                                        "Delete instrument by ID"
        "select"                   => Select(idx: usize);                                              "Select instrument by index"
        "select-next"              => SelectNext;                                                      "Select next instrument"
        "select-prev"              => SelectPrev;                                                      "Select previous instrument"
        "select-first"             => SelectFirst;                                                     "Select first instrument"
        "select-last"              => SelectLast;                                                      "Select last instrument"
        "add-effect"               => AddEffect(id: InstrumentId, fx: EffectType);                     "Add effect to instrument"
        "remove-effect"            => RemoveEffect(id: InstrumentId, fx_id: EffectId);                 "Remove effect from instrument"
        "move-stage"               => MoveStage(id: InstrumentId, idx: usize, dir: i8);                "Move processing stage up/down"
        "set-filter"               => SetFilter(id: InstrumentId, ft: Option<FilterType>);             "Set filter type (or none)"
        "toggle-effect-bypass"     => ToggleEffectBypass(id: InstrumentId, fx_id: EffectId);           "Toggle effect bypass"
        "toggle-filter"            => ToggleFilter(id: InstrumentId);                                  "Toggle filter on/off"
        "cycle-filter-type"        => NextFilterType(id: InstrumentId);                                "Cycle to next filter type"
        "adjust-filter-cutoff"     => AdjustFilterCutoff(id: InstrumentId, d: f32);                    "Adjust filter cutoff"
        "adjust-filter-resonance"  => AdjustFilterResonance(id: InstrumentId, d: f32);                 "Adjust filter resonance"
        "adjust-effect-param"      => AdjustEffectParam(id: InstrumentId, fx_id: EffectId, pi: ParamIndex, d: f32); "Adjust effect parameter"
        "toggle-arp"               => ToggleArpeggiator(id: InstrumentId);                              "Toggle arpeggiator"
        "cycle-arp-direction"      => NextArpeggiatorDirection(id: InstrumentId);                      "Cycle arp direction forward"
        "cycle-arp-direction-rev"  => PrevArpeggiatorDirection(id: InstrumentId);                      "Cycle arp direction reverse"
        "cycle-arp-rate"           => NextArpeggiatorRate(id: InstrumentId);                           "Cycle arp rate forward"
        "cycle-arp-rate-rev"       => PrevArpeggiatorRate(id: InstrumentId);                           "Cycle arp rate reverse"
        "adjust-arp-octaves"       => AdjustArpeggiatorOctaves(id: InstrumentId, d: i8);                "Adjust arp octave range"
        "adjust-arp-gate"          => AdjustArpeggiatorGate(id: InstrumentId, d: f32);                 "Adjust arp gate length"
        "cycle-chord-shape"        => NextChordShape(id: InstrumentId);                                "Cycle chord shape forward"
        "cycle-chord-shape-rev"    => PrevChordShape(id: InstrumentId);                                "Cycle chord shape reverse"
        "clear-chord-shape"        => ClearChordShape(id: InstrumentId);                               "Clear chord shape"
        "set-eq-param"             => SetEqualizerParam(id: InstrumentId, band: usize, kind: EqualizerParamKind, v: f32); "Set EQ band parameter"
        "toggle-eq"                => ToggleEqualizer(id: InstrumentId);                               "Toggle EQ on/off"
        "link-layer"               => LinkLayer(id: InstrumentId, target: InstrumentId);               "Link instrument to layer group"
        "unlink-layer"             => UnlinkLayer(id: InstrumentId);                                   "Unlink instrument from layer group"
        "adjust-layer-octave"      => AdjustLayerOctaveOffset(id: InstrumentId, d: i8);                "Adjust layer octave offset"
        "adjust-track-swing"       => AdjustTrackSwing(id: InstrumentId, d: f32);                      "Adjust per-track swing"
        "adjust-track-hum-vel"     => AdjustTrackHumanizeVelocity(id: InstrumentId, d: f32);           "Adjust track velocity humanization"
        "adjust-track-hum-time"    => AdjustTrackHumanizeTiming(id: InstrumentId, d: f32);             "Adjust track timing humanization"
        "adjust-track-timing"      => AdjustTrackTimingOffset(id: InstrumentId, d: f32);               "Adjust track timing offset"
        "reset-track-groove"       => ResetTrackGroove(id: InstrumentId);                              "Reset per-track groove settings"
        "cycle-track-time-sig"     => NextTrackTimeSignature(id: InstrumentId);                        "Cycle track time signature"
        "toggle-lfo"               => ToggleLfo(id: InstrumentId);                                     "Toggle LFO on/off"
        "adjust-lfo-rate"          => AdjustLfoRate(id: InstrumentId, d: f32);                         "Adjust LFO rate"
        "adjust-lfo-depth"         => AdjustLfoDepth(id: InstrumentId, d: f32);                        "Adjust LFO depth"
        "set-lfo-shape"            => SetLfoShape(id: InstrumentId, shape: LfoShape);                  "Set LFO waveform shape"
        "set-lfo-target"           => SetLfoTarget(id: InstrumentId, target: ParameterTarget);         "Set LFO modulation target"
        "adjust-attack"            => AdjustEnvelopeAttack(id: InstrumentId, d: f32);                  "Adjust envelope attack"
        "adjust-decay"             => AdjustEnvelopeDecay(id: InstrumentId, d: f32);                   "Adjust envelope decay"
        "adjust-sustain"           => AdjustEnvelopeSustain(id: InstrumentId, d: f32);                 "Adjust envelope sustain"
        "adjust-release"           => AdjustEnvelopeRelease(id: InstrumentId, d: f32);                 "Adjust envelope release"
        "toggle-channel-config"    => ToggleChannelConfig(id: InstrumentId);                           "Toggle mono/stereo channel config"
    }

    group "mixer" => DomainAction::Mixer, MixerAction {
        "move"                  => Move(d: i8);                                    "Move mixer selection"
        "jump"                  => Jump(d: i8);                                    "Jump mixer selection by section"
        "select-at"             => SelectAt(sel: MixerSelection);                  "Select specific mixer channel"
        "adjust-level"          => AdjustLevel(d: f32);                            "Adjust channel level"
        "toggle-mute"           => ToggleMute;                                     "Toggle mute on selected channel"
        "toggle-solo"           => ToggleSolo;                                     "Toggle solo on selected channel"
        "cycle-section"         => NextSection;                                    "Cycle mixer section"
        "cycle-output"          => NextOutput;                                     "Cycle output routing forward"
        "cycle-output-rev"      => PrevOutput;                                     "Cycle output routing reverse"
        "adjust-send"           => AdjustSend(bus: BusId, d: f32);                 "Adjust send level to bus"
        "toggle-send"           => ToggleSend(bus: BusId);                         "Toggle send to bus"
        "cycle-send-tap"        => NextSendTapPoint(bus: BusId);                   "Cycle send tap point"
        "adjust-pan"            => AdjustPan(d: f32);                              "Adjust pan position"
    }

    group "piano-roll" => DomainAction::PianoRoll, PianoRollAction {
        "play-stop"         => TogglePlayback;                                           "Toggle playback"
        "toggle-loop"       => ToggleLoop;                                         "Toggle loop mode"
        "set-loop-start"    => SetLoopStart(tick: u32);                            "Set loop start tick"
        "set-loop-end"      => SetLoopEnd(tick: u32);                              "Set loop end tick"
        "cycle-time-sig"    => NextTimeSig;                                       "Cycle time signature"
        "toggle-poly-mode"  => TogglePolyMode(track: usize);                      "Toggle polyphonic mode"
        "play-stop-record"  => ToggleRecordPlayback;                                "Toggle record mode"
        "adjust-swing"      => AdjustSwing(d: f32);                                "Adjust swing amount"
        "render"            => RenderToWav(id: InstrumentId);                      "Render track to WAV"
        "bounce"            => BounceToWav;                                         "Bounce master to WAV"
        "export-stems"      => ExportStems;                                         "Export individual stems"
        "cancel-export"     => CancelExport;                                        "Cancel export in progress"
    }

    group "sequencer" => DomainAction::Sequencer, SequencerAction {
        "toggle-step"           => ToggleStep(pad: usize, step: usize);                        "Toggle sequencer step"
        "adjust-velocity"       => AdjustVelocity(pad: usize, step: usize, d: i8);             "Adjust step velocity"
        "play-stop"             => TogglePlayback;                                               "Toggle sequencer playback"
        "clear-pad"             => ClearPad(pad: usize);                                       "Clear all steps on pad"
        "clear-pattern"         => ClearPattern;                                                "Clear entire pattern"
        "cycle-pattern-length"  => NextPatternLength;                                          "Cycle pattern length"
        "next-pattern"          => NextPattern;                                                  "Go to next pattern"
        "prev-pattern"          => PrevPattern;                                                  "Go to previous pattern"
        "adjust-pad-level"      => AdjustPadLevel(pad: usize, d: f32);                         "Adjust pad level"
        "adjust-swing"          => AdjustSwing(d: f32);                                         "Adjust sequencer swing"
        "adjust-probability"    => AdjustProbability(pad: usize, step: usize, d: f32);          "Adjust step probability"
        "toggle-chain"          => ToggleChain;                                                  "Toggle pattern chain mode"
        "add-chain-step"        => AddChainStep(pat: usize);                                    "Add pattern to chain"
        "remove-chain-step"     => RemoveChainStep(pos: usize);                                 "Remove step from chain"
        "toggle-reverse"        => ToggleReverse(pad: usize);                                   "Toggle pad reverse playback"
        "adjust-pad-pitch"      => AdjustPadPitch(pad: usize, d: i8);                          "Adjust pad pitch"
        "adjust-step-pitch"     => AdjustStepPitch(pad: usize, step: usize, d: i8);            "Adjust step pitch offset"
        "cycle-step-resolution" => NextStepResolution;                                         "Cycle step resolution"
    }

    group "arrangement" => DomainAction::Arrangement, ArrangementAction {
        "toggle-play-mode"       => TogglePlayMode;                                "Toggle arrangement play mode"
        "delete-clip"            => DeleteClip(id: ClipId);                        "Delete a clip"
        "remove-clip-instance"   => RemoveClipInstance(id: ClipInstanceId);         "Remove a clip instance"
        "duplicate-clip-instance" => DuplicateClipInstance(id: ClipInstanceId);    "Duplicate a clip instance"
        "move-cursor"            => MoveCursor(d: i32);                            "Move arrangement cursor"
        "scroll-view"            => ScrollView(d: i32);                            "Scroll arrangement view"
        "zoom-in"                => ZoomIn;                                         "Zoom in on arrangement"
        "zoom-out"               => ZoomOut;                                        "Zoom out on arrangement"
        "enter-clip-edit"        => EnterClipEdit(id: ClipId);                     "Enter clip editing mode"
        "exit-clip-edit"         => ExitClipEdit;                                   "Exit clip editing mode"
        "play-stop"              => TogglePlayback;                                       "Toggle arrangement playback"
        "select-lane"            => SelectLane(idx: usize);                        "Select arrangement lane"
    }

    group "automation" => DomainAction::Automation, AutomationAction {
        "remove-lane"           => RemoveLane(id: u32);                                    "Remove automation lane"
        "toggle-lane-enabled"   => ToggleLaneEnabled(id: u32);                             "Toggle lane enabled"
        "add-point"             => AddPoint(lane: u32, tick: u32, val: f32);                "Add automation point"
        "remove-point"          => RemovePoint(lane: u32, tick: u32);                       "Remove automation point"
        "set-curve-type"        => SetCurveType(lane: u32, tick: u32, curve: CurveType);    "Set point curve type"
        "select-lane"           => SelectLane(d: i8);                                       "Select next/prev lane"
        "clear-lane"            => ClearLane(id: u32);                                      "Clear all points on lane"
        "toggle-recording"      => ToggleRecording;                                         "Toggle automation recording"
        "toggle-lane-arm"       => ToggleLaneArm(id: u32);                                 "Toggle lane arm for recording"
        "arm-all"               => ArmAllLanes;                                             "Arm all lanes for recording"
        "disarm-all"            => DisarmAllLanes;                                           "Disarm all lanes"
    }

    group "session" => DomainAction::Session, SessionAction {
        "save"                  => Save;                                           "Save project"
        "save-as"               => SaveAs(path: PathBuf);                          "Save project to path"
        "load"                  => Load;                                           "Load project (from default)"
        "load-from"             => LoadFrom(path: PathBuf);                        "Load project from path"
        "new"                   => NewProject;                                     "Create new project"
        "adjust-hum-vel"        => AdjustHumanizeVelocity(d: f32);                "Adjust global velocity humanization"
        "adjust-hum-time"       => AdjustHumanizeTiming(d: f32);                  "Adjust global timing humanization"
        "toggle-master-mute"    => ToggleMasterMute;                              "Toggle master mute"
        "cycle-theme"           => NextTheme;                                    "Cycle UI theme"
        "create-checkpoint"     => CreateCheckpoint(name: String);                "Create named checkpoint"
        "restore-checkpoint"    => RestoreCheckpoint(id: i64);                    "Restore checkpoint by ID"
        "delete-checkpoint"     => DeleteCheckpoint(id: i64);                     "Delete checkpoint by ID"
    }

    group "server" => DomainAction::Server, ServerAction {
        "connect"               => Connect;                                        "Connect to SuperCollider server"
        "disconnect"            => Disconnect;                                     "Disconnect from server"
        "stop"                  => Stop;                                           "Stop the server"
        "compile-synthdefs"     => CompileSynthDefs;                              "Compile SynthDefs"
        "compile-vst-synthdefs" => CompileVstSynthDefs;                           "Compile VST SynthDefs"
        "load-synthdefs"        => LoadSynthDefs;                                 "Load SynthDefs into server"
        "record-master"         => ToggleRecordMaster;                             "Toggle master recording"
        "record-input"          => ToggleRecordInput;                              "Toggle input recording"
    }

    group "bus" => DomainAction::Bus, BusAction {
        "add"                   => Add;                                            "Add a new bus"
        "remove"                => Remove(id: BusId);                              "Remove a bus"
        "rename"                => Rename(id: BusId, name: String);                "Rename a bus"
        "add-effect"            => AddEffect(id: BusId, fx: EffectType);           "Add effect to bus"
        "remove-effect"         => RemoveEffect(id: BusId, fx_id: EffectId);       "Remove effect from bus"
        "move-effect"           => MoveEffect(id: BusId, fx_id: EffectId, d: i8);  "Move bus effect up/down"
        "toggle-effect-bypass"  => ToggleEffectBypass(id: BusId, fx_id: EffectId); "Toggle bus effect bypass"
        "adjust-effect-param"   => AdjustEffectParam(id: BusId, fx_id: EffectId, pi: ParamIndex, d: f32); "Adjust bus effect parameter"
    }

    group "layer-group" => DomainAction::Group, GroupAction {
        "add-effect"            => AddEffect(gid: u32, fx: EffectType);                            "Add effect to layer group"
        "remove-effect"         => RemoveEffect(gid: u32, fx_id: EffectId);                        "Remove effect from layer group"
        "move-effect"           => MoveEffect(gid: u32, fx_id: EffectId, d: i8);                   "Move layer group effect"
        "toggle-effect-bypass"  => ToggleEffectBypass(gid: u32, fx_id: EffectId);                  "Toggle layer group effect bypass"
        "adjust-effect-param"   => AdjustEffectParam(gid: u32, fx_id: EffectId, pi: ParamIndex, d: f32); "Adjust layer group effect param"
        "toggle-eq"             => ToggleEqualizer(gid: u32);                                      "Toggle layer group EQ"
        "set-eq-param"          => SetEqualizerParam(gid: u32, band: usize, kind: EqualizerParamKind, v: f32); "Set layer group EQ band param"
        "rename"                => Rename(gid: u32, name: String);                                  "Rename layer group"
    }

    group "generative" => DomainAction::Generative, GenerativeAction {
        "toggle"                    => ToggleEnabled;                                       "Toggle generative engine"
        "toggle-capture"            => ToggleCapture;                                       "Toggle capture mode"
        "commit-capture"            => CommitCapture;                                       "Commit captured events"
        "clear-capture"             => ClearCapture;                                        "Clear captured events"
        "remove-voice"              => RemoveVoice(id: GenVoiceId);                         "Remove a generative voice"
        "toggle-voice"              => ToggleVoice(id: GenVoiceId);                         "Toggle voice on/off"
        "mute-voice"                => MuteVoice(id: GenVoiceId);                           "Mute a voice"
        "adjust-density"            => AdjustDensity(d: f32);                               "Adjust density macro"
        "adjust-chaos"              => AdjustChaos(d: f32);                                 "Adjust chaos macro"
        "adjust-energy"             => AdjustEnergy(d: f32);                                "Adjust energy macro"
        "adjust-motion"             => AdjustMotion(d: f32);                                "Adjust motion macro"
        "toggle-scale-lock"         => ToggleScaleLock;                                     "Toggle scale lock"
        "adjust-pitch-min"          => AdjustPitchMin(d: i8);                               "Adjust minimum pitch"
        "adjust-pitch-max"          => AdjustPitchMax(d: i8);                               "Adjust maximum pitch"
        "adjust-max-notes"          => AdjustMaxNotesPerBeat(d: i8);                        "Adjust max notes per beat"
        "adjust-gen-hum-time"       => AdjustHumanizeTiming(d: f32);                        "Adjust generative timing humanization"
        "adjust-gen-hum-vel"        => AdjustHumanizeVelocity(d: f32);                      "Adjust generative velocity humanization"
        "set-euclidean-pulses"      => SetEuclideanPulses(id: GenVoiceId, n: u8);            "Set Euclidean pulse count"
        "set-euclidean-steps"       => SetEuclideanSteps(id: GenVoiceId, n: u8);             "Set Euclidean step count"
        "set-euclidean-rotation"    => SetEuclideanRotation(id: GenVoiceId, n: u8);           "Set Euclidean rotation"
        "cycle-euclidean-pitch"     => NextEuclideanPitchMode(id: GenVoiceId);              "Cycle Euclidean pitch mode"
        "cycle-voice-rate"          => NextVoiceRate(id: GenVoiceId);                       "Cycle voice rate forward"
        "cycle-voice-rate-rev"      => PrevVoiceRate(id: GenVoiceId);                "Cycle voice rate reverse"
        "adjust-markov-rest"        => AdjustMarkovRestProb(id: GenVoiceId, d: f32);         "Adjust Markov rest probability"
        "cycle-markov-duration"     => NextMarkovDurationMode(id: GenVoiceId);              "Cycle Markov duration mode"
        "randomize-markov"          => RandomizeMarkovMatrix(id: GenVoiceId);                 "Randomize Markov transition matrix"
        "set-lsystem-iterations"    => SetLSystemIterations(id: GenVoiceId, n: u8);           "Set L-System iterations"
        "adjust-lsystem-step"       => AdjustLSystemStepInterval(id: GenVoiceId, d: i8);      "Adjust L-System step interval"
        "remove-lsystem-rule"       => RemoveLSystemRule(id: GenVoiceId, idx: usize);         "Remove L-System rule by index"
    }

    group "slicer" => DomainAction::SampleSlicer, SampleSlicerAction {
        "add-slice"         => AddSlice(pos: f32);                     "Add slice at cursor position"
        "remove-slice"      => RemoveSlice;                            "Remove selected slice"
        "assign-to-pad"     => AssignToPad(pad: usize);               "Assign slice to pad"
        "auto-slice"        => AutoSlice(n: usize);                    "Auto-slice into N segments"
        "preview-slice"     => PreviewSlice;                           "Preview selected slice"
        "select-slice"      => SelectSlice(d: i8);                     "Select next/prev slice"
        "nudge-start"       => NudgeSliceStart(d: f32);                "Nudge slice start point"
        "nudge-end"         => NudgeSliceEnd(d: f32);                  "Nudge slice end point"
        "move-cursor"       => MoveCursor(d: i8);                      "Move slicer cursor"
        "commit-all"        => CommitAll;                              "Assign all slices to pads"
    }

    group "vst-param" => DomainAction::VstParam, VstParamAction {
        "set"           => SetParam(id: InstrumentId, target: VstTarget, idx: u32, val: f32);      "Set VST parameter value"
        "adjust"        => AdjustParam(id: InstrumentId, target: VstTarget, idx: u32, d: f32);     "Adjust VST parameter"
        "reset"         => ResetParam(id: InstrumentId, target: VstTarget, idx: u32);               "Reset VST parameter to default"
        "discover"      => DiscoverParams(id: InstrumentId, target: VstTarget);                     "Discover VST parameters"
        "save-state"    => SaveState(id: InstrumentId, target: VstTarget);                          "Save VST plugin state"
    }

    group "click" => DomainAction::Click, ClickAction {
        "toggle"        => Toggle;                                     "Toggle click track"
        "toggle-mute"   => ToggleMute;                                "Toggle click mute"
        "adjust-volume" => AdjustVolume(d: f32);                      "Adjust click volume"
        "set-volume"    => SetVolume(v: f32);                         "Set click volume"
    }

    group "tuner" => DomainAction::Tuner, TunerAction {
        "play"  => PlayTone(freq: f32);                                "Play reference tone at frequency"
        "stop"  => StopTone;                                           "Stop reference tone"
    }

    group "midi" => DomainAction::Midi, MidiAction {
        "connect"             => ConnectPort(port: usize);                          "Connect to MIDI port by index"
        "disconnect"          => DisconnectPort;                                    "Disconnect MIDI port"
        "toggle-passthrough"  => ToggleNotePassthrough;                            "Toggle MIDI note passthrough"
    }
}

// Hand-written parse for Option<FilterType> since the macro can't do Option directly
impl super::parse::ReplParseable for Option<FilterType> {
    fn parse_repl(s: &str) -> Result<Self, String> {
        if s == "none" || s == "None" {
            Ok(None)
        } else {
            FilterType::from_name(s).map(Some).ok_or_else(|| {
                format!("unknown filter type: {s} (try: lpf, hpf, bpf, notch, none)")
            })
        }
    }
    fn type_hint() -> &'static str {
        "<filter|none>"
    }
}

// Hand-written parse for i64 (used by checkpoint actions)
impl super::parse::ReplParseable for i64 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid i64: {s}"))
    }
    fn type_hint() -> &'static str {
        "<id>"
    }
}

// ClipId and ClipInstanceId are u32 type aliases, already covered
// AutomationLaneId is u32, already covered

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_instrument_add() {
        let action = parse_action("instrument add saw").unwrap();
        assert!(matches!(
            action,
            DomainAction::Instrument(InstrumentAction::Add(SourceType::Saw))
        ));
    }

    #[test]
    fn parse_instrument_select_next() {
        let action = parse_action("instrument select-next").unwrap();
        assert!(matches!(
            action,
            DomainAction::Instrument(InstrumentAction::SelectNext)
        ));
    }

    #[test]
    fn parse_instrument_adjust_filter_cutoff() {
        let action = parse_action("instrument adjust-filter-cutoff 1 0.5").unwrap();
        match action {
            DomainAction::Instrument(InstrumentAction::AdjustFilterCutoff(id, d)) => {
                assert_eq!(id, InstrumentId::new(1));
                assert!((d - 0.5).abs() < f32::EPSILON);
            }
            _ => panic!("wrong action"),
        }
    }

    #[test]
    fn parse_mixer_toggle_mute() {
        let action = parse_action("mixer toggle-mute").unwrap();
        assert!(matches!(
            action,
            DomainAction::Mixer(MixerAction::ToggleMute)
        ));
    }

    #[test]
    fn parse_click_toggle() {
        let action = parse_action("click toggle").unwrap();
        assert!(matches!(action, DomainAction::Click(ClickAction::Toggle)));
    }

    #[test]
    fn parse_unknown_group() {
        assert!(parse_action("nonexistent foo").is_err());
    }

    #[test]
    fn parse_unknown_subcommand() {
        assert!(parse_action("instrument nonexistent").is_err());
    }

    #[test]
    fn parse_missing_args() {
        assert!(parse_action("instrument add").is_err());
    }

    #[test]
    fn all_commands_non_empty() {
        let cmds = all_commands();
        assert!(cmds.len() > 100);
    }

    #[test]
    fn complete_groups() {
        let completions = complete_command("ins");
        assert!(completions.contains(&"instrument".to_string()));
    }

    #[test]
    fn complete_subcommands() {
        let completions = complete_command("instrument tog");
        assert!(completions.iter().any(|c| c.contains("toggle")));
    }
}
