mod audio_sync;
mod feedback;
mod input;
mod render;

use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use crate::action::{AudioEffect, IoFeedback};
use crate::audio::AudioHandle;
use crate::buffer::mixer::MixerBuffer;
use crate::buffer::scratch::ScratchBuffer;
use crate::buffer::{BufferId, BufferKind, BufferRegistry};
use crate::chrome::GlobalChrome;
use crate::config;
use crate::dispatch::LocalDispatcher;
use crate::input::InputProcessor;
use crate::midi;
use crate::setup;
use crate::state::AppState;
use crate::ui::keybindings;
use crate::ui::layer::LayerStack;
use crate::ui::ratatui_impl::RatatuiBackend;
use crate::ui::status_bar::StatusLevel;
use crate::window::WindowTree;

fn autosave_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("imbolc")
        .join(".imbolc-ui2.autosave")
}

/// Compute the window area: full terminal minus header (row 0) and status bar (last row).
pub fn window_area(full: ratatui::layout::Rect) -> ratatui::layout::Rect {
    if full.height < 3 {
        return full;
    }
    ratatui::layout::Rect::new(
        full.x,
        full.y + 1,
        full.width,
        full.height.saturating_sub(2),
    )
}

/// Top-level runtime that owns all application state.
pub struct AppRuntime {
    pub(crate) dispatcher: LocalDispatcher,
    pub(crate) audio: AudioHandle,
    pub(crate) tree: WindowTree,
    pub(crate) registry: BufferRegistry,
    pub(crate) layer_stack: LayerStack,
    pub(crate) chrome: GlobalChrome,
    pub(crate) input_processor: InputProcessor,
    pub(crate) midi_input: midi::MidiInputManager,
    pub(crate) io_rx: Receiver<IoFeedback>,

    // Per-frame state
    pub(crate) pending_audio_effects: Vec<AudioEffect>,
    pub(crate) needs_full_sync: bool,
    pub(crate) render_needed: bool,
    pub(crate) last_render_time: Instant,
    pub(crate) last_area: ratatui::layout::Rect,
    pub(crate) autosave_enabled: bool,
    pub(crate) autosave_interval: Duration,
    pub(crate) autosave_path: PathBuf,
    pub(crate) autosave_id: u64,
    pub(crate) autosave_in_progress: bool,
    pub(crate) last_autosave_at: Instant,
}

impl AppRuntime {
    pub fn new() -> Self {
        let (io_tx, io_rx) = std::sync::mpsc::channel::<IoFeedback>();
        let config = config::Config::load();
        let autosave_enabled = config.autosave_enabled();
        let autosave_interval_minutes = config.autosave_interval_minutes();
        let autosave_interval = Duration::from_secs(autosave_interval_minutes.saturating_mul(60));
        let autosave_path = autosave_path();
        let mut state = AppState::new_with_defaults(config.defaults());
        state.keyboard_layout = config.keyboard_layout();
        state.session.theme = config.theme();

        // Load keybindings
        let (layers, _keymaps) = keybindings::load_keybindings();
        let mut layer_stack = LayerStack::new(layers);
        layer_stack.push("global");

        // Buffer registry with initial scratch buffer
        let mut registry = BufferRegistry::new();
        registry.register(Box::new(ScratchBuffer::new()));
        registry.register(Box::new(MixerBuffer::new()));

        // Window tree with scratch
        let scratch_id = BufferId::global(BufferKind::Scratch);
        let tree = WindowTree::new(scratch_id);
        layer_stack.set_buffer_layer("scratch");

        // Audio
        let mut audio = AudioHandle::new();
        audio.sync_state(&state);

        let dispatcher = LocalDispatcher::new(state, io_tx);
        let mut chrome = GlobalChrome::new();
        chrome.set_autosave_config(autosave_enabled, autosave_interval_minutes);

        // MIDI
        let mut midi_input = midi::MidiInputManager::new();
        midi_input.refresh_ports();
        if !midi_input.list_ports().is_empty() {
            let _ = midi_input.connect(0);
        }

        // Auto-start SuperCollider
        let startup_events = setup::auto_start_sc(&mut audio);
        for event in &startup_events {
            chrome.status_bar.push(&event.message, StatusLevel::Info);
        }

        Self {
            dispatcher,
            audio,
            tree,
            registry,
            layer_stack,
            chrome,
            input_processor: InputProcessor::new(),
            midi_input,
            io_rx,
            pending_audio_effects: Vec::new(),
            needs_full_sync: false,
            render_needed: true,
            last_render_time: Instant::now(),
            last_area: ratatui::layout::Rect::new(0, 0, 80, 24),
            autosave_enabled,
            autosave_interval,
            autosave_path,
            autosave_id: 0,
            autosave_in_progress: false,
            last_autosave_at: Instant::now(),
        }
    }

    /// Main event loop.
    pub fn run(&mut self, backend: &mut RatatuiBackend) -> std::io::Result<()> {
        loop {
            // Sync buffer layer to focused window
            let buf_id = self.tree.focused_buffer();
            if let Some(buffer) = self.registry.get(&buf_id) {
                self.layer_stack.set_buffer_layer(buffer.layer_name());
            }

            let frame_budget = Duration::from_millis(16);
            let elapsed = self.last_render_time.elapsed();
            let poll_timeout = if self.audio.is_running() || self.render_needed {
                frame_budget
                    .saturating_sub(elapsed)
                    .max(Duration::from_millis(1))
            } else {
                Duration::from_millis(50)
            };

            if self.process_events(backend, poll_timeout)? {
                break;
            }

            self.process_tick();
            self.apply_pending_effects();
            self.drain_io_feedback();
            self.maybe_autosave();
            self.drain_audio_feedback();
            self.drain_midi_events();
            self.maybe_render(backend)?;
        }
        Ok(())
    }

    /// Periodic autosave.
    pub(crate) fn maybe_autosave(&mut self) {
        if !self.autosave_enabled || self.autosave_in_progress {
            return;
        }
        if !self.dispatcher.state().project.dirty {
            return;
        }
        if self.last_autosave_at.elapsed() < self.autosave_interval {
            return;
        }

        self.last_autosave_at = Instant::now();
        self.autosave_in_progress = true;
        self.autosave_id = self.autosave_id.wrapping_add(1);

        let id = self.autosave_id;
        let path = self.autosave_path.clone();
        let session = self.dispatcher.state().session.clone();
        let instruments = self.dispatcher.state().tracks.clone();
        let tx = self.dispatcher.io_tx().clone();

        std::thread::spawn(move || {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let result = crate::state::persistence::save_project(&path, &session, &instruments)
                .map_err(|e| e.to_string());
            let _ = tx.send(IoFeedback::AutosaveComplete { id, path, result });
        });
    }
}

/// Public entry point.
pub fn run(backend: &mut RatatuiBackend) -> std::io::Result<()> {
    let mut runtime = AppRuntime::new();
    runtime.run(backend)
}
