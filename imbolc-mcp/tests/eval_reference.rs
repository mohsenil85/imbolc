//! Layer 1: Reference tool evals.
//!
//! Pure function calls against static metadata catalogs.
//! No DAW, no IPC — tests that the catalog data is structured correctly.

#[allow(dead_code)]
mod common;

use imbolc_mcp::tools::reference;

// ============================================================================
// list_instruments
// ============================================================================

#[test]
fn list_instruments_has_categories() {
    let out = reference::list_instruments();
    common::assert_contains_all(&out, &["Oscillators", "Physical Models", "Drums"]);
}

#[test]
fn list_instruments_has_saw() {
    let out = reference::list_instruments();
    common::assert_contains(&out, "**Saw** (`saw`)");
}

#[test]
fn list_instruments_has_tags_and_use_cases() {
    let out = reference::list_instruments();
    common::assert_contains(&out, "Tags:");
    common::assert_contains(&out, "Use:");
}

// ============================================================================
// list_effects
// ============================================================================

#[test]
fn list_effects_has_categories() {
    let out = reference::list_effects();
    common::assert_contains_all(&out, &["Time", "Dynamics", "Modulation"]);
}

#[test]
fn list_effects_has_reverb() {
    let out = reference::list_effects();
    common::assert_contains(&out, "**Reverb**");
}

// ============================================================================
// list_filters
// ============================================================================

#[test]
fn list_filters_has_types() {
    let out = reference::list_filters();
    common::assert_contains_all(&out, &["Low-Pass", "High-Pass", "Band-Pass"]);
}

// ============================================================================
// describe_instrument
// ============================================================================

#[test]
fn describe_instrument_saw() {
    let out = reference::describe_instrument("saw");
    common::assert_contains_all(&out, &["Saw", "Oscillators", "Parameters:"]);
}

#[test]
fn describe_instrument_case_insensitive() {
    let out = reference::describe_instrument("RHODES");
    // Should resolve to Electric Piano
    common::assert_contains(&out, "Electric Piano");
}

#[test]
fn describe_instrument_alias() {
    let out = reference::describe_instrument("sawtooth");
    common::assert_contains(&out, "Saw");
}

#[test]
fn describe_instrument_unknown() {
    let out = reference::describe_instrument("nonexistent");
    common::assert_contains(&out, "Unknown instrument");
}

// ============================================================================
// describe_effect
// ============================================================================

#[test]
fn describe_effect_reverb() {
    let out = reference::describe_effect("reverb");
    common::assert_contains_all(&out, &["Reverb", "Parameters:"]);
}

#[test]
fn describe_effect_alias() {
    let out = reference::describe_effect("echo");
    common::assert_contains(&out, "Delay");
}

#[test]
fn describe_effect_unknown() {
    let out = reference::describe_effect("nonexistent");
    common::assert_contains(&out, "Unknown effect");
}
