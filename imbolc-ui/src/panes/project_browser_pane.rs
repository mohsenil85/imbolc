use std::any::Any;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::recent_projects::RecentProjects;
use crate::state::AppState;
use crate::ui::action_id::{ActionId, ProjectBrowserActionId};
use crate::ui::layout_helpers::center_rect;
use crate::ui::{
    Action, InputEvent, Keymap, NavAction, Palette, Pane, PaneIdStr, Rect, RenderBuf,
    SessionAction, Style,
};

pub struct ProjectBrowserPane {
    keymap: Keymap,
    entries: Vec<ProjectEntry>,
    selected: usize,
    autosave_path: Option<PathBuf>,
    has_autosave: bool,
}

struct ProjectEntry {
    name: String,
    path: PathBuf,
    last_opened: SystemTime,
}

impl ProjectBrowserPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            entries: Vec::new(),
            selected: 0,
            autosave_path: None,
            has_autosave: false,
        }
    }

    pub fn set_autosave_path(&mut self, path: PathBuf) {
        self.autosave_path = Some(path);
    }

    /// Number of fixed entries before recent projects (New Project + optional autosave)
    fn fixed_entry_count(&self) -> usize {
        1 + if self.has_autosave { 1 } else { 0 }
    }

    fn total_entries(&self) -> usize {
        self.fixed_entry_count() + self.entries.len()
    }

    /// Refresh the project list from disk
    pub fn refresh(&mut self) {
        let recent = RecentProjects::load();
        self.entries = recent
            .entries
            .into_iter()
            .map(|e| ProjectEntry {
                name: e.name,
                path: e.path,
                last_opened: e.last_opened,
            })
            .collect();
        self.has_autosave = self.autosave_path.as_ref().is_some_and(|p| p.exists());
        self.selected = 0;
    }

    fn format_time_ago(time: SystemTime) -> String {
        let now = SystemTime::now();
        let elapsed = now.duration_since(time).unwrap_or_default();
        let secs = elapsed.as_secs();
        if secs < 60 {
            return "just now".to_string();
        }
        if secs < 3600 {
            return format!("{} min ago", secs / 60);
        }
        if secs < 86400 {
            return format!("{} hours ago", secs / 3600);
        }
        if secs < 604800 {
            return format!("{} days ago", secs / 86400);
        }
        // Fallback to date
        let since_epoch = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let days = since_epoch / 86400;
        let years = 1970 + days / 365;
        format!("{}", years)
    }

    /// Map selected index to the recent project entry index, if applicable
    fn selected_project_index(&self) -> Option<usize> {
        let fixed = self.fixed_entry_count();
        if self.selected >= fixed {
            Some(self.selected - fixed)
        } else {
            None
        }
    }

    fn delete_selected_entry(&mut self) {
        if let Some(idx) = self.selected_project_index() {
            if let Some(entry) = self.entries.get(idx) {
                let path = entry.path.clone();
                let mut recent = RecentProjects::load();
                recent.remove(&path);
                recent.save();
                self.refresh();
            }
        }
    }
}

impl Pane for ProjectBrowserPane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("project_browser")
    }

    fn on_enter(&mut self, _state: &AppState) {
        self.refresh();
    }

    fn handle_action(
        &mut self,
        action: ActionId,
        _event: &InputEvent,
        _state: &AppState,
    ) -> Action {
        match action {
            ActionId::ProjectBrowser(ProjectBrowserActionId::Close) => {
                Action::Nav(NavAction::PopPane)
            }
            ActionId::ProjectBrowser(ProjectBrowserActionId::Up) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::None
            }
            ActionId::ProjectBrowser(ProjectBrowserActionId::Down) => {
                if self.selected + 1 < self.total_entries() {
                    self.selected += 1;
                }
                Action::None
            }
            ActionId::ProjectBrowser(ProjectBrowserActionId::Select) => {
                if self.selected == 0 {
                    return Action::Session(SessionAction::NewProject);
                }
                if self.has_autosave && self.selected == 1 {
                    if let Some(path) = self.autosave_path.clone() {
                        return Action::Session(SessionAction::LoadFrom(path));
                    }
                }
                if let Some(idx) = self.selected_project_index() {
                    if let Some(entry) = self.entries.get(idx) {
                        return Action::Session(SessionAction::LoadFrom(entry.path.clone()));
                    }
                }
                Action::None
            }
            ActionId::ProjectBrowser(ProjectBrowserActionId::NewProject) => {
                Action::Session(SessionAction::NewProject)
            }
            ActionId::ProjectBrowser(ProjectBrowserActionId::DeleteEntry) => {
                self.delete_selected_entry();
                Action::None
            }
            _ => Action::None,
        }
    }

    fn handle_raw_input(&mut self, event: &InputEvent, _state: &AppState) -> Action {
        match event.key {
            crate::ui::KeyCode::Char('n') | crate::ui::KeyCode::Char('N') => {
                Action::Session(SessionAction::NewProject)
            }
            crate::ui::KeyCode::Char('i') | crate::ui::KeyCode::Char('I') => Action::Session(
                SessionAction::RequestFileBrowser(crate::ui::FileSelectAction::ImportProject),
            ),
            crate::ui::KeyCode::Char('d') | crate::ui::KeyCode::Char('D') => {
                self.delete_selected_entry();
                Action::None
            }
            _ => Action::None,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let width = 56_u16.min(area.width.saturating_sub(4));
        let height = (self.total_entries() as u16 + 8)
            .min(area.height.saturating_sub(4))
            .max(10);
        let rect = center_rect(area, width, height);

        let border_style = Style::new().fg(p.accent);
        let inner = buf.draw_block(rect, " Projects ", border_style, border_style);
        let content_width = inner.width.saturating_sub(2);

        // Virtual list: "New Project", optional autosave, then recent projects
        let max_visible = (inner.height.saturating_sub(3)) as usize; // room for header gap + footer
        let total = self.total_entries();
        let scroll = if self.selected >= max_visible {
            self.selected - max_visible + 1
        } else {
            0
        };

        for (row, vi) in (scroll..total).enumerate() {
            if row >= max_visible {
                break;
            }
            let y = inner.y + 1 + row as u16;
            if y >= inner.y + inner.height.saturating_sub(2) {
                break;
            }
            let is_selected = vi == self.selected;
            let x = inner.x;

            match vi {
                0 => {
                    // "New Project" entry
                    let style = if is_selected {
                        Style::new().fg(p.bg).bg(p.accent).bold()
                    } else {
                        Style::new().fg(p.fg)
                    };
                    if is_selected {
                        for cx in (x + 1)..(x + 1 + content_width) {
                            buf.set_cell(cx, y, ' ', style);
                        }
                    }
                    let prefix = if is_selected { " > " } else { "   " };
                    let line_area = Rect::new(x, y, content_width + 2, 1);
                    buf.draw_line(line_area, &[(prefix, style), ("New Project", style)]);
                }
                1 if self.has_autosave => {
                    // Autosave recovery entry
                    let style = if is_selected {
                        Style::new().fg(p.bg).bg(p.accent).bold()
                    } else {
                        Style::new().fg(p.warning)
                    };
                    if is_selected {
                        for cx in (x + 1)..(x + 1 + content_width) {
                            buf.set_cell(cx, y, ' ', style);
                        }
                    }
                    let prefix = if is_selected { " > " } else { "   " };
                    let indicator = if is_selected { "" } else { "! " };
                    let line_area = Rect::new(x, y, content_width + 2, 1);
                    buf.draw_line(
                        line_area,
                        &[
                            (prefix, style),
                            (indicator, style),
                            ("Recover last session", style),
                        ],
                    );
                }
                _ => {
                    // Recent project entry
                    let idx = vi - self.fixed_entry_count();
                    if let Some(entry) = self.entries.get(idx) {
                        let time_str = Self::format_time_ago(entry.last_opened);
                        let name_max =
                            content_width.saturating_sub(time_str.len() as u16 + 6) as usize;
                        let display_name: String = entry.name.chars().take(name_max).collect();

                        let (name_style, time_style) = if is_selected {
                            (
                                Style::new().fg(p.bg).bg(p.accent).bold(),
                                Style::new().fg(p.bg).bg(p.accent),
                            )
                        } else {
                            (Style::new().fg(p.fg), Style::new().fg(p.muted))
                        };

                        if is_selected {
                            for cx in (x + 1)..(x + 1 + content_width) {
                                buf.set_cell(cx, y, ' ', Style::new().fg(p.bg).bg(p.accent).bold());
                            }
                        }

                        let prefix = if is_selected { " > " } else { "   " };
                        let padding_len = name_max.saturating_sub(display_name.len());
                        let padding: String = " ".repeat(padding_len);
                        let time_col = format!("  {}", time_str);

                        let line_area = Rect::new(x, y, content_width + 2, 1);
                        buf.draw_line(
                            line_area,
                            &[
                                (prefix, name_style),
                                (&display_name, name_style),
                                (&padding, name_style),
                                (&time_col, time_style),
                            ],
                        );
                    }
                }
            }
        }

        // "No recent projects" when list is empty
        if self.entries.is_empty() {
            let y = inner.y + 1 + self.fixed_entry_count() as u16;
            if y < inner.y + inner.height.saturating_sub(2) {
                let area = Rect::new(inner.x + 1, y, content_width, 1);
                buf.draw_line(area, &[("  No recent projects", Style::new().fg(p.muted))]);
            }
        }

        // Footer
        let footer_y = rect.y + rect.height.saturating_sub(2);
        if footer_y < area.y + area.height {
            let hi = Style::new().fg(p.accent).bold();
            let lo = Style::new().fg(p.dim);
            let footer_area = Rect::new(inner.x + 1, footer_y, content_width, 1);
            buf.draw_line(
                footer_area,
                &[
                    ("[N]", hi),
                    ("ew  ", lo),
                    ("[I]", hi),
                    ("mport  ", lo),
                    ("[Enter]", hi),
                    (" Open  ", lo),
                    ("[D]", hi),
                    ("elete  ", lo),
                    ("[Esc]", hi),
                    (" Close", lo),
                ],
            );
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
