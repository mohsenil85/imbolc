/// Command metadata returned by all_commands().
pub struct CommandInfo {
    pub group: &'static str,
    pub name: &'static str,
    pub args: &'static str,
    pub description: &'static str,
}

/// Declarative macro for generating the REPL command registry.
///
/// Generates:
/// - `parse_action(input) -> Result<DomainAction, String>`: parses a command string into a DomainAction
/// - `all_commands() -> Vec<CommandInfo>`: returns metadata for all registered commands
/// - `complete_command(partial) -> Vec<String>`: prefix-matching for tab completion
macro_rules! repl_actions {
    (
        $(
            group $group:literal => $wrapper:path, $enum:ident {
                $( $cmd:literal => $variant:ident
                    $( ( $($arg_name:ident : $arg_type:ty),+ ) )?
                    ; $desc:literal
                )*
            }
        )*
    ) => {
        #[allow(unused_assignments)]
        pub fn parse_action(input: &str) -> Result<imbolc_types::DomainAction, String> {
            let parts = $crate::repl::parse::tokenize(input)?;
            if parts.is_empty() {
                return Err("empty command".to_string());
            }

            let group = parts[0].as_str();
            let subcommand = parts.get(1).map(|s| s.as_str()).unwrap_or("");

            match group {
                $(
                    $group => {
                        match subcommand {
                            $(
                                $cmd => {
                                    #[allow(unused_variables, unused_mut, unused_assignments)]
                                    let mut arg_idx = 2usize;
                                    $(
                                        $(
                                            let $arg_name = parts.get(arg_idx)
                                                .ok_or_else(|| format!("missing argument '{}' ({})", stringify!($arg_name), <$arg_type as $crate::repl::parse::ReplParseable>::type_hint()))
                                                .and_then(|s| <$arg_type as $crate::repl::parse::ReplParseable>::parse_repl(s.as_str()))?;
                                            arg_idx += 1;
                                        )+
                                    )?
                                    if parts.len() > arg_idx {
                                        let trailing: Vec<&str> = parts[arg_idx..].iter().map(|s| s.as_str()).collect();
                                        return Err(format!("unexpected trailing: '{}'", trailing.join(" ")));
                                    }
                                    Ok($wrapper(
                                        imbolc_types::$enum::$variant $(( $($arg_name),+ ))?
                                    ))
                                }
                            )*
                            "" => Err(format!("missing subcommand for '{}'. Try: help {}", $group, $group)),
                            other => Err(format!("unknown {} command: '{}'. Try: help {}", $group, other, $group)),
                        }
                    }
                )*
                _ => Err(format!("unknown command group: '{}'. Try: help", group)),
            }
        }

        pub fn all_commands() -> Vec<$crate::repl::macro_def::CommandInfo> {
            vec![
            $(
                $(
                    $crate::repl::macro_def::CommandInfo {
                        group: $group,
                        name: $cmd,
                        args: repl_actions!(@args_str $( $($arg_name : $arg_type),+ )? ),
                        description: $desc,
                    },
                )*
            )*
            ]
        }

        pub fn complete_command(partial: &str) -> Vec<String> {
            let parts: Vec<&str> = partial.split_whitespace().collect();

            // If empty or typing first word — complete group names
            if parts.len() <= 1 {
                let prefix = parts.first().copied().unwrap_or("");
                let groups: &[&str] = &[ $( $group ),* ];
                return groups.iter()
                    .filter(|g| g.starts_with(prefix))
                    .map(|g| g.to_string())
                    .collect();
            }

            // Typing second word — complete subcommands for matched group
            let group = parts[0];
            let sub_prefix = parts.get(1).copied().unwrap_or("");

            match group {
                $(
                    $group => {
                        let subs: &[&str] = &[ $( $cmd ),* ];
                        subs.iter()
                            .filter(|s| s.starts_with(sub_prefix))
                            .map(|s| format!("{} {}", $group, s))
                            .collect()
                    }
                )*
                _ => Vec::new(),
            }
        }

        pub fn group_names() -> &'static [&'static str] {
            &[ $( $group ),* ]
        }
    };

    // Helper to generate args display string
    (@args_str $( $arg_name:ident : $arg_type:ty ),+ ) => {
        concat!( $( " ", stringify!($arg_name) ),+ )
    };
    (@args_str ) => {
        ""
    };
}

pub(crate) use repl_actions;
