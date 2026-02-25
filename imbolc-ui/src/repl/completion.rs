use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::Helper;

use super::registry;

pub struct ReplHelper;

impl Helper for ReplHelper {}
impl Validator for ReplHelper {}
impl Highlighter for ReplHelper {}

impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if line.is_empty() {
            return None;
        }

        let completions = complete_input(line);
        if completions.len() == 1 {
            let completion = &completions[0];
            if completion.len() > line.len() && completion.starts_with(line) {
                return Some(completion[line.len()..].to_string());
            }
        }
        None
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let input = &line[..pos];
        let completions = complete_input(input);

        // Find replacement start position
        let start = input.rfind(' ').map(|i| i + 1).unwrap_or(0);

        let pairs: Vec<Pair> = completions
            .into_iter()
            .map(|full| {
                let display = full.clone();
                let replacement = if full.len() > start {
                    full[start..].to_string()
                } else {
                    full
                };
                Pair {
                    display,
                    replacement,
                }
            })
            .collect();

        Ok((start, pairs))
    }
}

pub fn complete_input(input: &str) -> Vec<String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let trailing_space = input.ends_with(' ');

    // Meta commands
    let meta_commands = &[
        "help", "show", "set", "undo", "redo", "save", "load", "quit", "exit", "status", "bookmark",
    ];

    if parts.is_empty() || (parts.len() == 1 && !trailing_space) {
        // Complete group names + meta commands
        let prefix = parts.first().copied().unwrap_or("");
        let mut completions: Vec<String> = registry::group_names()
            .iter()
            .filter(|g| g.starts_with(prefix))
            .map(|g| g.to_string())
            .collect();

        completions.extend(
            meta_commands
                .iter()
                .filter(|c| c.starts_with(prefix))
                .map(|c| c.to_string()),
        );

        completions.sort();
        completions
    } else if parts.len() == 1 && trailing_space {
        // Group typed, complete subcommands
        let group = parts[0];
        match group {
            "show" => vec![
                "tracks",
                "track",
                "transport",
                "mixer",
                "notes",
                "effects",
                "buses",
                "arrangement",
                "automation",
                "generative",
                "session",
                "sequencer",
                "server",
                "midi",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            "set" => vec!["bpm", "key", "scale", "tuning", "ji-flavor"]
                .into_iter()
                .map(String::from)
                .collect(),
            "bookmark" => vec!["list", "set", "delete"]
                .into_iter()
                .map(String::from)
                .collect(),
            "help" => {
                let mut groups: Vec<String> = registry::group_names()
                    .iter()
                    .map(|g| g.to_string())
                    .collect();
                groups.sort();
                groups
            }
            _ => registry::complete_command(&format!("{} ", group)),
        }
    } else if parts.len() == 2 && !trailing_space {
        // Typing second word
        let group = parts[0];
        let prefix = parts[1];
        match group {
            "show" => {
                let subs = [
                    "tracks",
                    "track",
                    "transport",
                    "mixer",
                    "notes",
                    "effects",
                    "buses",
                    "arrangement",
                    "automation",
                    "generative",
                    "session",
                    "sequencer",
                    "server",
                    "midi",
                ];
                subs.iter()
                    .filter(|s| s.starts_with(prefix))
                    .map(|s| s.to_string())
                    .collect()
            }
            "set" => {
                let subs = ["bpm", "key", "scale", "tuning", "ji-flavor"];
                subs.iter()
                    .filter(|s| s.starts_with(prefix))
                    .map(|s| s.to_string())
                    .collect()
            }
            "bookmark" => {
                let subs = ["list", "set", "delete"];
                subs.iter()
                    .filter(|s| s.starts_with(prefix))
                    .map(|s| s.to_string())
                    .collect()
            }
            "help" => registry::group_names()
                .iter()
                .filter(|g| g.starts_with(prefix))
                .map(|g| g.to_string())
                .collect(),
            _ => {
                let full_prefix = format!("{} {}", group, prefix);
                registry::complete_command(&full_prefix)
                    .into_iter()
                    .map(|s| {
                        // Extract just the subcommand part
                        s.split_whitespace().nth(1).unwrap_or(&s).to_string()
                    })
                    .collect()
            }
        }
    } else if parts.len() >= 2 && trailing_space {
        // After command, hint at arg types from source types etc.
        let group = parts[0];
        let cmd = parts[1];
        if group == "track" && cmd == "add" && parts.len() == 2 {
            return imbolc_types::SourceType::all_short_names()
                .iter()
                .map(|s| s.to_string())
                .collect();
        }
        Vec::new()
    } else {
        Vec::new()
    }
}
