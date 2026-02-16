# Imbolc Deep Review: 11 Points

This document captures the 11 key review points discussed: the original 10 codebase findings plus the generative-networking architecture decision.

1. **[P3] Net outbox unit tests depend on real loopback sockets**  
   `imbolc-net/src/server.rs:2041` tests queue-drop logic using real TCP loopback binds, which makes the suite environment-sensitive in sandboxed/locked-down CI.

2. **[P0] Net feature build breaks on discover flag variable**  
   `imbolc-ui/src/main.rs:65` used `_discover_mode` while later referencing `discover_mode`, causing compile failure in net builds.

3. **[P0] Client ownership IDs use wrong type for `connect()`**  
   `imbolc-ui/src/network.rs:236` passed `Vec<u32>` into `RemoteDispatcher::connect`, which expects `Vec<InstrumentId>`.

4. **[P0] Network action mapping drifted from `Action` enum**  
   `imbolc-ui/src/network.rs:492` omitted `Action::Generative`, causing non-exhaustive match and protocol drift.

5. **[P1] Alt+named key bindings are silently unsupported**  
   `imbolc-ui/src/ui/keybindings.rs:64` parses `Alt+...` as a single char variant, so named keys like `Alt+Right`/`Alt+Left` are dropped.

6. **[P1] Back/forward history algorithm is brittle and likely inverted**  
   `imbolc-ui/src/global_actions.rs:635` mixes index deltas (`-1`, `+1`, `±2`) around `at_front`, increasing cognitive load and regression risk.

7. **[P1] REPL parser ignores trailing tokens and lacks quoted args**  
   `imbolc-ui/src/repl/macro_def.rs:27` tokenization by `split_whitespace` plus permissive arity means extra tokens are accepted and quoted/spaced args are unsupported.

8. **[P2] REPL registry has stale contract for piano-roll note editing**  
   `imbolc-ui/src/repl/registry.rs:88` references a handwritten `ToggleNote` handler that does not exist, creating docs/behavior drift.

9. **[P2] Pane identity duplicated across independent enums**  
   `imbolc-ui/src/ui/action_id.rs:6` duplicates pane identity already modeled by `imbolc_types::PaneId`, increasing mapper/drift surface.

10. **[P2] Hot-path action routing clones large enum payloads**  
    `imbolc-types/src/action.rs:1140` `Action::route()` clones payloads for all domain variants, including heavy variants like generative actions.

11. **[Architecture] Generative actions should be network-synced and server-authoritative**  
    Given the “only DAW” principle (single transport, single global tuning/BPM, single rendering engine), `GenerativeAction` belongs in the shared network command model and should be validated by server authority policy (privilege/ownership model), then broadcast through normal state patch/full sync paths.
