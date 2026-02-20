//! Typed action identifiers for the keybinding system.
//!
//! This module defines enums for all action identifiers used in the keybinding system,
//! replacing the previous string-based approach with type-safe enums.

use imbolc_types::PaneId;

/// Macro to generate action enums with string conversion methods
macro_rules! define_action_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $variant:ident => $str:literal, )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $( $variant, )*
        }
        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $( $name::$variant => $str, )*
                }
            }
            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $( $str => Some($name::$variant), )*
                    _ => None,
                }
            }
        }
    }
}

/// Global layer actions (available in all contexts)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalActionId {
    Undo,
    Redo,
    Quit,
    Save,
    Load,
    SaveAs,
    MasterMute,
    ToggleRecordMaster,
    Copy,
    Cut,
    Paste,
    SelectAll,
    AddInstrument,
    DeleteInstrument,
    NavBack,
    NavForward,
    Help,
    CommandLine,
    CommandPalette,
    TogglePianoMode,
    OpenProjectBrowser,
    Escape,
    SelectPrevInstrument,
    SelectNextInstrument,
    SelectTwoDigit,
    TogglePlayback,
    RefreshScreen,
    ClickTrackToggle,
    NextTheme,
    RequestPrivilege,
    OpenCheckpointList,
    SwitchPane(PaneId),
    SwitchPianoRollOrSequencer,
    SelectInstrument(u8), // 1-10
}

impl GlobalActionId {
    pub fn as_str(&self) -> &'static str {
        match self {
            GlobalActionId::Undo => "undo",
            GlobalActionId::Redo => "redo",
            GlobalActionId::Quit => "quit",
            GlobalActionId::Save => "save",
            GlobalActionId::Load => "load",
            GlobalActionId::SaveAs => "save_as",
            GlobalActionId::MasterMute => "master_mute",
            GlobalActionId::ToggleRecordMaster => "record_master",
            GlobalActionId::Copy => "copy",
            GlobalActionId::Cut => "cut",
            GlobalActionId::Paste => "paste",
            GlobalActionId::SelectAll => "select_all",
            GlobalActionId::AddInstrument => "add_instrument",
            GlobalActionId::DeleteInstrument => "delete_instrument",
            GlobalActionId::NavBack => "nav_back",
            GlobalActionId::NavForward => "nav_forward",
            GlobalActionId::Help => "help",
            GlobalActionId::CommandLine => "command_line",
            GlobalActionId::CommandPalette => "command_palette",
            GlobalActionId::TogglePianoMode => "toggle_piano_mode",
            GlobalActionId::OpenProjectBrowser => "open_project_browser",
            GlobalActionId::Escape => "escape",
            GlobalActionId::TogglePlayback => "play_stop",
            GlobalActionId::SelectPrevInstrument => "select_prev_instrument",
            GlobalActionId::SelectNextInstrument => "select_next_instrument",
            GlobalActionId::SelectTwoDigit => "select_two_digit",
            GlobalActionId::RefreshScreen => "refresh_screen",
            GlobalActionId::ClickTrackToggle => "click_track_toggle",
            GlobalActionId::NextTheme => "cycle_theme",
            GlobalActionId::RequestPrivilege => "request_privilege",
            GlobalActionId::OpenCheckpointList => "open_checkpoint_list",
            GlobalActionId::SwitchPane(pane) => match pane {
                PaneId::Home => "switch:home",
                PaneId::InstrumentEdit => "switch:instrument",
                PaneId::PianoRoll => "switch:piano_roll",
                PaneId::Sequencer => "switch:sequencer",
                PaneId::Waveform => "switch:waveform",
                PaneId::Track => "switch:track",
                PaneId::Mixer => "switch:mixer",
                PaneId::Server => "switch:server",
                PaneId::Automation => "switch:automation",
                PaneId::Eq => "switch:eq",
                PaneId::MasterEq => "switch:master_eq",
                PaneId::SampleSlicer => "switch:sample_slicer",
                PaneId::FrameEdit => "switch:frame_edit",
                PaneId::MidiSettings => "switch:midi_settings",
                PaneId::Generative => "switch:generative",
                PaneId::Arpeggiator => "switch:arpeggiator",
                PaneId::VstParams => "switch:vst_params",
                PaneId::FileBrowser => "switch:file_browser",
                PaneId::Groove => "switch:groove",
                PaneId::Tuner => "switch:tuner",
                PaneId::TagView => "switch:tag_view",
                // Non-switchable panes should not appear here, but handle gracefully
                _ => "switch:unknown",
            },
            GlobalActionId::SwitchPianoRollOrSequencer => "switch:piano_roll_or_sequencer",
            GlobalActionId::SelectInstrument(n) => match n {
                1 => "select:1",
                2 => "select:2",
                3 => "select:3",
                4 => "select:4",
                5 => "select:5",
                6 => "select:6",
                7 => "select:7",
                8 => "select:8",
                9 => "select:9",
                10 => "select:10",
                _ => "select:invalid",
            },
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "undo" => Some(GlobalActionId::Undo),
            "redo" => Some(GlobalActionId::Redo),
            "quit" => Some(GlobalActionId::Quit),
            "save" => Some(GlobalActionId::Save),
            "load" => Some(GlobalActionId::Load),
            "save_as" => Some(GlobalActionId::SaveAs),
            "master_mute" => Some(GlobalActionId::MasterMute),
            "record_master" => Some(GlobalActionId::ToggleRecordMaster),
            "copy" => Some(GlobalActionId::Copy),
            "cut" => Some(GlobalActionId::Cut),
            "paste" => Some(GlobalActionId::Paste),
            "select_all" => Some(GlobalActionId::SelectAll),
            "add_instrument" => Some(GlobalActionId::AddInstrument),
            "delete_instrument" => Some(GlobalActionId::DeleteInstrument),
            "nav_back" => Some(GlobalActionId::NavBack),
            "nav_forward" => Some(GlobalActionId::NavForward),
            "help" => Some(GlobalActionId::Help),
            "command_line" => Some(GlobalActionId::CommandLine),
            "command_palette" => Some(GlobalActionId::CommandPalette),
            "toggle_piano_mode" => Some(GlobalActionId::TogglePianoMode),
            "open_project_browser" => Some(GlobalActionId::OpenProjectBrowser),
            "escape" => Some(GlobalActionId::Escape),
            "play_stop" => Some(GlobalActionId::TogglePlayback),
            "select_prev_instrument" => Some(GlobalActionId::SelectPrevInstrument),
            "select_next_instrument" => Some(GlobalActionId::SelectNextInstrument),
            "select_two_digit" => Some(GlobalActionId::SelectTwoDigit),
            "refresh_screen" => Some(GlobalActionId::RefreshScreen),
            "click_track_toggle" => Some(GlobalActionId::ClickTrackToggle),
            "cycle_theme" => Some(GlobalActionId::NextTheme),
            "request_privilege" => Some(GlobalActionId::RequestPrivilege),
            "open_checkpoint_list" => Some(GlobalActionId::OpenCheckpointList),
            "switch:home" => Some(GlobalActionId::SwitchPane(PaneId::Home)),
            "switch:instrument" => Some(GlobalActionId::SwitchPane(PaneId::InstrumentEdit)),
            "switch:piano_roll" => Some(GlobalActionId::SwitchPane(PaneId::PianoRoll)),
            "switch:piano_roll_or_sequencer" => Some(GlobalActionId::SwitchPianoRollOrSequencer),
            "switch:sequencer" => Some(GlobalActionId::SwitchPane(PaneId::Sequencer)),
            "switch:waveform" => Some(GlobalActionId::SwitchPane(PaneId::Waveform)),
            "switch:track" => Some(GlobalActionId::SwitchPane(PaneId::Track)),
            "switch:mixer" => Some(GlobalActionId::SwitchPane(PaneId::Mixer)),
            "switch:server" => Some(GlobalActionId::SwitchPane(PaneId::Server)),
            "switch:automation" => Some(GlobalActionId::SwitchPane(PaneId::Automation)),
            "switch:eq" => Some(GlobalActionId::SwitchPane(PaneId::Eq)),
            "switch:master_eq" => Some(GlobalActionId::SwitchPane(PaneId::MasterEq)),
            "switch:sample_slicer" => Some(GlobalActionId::SwitchPane(PaneId::SampleSlicer)),
            "switch:frame_edit" => Some(GlobalActionId::SwitchPane(PaneId::FrameEdit)),
            "switch:midi_settings" => Some(GlobalActionId::SwitchPane(PaneId::MidiSettings)),
            "switch:generative" => Some(GlobalActionId::SwitchPane(PaneId::Generative)),
            "switch:arpeggiator" => Some(GlobalActionId::SwitchPane(PaneId::Arpeggiator)),
            "switch:vst_params" => Some(GlobalActionId::SwitchPane(PaneId::VstParams)),
            "switch:file_browser" => Some(GlobalActionId::SwitchPane(PaneId::FileBrowser)),
            "switch:groove" => Some(GlobalActionId::SwitchPane(PaneId::Groove)),
            "switch:tuner" => Some(GlobalActionId::SwitchPane(PaneId::Tuner)),
            "switch:tag_view" => Some(GlobalActionId::SwitchPane(PaneId::TagView)),
            "select:1" => Some(GlobalActionId::SelectInstrument(1)),
            "select:2" => Some(GlobalActionId::SelectInstrument(2)),
            "select:3" => Some(GlobalActionId::SelectInstrument(3)),
            "select:4" => Some(GlobalActionId::SelectInstrument(4)),
            "select:5" => Some(GlobalActionId::SelectInstrument(5)),
            "select:6" => Some(GlobalActionId::SelectInstrument(6)),
            "select:7" => Some(GlobalActionId::SelectInstrument(7)),
            "select:8" => Some(GlobalActionId::SelectInstrument(8)),
            "select:9" => Some(GlobalActionId::SelectInstrument(9)),
            "select:10" => Some(GlobalActionId::SelectInstrument(10)),
            _ => None,
        }
    }
}

define_action_enum! {
    /// Track edit layer actions
    pub enum TrackEditActionId {
        Next => "next",
        Prev => "prev",
        NextSection => "next_section",
        PrevSection => "prev_section",
        Decrease => "decrease",
        Increase => "increase",
        IncreaseBig => "increase_big",
        DecreaseBig => "decrease_big",
        IncreaseTiny => "increase_tiny",
        DecreaseTiny => "decrease_tiny",
        IncreaseMusical => "increase_musical",
        DecreaseMusical => "decrease_musical",
        EnterEdit => "enter_edit",
        ToggleFilter => "toggle_filter",
        NextFilterType => "cycle_filter_type",
        AddEffect => "add_effect",
        RemoveEffect => "remove_effect",
        TogglePoly => "toggle_poly",
        ZeroParam => "zero_param",
        ZeroSection => "zero_section",
        ResetParam => "reset_param",
        ToggleEq => "toggle_eq",
        ToggleLfo => "toggle_lfo",
        CycleLfoShape => "cycle_lfo_shape",
        CycleLfoTarget => "cycle_lfo_target",
        ToggleActive => "toggle_active",
        ToggleChannelConfig => "toggle_channel_config",
        LoadSample => "load_sample",
        VstParams => "vst_params",
        MoveStageUp => "move_stage_up",
        MoveStageDown => "move_stage_down",
        ToggleEffectBypass => "toggle_effect_bypass",
        Done => "done",
        TagParam => "tag_param",
        OpenEq => "open_eq",
        ExportPatch => "export_patch",
    }
}

define_action_enum! {
    /// Mixer layer actions
    pub enum MixerActionId {
        Prev => "prev",
        Next => "next",
        First => "first",
        Last => "last",
        LevelUp => "level_up",
        LevelDown => "level_down",
        LevelUpBig => "level_up_big",
        LevelDownBig => "level_down_big",
        Mute => "mute",
        Solo => "solo",
        Output => "output",
        OutputRev => "output_rev",
        Section => "section",
        SectionPrev => "section_prev",
        SendNext => "send_next",
        SendPrev => "send_prev",
        SendToggle => "send_toggle",
        Escape => "escape",
        EnterDetail => "enter_detail",
        AddEffect => "add_effect",
        RemoveEffect => "remove_effect",
        ToggleEffect => "toggle_effect",
        ToggleFilter => "toggle_filter",
        NextFilterType => "cycle_filter_type",
        FineLeft => "fine_left",
        FineRight => "fine_right",
        MoveUp => "move_up",
        MoveDown => "move_down",
        PanLeft => "pan_left",
        PanRight => "pan_right",
        ClearSend => "clear_send",
        Increase => "increase",
        Decrease => "decrease",
        OpenMasterEq => "open_master_eq",
    }
}

define_action_enum! {
    /// Piano roll layer actions
    pub enum PianoRollActionId {
        Up => "up",
        Down => "down",
        Left => "left",
        Right => "right",
        ToggleNote => "toggle_note",
        VelUp => "vel_up",
        VelDown => "vel_down",
        TogglePlayback => "play_stop",
        Loop => "loop",
        LoopStart => "loop_start",
        LoopEnd => "loop_end",
        OctaveUp => "octave_up",
        OctaveDown => "octave_down",
        Home => "home",
        End => "end",
        ZoomIn => "zoom_in",
        ZoomOut => "zoom_out",
        TimeSig => "time_sig",
        TogglePoly => "toggle_poly",
        GrowDuration => "grow_duration",
        ShrinkDuration => "shrink_duration",
        SelectUp => "select_up",
        SelectDown => "select_down",
        SelectLeft => "select_left",
        SelectRight => "select_right",
        ToggleAutomation => "toggle_automation",
        AutomationLanePrev => "automation_lane_prev",
        AutomationLaneNext => "automation_lane_next",
        RenderToWav => "render_to_wav",
        BounceToWav => "bounce_to_wav",
        ExportStems => "export_stems",
        AutomationSwitchFocus => "automation_switch_focus",
        AutomationDeletePoint => "automation_delete_point",
        AutomationCycleCurve => "automation_cycle_curve",
        AutomationClearLane => "automation_clear_lane",
        AutomationAddLane => "automation_add_lane",
        AutomationRemoveLane => "automation_remove_lane",
        AutomationToggleEnabled => "automation_toggle_enabled",
        AutomationToggleRecording => "automation_toggle_recording",
        AutomationToggleArm => "automation_toggle_arm",
        NextClip => "next_clip",
        PrevClip => "prev_clip",
    }
}

define_action_enum! {
    /// Sequencer layer actions
    pub enum SequencerActionId {
        Up => "up",
        Down => "down",
        Left => "left",
        Right => "right",
        Toggle => "toggle",
        TogglePlayback => "play_stop",
        LoadSample => "load_sample",
        Chopper => "slicer",
        ClearPad => "clear_pad",
        ClearPattern => "clear_pattern",
        PrevPattern => "prev_pattern",
        NextPattern => "next_pattern",
        CycleLength => "cycle_length",
        SelectUp => "select_up",
        SelectDown => "select_down",
        SelectLeft => "select_left",
        SelectRight => "select_right",
        PadLevelDown => "pad_level_down",
        PadLevelUp => "pad_level_up",
        ToggleReverse => "toggle_reverse",
        PitchUp => "pitch_up",
        PitchDown => "pitch_down",
        PitchUpOctave => "pitch_up_octave",
        PitchDownOctave => "pitch_down_octave",
        VelUp => "vel_up",
        VelDown => "vel_down",
        StepPitchUp => "step_pitch_up",
        StepPitchDown => "step_pitch_down",
        AssignInstrument => "assign_instrument",
        ClearInstrument => "clear_instrument",
        FreqUp => "freq_up",
        FreqDown => "freq_down",
        CycleGrid => "cycle_grid",
        OpenGroove => "open_groove",
    }
}

define_action_enum! {
    /// Server layer actions
    pub enum ServerActionId {
        Start => "start",
        Stop => "stop",
        Connect => "connect",
        Disconnect => "disconnect",
        Compile => "compile",
        CompileVst => "compile_vst",
        LoadSynthDefs => "load_synthdefs",
        RefreshDevices => "refresh_devices",
        ToggleRecordMaster => "record_master",
        NextSection => "next_section",
    }
}

define_action_enum! {
    /// Add layer actions (shared by add and add_effect layers)
    pub enum AddActionId {
        Confirm => "confirm",
        Cancel => "cancel",
        Next => "next",
        Prev => "prev",
    }
}

define_action_enum! {
    /// Home layer actions
    pub enum HomeActionId {
        Up => "up",
        Down => "down",
        Select => "select",
        Quit => "quit",
    }
}

define_action_enum! {
    /// Help layer actions
    pub enum HelpActionId {
        Close => "close",
        Up => "up",
        Down => "down",
        Top => "top",
        Bottom => "bottom",
    }
}

define_action_enum! {
    /// Frame edit layer actions
    pub enum FrameEditActionId {
        Prev => "prev",
        Next => "next",
        Decrease => "decrease",
        Increase => "increase",
        Confirm => "confirm",
        Cancel => "cancel",
        TapTempo => "tap_tempo",
    }
}

define_action_enum! {
    /// File browser layer actions
    pub enum FileBrowserActionId {
        Select => "select",
        Cancel => "cancel",
        Parent => "parent",
        Next => "next",
        Prev => "prev",
        Home => "home",
        GotoTop => "goto_top",
        GotoBottom => "goto_bottom",
        ToggleHidden => "toggle_hidden",
    }
}

/// Sample chopper layer actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleSlicerActionId {
    MoveLeft,
    MoveRight,
    NextSlice,
    PrevSlice,
    Chop,
    Delete,
    AutoSlice,
    LoadSample,
    Preview,
    Commit,
    Back,
    NudgeStart,
    NudgeEnd,
    AssignToPad(u8), // 1-12
}

impl SampleSlicerActionId {
    pub fn as_str(&self) -> &'static str {
        match self {
            SampleSlicerActionId::MoveLeft => "move_left",
            SampleSlicerActionId::MoveRight => "move_right",
            SampleSlicerActionId::NextSlice => "next_slice",
            SampleSlicerActionId::PrevSlice => "prev_slice",
            SampleSlicerActionId::Chop => "chop",
            SampleSlicerActionId::Delete => "delete",
            SampleSlicerActionId::AutoSlice => "auto_slice",
            SampleSlicerActionId::LoadSample => "load_sample",
            SampleSlicerActionId::Preview => "preview",
            SampleSlicerActionId::Commit => "commit",
            SampleSlicerActionId::Back => "back",
            SampleSlicerActionId::NudgeStart => "nudge_start",
            SampleSlicerActionId::NudgeEnd => "nudge_end",
            SampleSlicerActionId::AssignToPad(n) => match n {
                1 => "assign_1",
                2 => "assign_2",
                3 => "assign_3",
                4 => "assign_4",
                5 => "assign_5",
                6 => "assign_6",
                7 => "assign_7",
                8 => "assign_8",
                9 => "assign_9",
                10 => "assign_10",
                11 => "assign_11",
                12 => "assign_12",
                _ => "assign_invalid",
            },
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "move_left" => Some(SampleSlicerActionId::MoveLeft),
            "move_right" => Some(SampleSlicerActionId::MoveRight),
            "next_slice" => Some(SampleSlicerActionId::NextSlice),
            "prev_slice" => Some(SampleSlicerActionId::PrevSlice),
            "chop" => Some(SampleSlicerActionId::Chop),
            "delete" => Some(SampleSlicerActionId::Delete),
            "auto_slice" => Some(SampleSlicerActionId::AutoSlice),
            "load_sample" => Some(SampleSlicerActionId::LoadSample),
            "preview" => Some(SampleSlicerActionId::Preview),
            "commit" => Some(SampleSlicerActionId::Commit),
            "back" => Some(SampleSlicerActionId::Back),
            "nudge_start" => Some(SampleSlicerActionId::NudgeStart),
            "nudge_end" => Some(SampleSlicerActionId::NudgeEnd),
            "assign_1" => Some(SampleSlicerActionId::AssignToPad(1)),
            "assign_2" => Some(SampleSlicerActionId::AssignToPad(2)),
            "assign_3" => Some(SampleSlicerActionId::AssignToPad(3)),
            "assign_4" => Some(SampleSlicerActionId::AssignToPad(4)),
            "assign_5" => Some(SampleSlicerActionId::AssignToPad(5)),
            "assign_6" => Some(SampleSlicerActionId::AssignToPad(6)),
            "assign_7" => Some(SampleSlicerActionId::AssignToPad(7)),
            "assign_8" => Some(SampleSlicerActionId::AssignToPad(8)),
            "assign_9" => Some(SampleSlicerActionId::AssignToPad(9)),
            "assign_10" => Some(SampleSlicerActionId::AssignToPad(10)),
            "assign_11" => Some(SampleSlicerActionId::AssignToPad(11)),
            "assign_12" => Some(SampleSlicerActionId::AssignToPad(12)),
            _ => None,
        }
    }
}

define_action_enum! {
    /// EQ layer actions
    pub enum EqActionId {
        PrevBand => "prev_band",
        NextBand => "next_band",
        NextParam => "next_param",
        PrevParam => "prev_param",
        Increase => "increase",
        Decrease => "decrease",
        IncreaseBig => "increase_big",
        DecreaseBig => "decrease_big",
        IncreaseTiny => "increase_tiny",
        DecreaseTiny => "decrease_tiny",
        ToggleEq => "toggle_eq",
        ToggleBand => "toggle_band",
        CycleType => "cycle_type",
        CycleSlope => "cycle_slope",
    }
}

define_action_enum! {
    /// Tuner layer actions
    pub enum TunerActionId {
        PrevInstrument => "prev_instrument",
        NextInstrument => "next_instrument",
        PrevString => "prev_string",
        NextString => "next_string",
        TogglePlayback => "play_stop",
    }
}

define_action_enum! {
    /// Arpeggiator layer actions
    pub enum ArpeggiatorActionId {
        PrevParam => "prev_param",
        NextParam => "next_param",
        Toggle => "toggle",
        Increase => "increase",
        Decrease => "decrease",
        ClearChord => "clear_chord",
    }
}

define_action_enum! {
    /// Generative engine layer actions
    pub enum GenerativeActionId {
        PrevParam => "prev_param",
        NextParam => "next_param",
        PrevSection => "prev_section",
        NextSection => "next_section",
        PrevVoice => "prev_voice",
        NextVoice => "next_voice",
        Increase => "increase",
        Decrease => "decrease",
        Toggle => "toggle",
        ToggleEngine => "toggle_engine",
        ToggleCapture => "toggle_capture",
        CommitCapture => "commit_capture",
        ClearCapture => "clear_capture",
        AddVoice => "add_voice",
        RemoveVoice => "remove_voice",
        ToggleVoice => "toggle_voice",
        MuteVoice => "mute_voice",
        CycleAlgorithm => "cycle_algorithm",
        CycleTarget => "cycle_target",
        CycleRate => "cycle_rate",
        CycleRateReverse => "cycle_rate_reverse",
        CyclePitchMode => "cycle_pitch_mode",
    }
}

define_action_enum! {
    /// Groove layer actions
    pub enum GrooveActionId {
        PrevParam => "prev_param",
        NextParam => "next_param",
        Increase => "increase",
        Decrease => "decrease",
        IncreaseBig => "increase_big",
        DecreaseBig => "decrease_big",
        IncreaseTiny => "increase_tiny",
        DecreaseTiny => "decrease_tiny",
        CycleSwingGrid => "cycle_swing_grid",
        NextTimeSig => "cycle_time_sig",
        Reset => "reset",
    }
}

define_action_enum! {
    /// Track layer actions
    pub enum TrackActionId {
        LaneUp => "lane_up",
        LaneDown => "lane_down",
        CursorLeft => "cursor_left",
        CursorRight => "cursor_right",
        CursorHome => "cursor_home",
        CursorEnd => "cursor_end",
        NewClip => "new_clip",
        NewEmptyClip => "new_empty_clip",
        PlaceClip => "place_clip",
        EditClip => "edit_clip",
        Delete => "delete",
        DeleteClip => "delete_clip",
        Duplicate => "duplicate",
        ToggleMode => "toggle_mode",
        TogglePlayback => "play_stop",
        MoveLeft => "move_left",
        MoveRight => "move_right",
        ZoomIn => "zoom_in",
        ZoomOut => "zoom_out",
        SelectNextClipInstance => "select_next_placement",
        SelectPrevClipInstance => "select_prev_placement",
        SelectPrevClip => "select_prev_clip",
        SelectNextClip => "select_next_clip",
        LinkLayer => "link_layer",
        UnlinkLayer => "unlink_layer",
        LayerOctaveUp => "layer_octave_up",
        LayerOctaveDown => "layer_octave_down",
        RenameLayerGroup => "rename_layer_group",
        ImportPatch => "import_patch",
    }
}

define_action_enum! {
    /// VST parameters layer actions
    pub enum VstParamsActionId {
        Up => "up",
        Down => "down",
        Prev => "prev",
        Next => "next",
        Left => "left",
        Right => "right",
        AdjustDown => "adjust_down",
        AdjustUp => "adjust_up",
        CoarseLeft => "coarse_left",
        CoarseRight => "coarse_right",
        Search => "search",
        Reset => "reset",
        Automate => "automate",
        Discover => "discover",
        GotoTop => "goto_top",
        GotoBottom => "goto_bottom",
        Escape => "escape",
        Cancel => "cancel",
    }
}

define_action_enum! {
    /// Waveform layer actions
    pub enum WaveformActionId {
        CycleMode => "cycle_mode",
        LoadSample => "load_sample",
        Left => "left",
        Right => "right",
        JumpLeft => "jump_left",
        JumpRight => "jump_right",
        ToggleSelect => "toggle_select",
        DeleteSelection => "delete_selection",
        TrimSelection => "trim_selection",
        RenameClip => "rename_clip",
        CreatePitchedSampler => "create_pitched_sampler",
        CreateTimeStretchSampler => "create_timestretch_sampler",
        OpenInChopper => "open_in_chopper",
    }
}

define_action_enum! {
    /// MIDI settings layer actions
    pub enum MidiSettingsActionId {
        SwitchSection => "switch_section",
        Up => "up",
        Down => "down",
        Connect => "connect",
        Disconnect => "disconnect",
        RemoveMapping => "remove_mapping",
        TogglePassthrough => "toggle_passthrough",
        SetChannelAll => "set_channel_all",
        SetLiveInstrument => "set_live_instrument",
        ClearLiveInstrument => "clear_live_instrument",
        Learn => "learn",
        Confirm => "confirm",
        Cancel => "cancel",
    }
}

define_action_enum! {
    /// Mode layer actions (piano_mode, pad_mode, text_edit, command_palette, tap_tempo)
    pub enum ModeActionId {
        PianoEscape => "piano:escape",
        PianoOctaveDown => "piano:octave_down",
        PianoOctaveUp => "piano:octave_up",
        PianoSpace => "piano:space",
        PianoKey => "piano:key",
        PadEscape => "pad:escape",
        PadKey => "pad:key",
        TextConfirm => "text:confirm",
        TextCancel => "text:cancel",
        PaletteConfirm => "palette:confirm",
        PaletteCancel => "palette:cancel",
        TapTap => "tap:tap",
        TapPrev => "tap:prev",
        TapNext => "tap:next",
        TapConfirm => "tap:confirm",
        TapCancel => "tap:cancel",
    }
}

define_action_enum! {
    /// Confirm dialog layer actions
    pub enum ConfirmActionId {
        Confirm => "confirm",
        Cancel => "cancel",
        Left => "left",
        Right => "right",
        Toggle => "toggle",
    }
}

define_action_enum! {
    /// Project browser layer actions
    pub enum ProjectBrowserActionId {
        Select => "select",
        Close => "close",
        Up => "up",
        Down => "down",
        NewProject => "new_project",
        DeleteEntry => "delete_entry",
    }
}

define_action_enum! {
    /// Checkpoint list layer actions
    pub enum CheckpointListActionId {
        Select => "select",
        Close => "close",
        Up => "up",
        Down => "down",
        Delete => "delete",
    }
}

define_action_enum! {
    /// Tag view layer actions
    pub enum TagViewActionId {
        PrevParam => "prev_param",
        NextParam => "next_param",
        Increase => "increase",
        Decrease => "decrease",
        IncreaseBig => "increase_big",
        DecreaseBig => "decrease_big",
        IncreaseTiny => "increase_tiny",
        DecreaseTiny => "decrease_tiny",
        NextTag => "next_tag",
        PrevTag => "prev_tag",
        RemoveTarget => "remove_target",
        DeleteTag => "delete_tag",
        CreateTag => "create_tag",
        RenameTag => "rename_tag",
    }
}

/// Top-level action identifier wrapping all layer-specific action enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    Global(GlobalActionId),
    Mode(ModeActionId),
    TrackEdit(TrackEditActionId),
    Mixer(MixerActionId),
    PianoRoll(PianoRollActionId),
    Sequencer(SequencerActionId),
    Server(ServerActionId),
    Add(AddActionId),
    Home(HomeActionId),
    Help(HelpActionId),
    FrameEdit(FrameEditActionId),
    FileBrowser(FileBrowserActionId),
    SampleSlicer(SampleSlicerActionId),
    Eq(EqActionId),
    Arpeggiator(ArpeggiatorActionId),
    Generative(GenerativeActionId),
    Groove(GrooveActionId),
    Track(TrackActionId),
    VstParams(VstParamsActionId),
    Waveform(WaveformActionId),
    MidiSettings(MidiSettingsActionId),
    Confirm(ConfirmActionId),
    ProjectBrowser(ProjectBrowserActionId),
    CheckpointList(CheckpointListActionId),
    Tuner(TunerActionId),
    TagView(TagViewActionId),
}

impl ActionId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionId::Global(a) => a.as_str(),
            ActionId::Mode(a) => a.as_str(),
            ActionId::TrackEdit(a) => a.as_str(),
            ActionId::Mixer(a) => a.as_str(),
            ActionId::PianoRoll(a) => a.as_str(),
            ActionId::Sequencer(a) => a.as_str(),
            ActionId::Server(a) => a.as_str(),
            ActionId::Add(a) => a.as_str(),
            ActionId::Home(a) => a.as_str(),
            ActionId::Help(a) => a.as_str(),
            ActionId::FrameEdit(a) => a.as_str(),
            ActionId::FileBrowser(a) => a.as_str(),
            ActionId::SampleSlicer(a) => a.as_str(),
            ActionId::Eq(a) => a.as_str(),
            ActionId::Arpeggiator(a) => a.as_str(),
            ActionId::Generative(a) => a.as_str(),
            ActionId::Groove(a) => a.as_str(),
            ActionId::Track(a) => a.as_str(),
            ActionId::VstParams(a) => a.as_str(),
            ActionId::Waveform(a) => a.as_str(),
            ActionId::MidiSettings(a) => a.as_str(),
            ActionId::Confirm(a) => a.as_str(),
            ActionId::ProjectBrowser(a) => a.as_str(),
            ActionId::CheckpointList(a) => a.as_str(),
            ActionId::Tuner(a) => a.as_str(),
            ActionId::TagView(a) => a.as_str(),
        }
    }
}

/// Parse an action identifier from layer name and action string
pub fn parse_action_id(layer: &str, action: &str) -> Option<ActionId> {
    match layer {
        "global" => GlobalActionId::from_str(action).map(ActionId::Global),
        "instrument_edit" => TrackEditActionId::from_str(action).map(ActionId::TrackEdit),
        "mixer" => MixerActionId::from_str(action).map(ActionId::Mixer),
        "piano_roll" => PianoRollActionId::from_str(action).map(ActionId::PianoRoll),
        "sequencer" => SequencerActionId::from_str(action).map(ActionId::Sequencer),
        "server" => ServerActionId::from_str(action).map(ActionId::Server),
        "add" | "add_effect" | "tag_picker" => AddActionId::from_str(action).map(ActionId::Add),
        "home" => HomeActionId::from_str(action).map(ActionId::Home),
        "help" => HelpActionId::from_str(action).map(ActionId::Help),
        "frame_edit" => FrameEditActionId::from_str(action).map(ActionId::FrameEdit),
        "file_browser" => FileBrowserActionId::from_str(action).map(ActionId::FileBrowser),
        "sample_slicer" => SampleSlicerActionId::from_str(action).map(ActionId::SampleSlicer),
        "eq" => EqActionId::from_str(action).map(ActionId::Eq),
        "arpeggiator" => ArpeggiatorActionId::from_str(action).map(ActionId::Arpeggiator),
        "generative" => GenerativeActionId::from_str(action).map(ActionId::Generative),
        "groove" => GrooveActionId::from_str(action).map(ActionId::Groove),
        "tuner" => TunerActionId::from_str(action).map(ActionId::Tuner),
        "track" | "track_list" | "instrument" => {
            TrackActionId::from_str(action).map(ActionId::Track)
        }
        "vst_params" => VstParamsActionId::from_str(action).map(ActionId::VstParams),
        "waveform" => WaveformActionId::from_str(action).map(ActionId::Waveform),
        "midi_settings" => MidiSettingsActionId::from_str(action).map(ActionId::MidiSettings),
        "confirm" => ConfirmActionId::from_str(action).map(ActionId::Confirm),
        "project_browser" => ProjectBrowserActionId::from_str(action).map(ActionId::ProjectBrowser),
        "checkpoint_list" => CheckpointListActionId::from_str(action).map(ActionId::CheckpointList),
        "tag_view" => TagViewActionId::from_str(action).map(ActionId::TagView),
        "piano_mode" | "pad_mode" | "text_edit" | "command_palette" | "tap_tempo" => {
            ModeActionId::from_str(action).map(ActionId::Mode)
        }
        "command_line" | "quit_prompt" | "save_as" | "patch_export" => None, // No actions — handled via raw input
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_action_round_trip() {
        let actions = vec![
            GlobalActionId::Undo,
            GlobalActionId::Redo,
            GlobalActionId::Quit,
            GlobalActionId::Save,
            GlobalActionId::Load,
            GlobalActionId::SaveAs,
            GlobalActionId::MasterMute,
            GlobalActionId::ToggleRecordMaster,
            GlobalActionId::Copy,
            GlobalActionId::Cut,
            GlobalActionId::Paste,
            GlobalActionId::SelectAll,
            GlobalActionId::AddInstrument,
            GlobalActionId::DeleteInstrument,
            GlobalActionId::NavBack,
            GlobalActionId::NavForward,
            GlobalActionId::Help,
            GlobalActionId::CommandLine,
            GlobalActionId::CommandPalette,
            GlobalActionId::TogglePianoMode,
            GlobalActionId::OpenProjectBrowser,
            GlobalActionId::Escape,
            GlobalActionId::TogglePlayback,
            GlobalActionId::ClickTrackToggle,
            GlobalActionId::RequestPrivilege,
            GlobalActionId::SelectPrevInstrument,
            GlobalActionId::SelectNextInstrument,
            GlobalActionId::SelectTwoDigit,
            GlobalActionId::SwitchPane(PaneId::Home),
            GlobalActionId::SwitchPane(PaneId::InstrumentEdit),
            GlobalActionId::SwitchPane(PaneId::PianoRoll),
            GlobalActionId::SwitchPianoRollOrSequencer,
            GlobalActionId::SwitchPane(PaneId::Sequencer),
            GlobalActionId::SwitchPane(PaneId::Waveform),
            GlobalActionId::SwitchPane(PaneId::Track),
            GlobalActionId::SwitchPane(PaneId::Mixer),
            GlobalActionId::SwitchPane(PaneId::Server),
            GlobalActionId::SwitchPane(PaneId::Automation),
            GlobalActionId::SwitchPane(PaneId::Eq),
            GlobalActionId::SwitchPane(PaneId::MasterEq),
            GlobalActionId::SwitchPane(PaneId::SampleSlicer),
            GlobalActionId::SwitchPane(PaneId::FrameEdit),
            GlobalActionId::SwitchPane(PaneId::MidiSettings),
            GlobalActionId::SwitchPane(PaneId::Generative),
            GlobalActionId::SwitchPane(PaneId::Arpeggiator),
            GlobalActionId::SwitchPane(PaneId::VstParams),
            GlobalActionId::SwitchPane(PaneId::FileBrowser),
            GlobalActionId::SwitchPane(PaneId::Groove),
            GlobalActionId::SwitchPane(PaneId::Tuner),
            GlobalActionId::SwitchPane(PaneId::TagView),
            GlobalActionId::SelectInstrument(1),
            GlobalActionId::SelectInstrument(2),
            GlobalActionId::SelectInstrument(3),
            GlobalActionId::SelectInstrument(4),
            GlobalActionId::SelectInstrument(5),
            GlobalActionId::SelectInstrument(6),
            GlobalActionId::SelectInstrument(7),
            GlobalActionId::SelectInstrument(8),
            GlobalActionId::SelectInstrument(9),
            GlobalActionId::SelectInstrument(10),
        ];

        for action in actions {
            let s = action.as_str();
            let parsed = GlobalActionId::from_str(s);
            assert_eq!(Some(action), parsed, "Failed round-trip for {}", s);
        }
    }

    #[test]
    fn test_instrument_edit_round_trip() {
        let actions = vec![
            TrackEditActionId::Next,
            TrackEditActionId::Prev,
            TrackEditActionId::NextSection,
            TrackEditActionId::PrevSection,
            TrackEditActionId::Decrease,
            TrackEditActionId::Increase,
            TrackEditActionId::IncreaseBig,
            TrackEditActionId::DecreaseBig,
            TrackEditActionId::IncreaseTiny,
            TrackEditActionId::DecreaseTiny,
            TrackEditActionId::IncreaseMusical,
            TrackEditActionId::DecreaseMusical,
            TrackEditActionId::EnterEdit,
            TrackEditActionId::ToggleFilter,
            TrackEditActionId::NextFilterType,
            TrackEditActionId::AddEffect,
            TrackEditActionId::RemoveEffect,
            TrackEditActionId::TogglePoly,
            TrackEditActionId::ZeroParam,
            TrackEditActionId::ZeroSection,
            TrackEditActionId::ResetParam,
            TrackEditActionId::ToggleEq,
            TrackEditActionId::ToggleLfo,
            TrackEditActionId::CycleLfoShape,
            TrackEditActionId::CycleLfoTarget,
            TrackEditActionId::ToggleActive,
            TrackEditActionId::LoadSample,
            TrackEditActionId::VstParams,
            TrackEditActionId::MoveStageUp,
            TrackEditActionId::MoveStageDown,
            TrackEditActionId::ToggleEffectBypass,
            TrackEditActionId::Done,
            TrackEditActionId::TagParam,
            TrackEditActionId::OpenEq,
            TrackEditActionId::ExportPatch,
        ];

        for action in actions {
            let s = action.as_str();
            let parsed = TrackEditActionId::from_str(s);
            assert_eq!(Some(action), parsed, "Failed round-trip for {}", s);
        }
    }

    #[test]
    fn test_mixer_round_trip() {
        let actions = vec![
            MixerActionId::Prev,
            MixerActionId::Next,
            MixerActionId::First,
            MixerActionId::Last,
            MixerActionId::LevelUp,
            MixerActionId::LevelDown,
            MixerActionId::LevelUpBig,
            MixerActionId::LevelDownBig,
            MixerActionId::Mute,
            MixerActionId::Solo,
            MixerActionId::Output,
            MixerActionId::OutputRev,
            MixerActionId::Section,
            MixerActionId::SectionPrev,
            MixerActionId::SendNext,
            MixerActionId::SendPrev,
            MixerActionId::SendToggle,
            MixerActionId::Escape,
            MixerActionId::EnterDetail,
            MixerActionId::AddEffect,
            MixerActionId::RemoveEffect,
            MixerActionId::ToggleEffect,
            MixerActionId::ToggleFilter,
            MixerActionId::NextFilterType,
            MixerActionId::FineLeft,
            MixerActionId::FineRight,
            MixerActionId::MoveUp,
            MixerActionId::MoveDown,
            MixerActionId::PanLeft,
            MixerActionId::PanRight,
            MixerActionId::ClearSend,
            MixerActionId::Increase,
            MixerActionId::Decrease,
            MixerActionId::OpenMasterEq,
        ];

        for action in actions {
            let s = action.as_str();
            let parsed = MixerActionId::from_str(s);
            assert_eq!(Some(action), parsed, "Failed round-trip for {}", s);
        }
    }

    #[test]
    fn test_sample_chopper_round_trip() {
        let actions = vec![
            SampleSlicerActionId::MoveLeft,
            SampleSlicerActionId::MoveRight,
            SampleSlicerActionId::NextSlice,
            SampleSlicerActionId::PrevSlice,
            SampleSlicerActionId::Chop,
            SampleSlicerActionId::Delete,
            SampleSlicerActionId::AutoSlice,
            SampleSlicerActionId::LoadSample,
            SampleSlicerActionId::Preview,
            SampleSlicerActionId::Commit,
            SampleSlicerActionId::Back,
            SampleSlicerActionId::NudgeStart,
            SampleSlicerActionId::NudgeEnd,
            SampleSlicerActionId::AssignToPad(1),
            SampleSlicerActionId::AssignToPad(2),
            SampleSlicerActionId::AssignToPad(3),
            SampleSlicerActionId::AssignToPad(4),
            SampleSlicerActionId::AssignToPad(5),
            SampleSlicerActionId::AssignToPad(6),
            SampleSlicerActionId::AssignToPad(7),
            SampleSlicerActionId::AssignToPad(8),
            SampleSlicerActionId::AssignToPad(9),
            SampleSlicerActionId::AssignToPad(10),
            SampleSlicerActionId::AssignToPad(11),
            SampleSlicerActionId::AssignToPad(12),
        ];

        for action in actions {
            let s = action.as_str();
            let parsed = SampleSlicerActionId::from_str(s);
            assert_eq!(Some(action), parsed, "Failed round-trip for {}", s);
        }
    }

    #[test]
    fn test_parse_action_id() {
        assert_eq!(
            parse_action_id("global", "undo"),
            Some(ActionId::Global(GlobalActionId::Undo))
        );

        assert_eq!(
            parse_action_id("mixer", "mute"),
            Some(ActionId::Mixer(MixerActionId::Mute))
        );

        assert_eq!(
            parse_action_id("piano_roll", "toggle_note"),
            Some(ActionId::PianoRoll(PianoRollActionId::ToggleNote))
        );

        assert_eq!(
            parse_action_id("add", "confirm"),
            Some(ActionId::Add(AddActionId::Confirm))
        );

        assert_eq!(
            parse_action_id("add_effect", "confirm"),
            Some(ActionId::Add(AddActionId::Confirm))
        );

        assert_eq!(
            parse_action_id("piano_mode", "piano:escape"),
            Some(ActionId::Mode(ModeActionId::PianoEscape))
        );

        assert_eq!(parse_action_id("save_as", "anything"), None);

        assert_eq!(parse_action_id("unknown_layer", "action"), None);

        assert_eq!(parse_action_id("global", "unknown_action"), None);
    }

    #[test]
    fn test_action_id_as_str() {
        let action = ActionId::Global(GlobalActionId::Undo);
        assert_eq!(action.as_str(), "undo");

        let action = ActionId::Mixer(MixerActionId::Mute);
        assert_eq!(action.as_str(), "mute");

        let action = ActionId::Mode(ModeActionId::PianoEscape);
        assert_eq!(action.as_str(), "piano:escape");
    }
}
