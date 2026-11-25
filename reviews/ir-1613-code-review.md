---
id: SR-10230840607607
title: "code-review and rust-review of IR-1613 string-edge and clippy lint fix"
type: SpecReview
analysis: base
scope: "agent-ix/filament-contract-ir; PR #1171 at 19f32a09 (base origin/main 4f375562); crates/filament-contract-model/src/certificate_identity.rs, crates/filament-contract-model/src/checked_package/v2/{composition_coordinates.rs,exact_projection.rs,exact_projection/model_union.rs,intake.rs,mod.rs,model_members.rs,native_join_storage.rs,operations.rs,requirement_records.rs,typed_terms.rs}, tests/{conformance_fspec/main.rs,ir1584_reader_memory.rs,it/checked_package_v2_guarantee_composition.rs,it/checked_package_v2_model_fields.rs,it/support/fspec_inputs.rs}"
review_set: subset
---

## Summary

Code review with the rust-review lane folded in, for filament-contract-ir PR #1171. Ticket: IR-1613. The PR claims a lint-only change: 14 `make string-edge` occurrences typed or marked, the clippy `-D warnings` lanes and test warnings cleared, one FR-061 sentence, and one missing match arm in a conformance test. It claims no wire-format, diagnostic-code or public-behaviour change. Each retyped decision was checked for equivalence with the string comparison it replaces, including unknown values.

The review was read-only against the ticket worktree and ran no cargo. Evidence comes from the coder's gate logs (`gates-lead-conf/IR-1613-*`): string-edge exit 0, fmt exit 0, both clippy lanes exit 0, and the workspace `cargo check --all-targets --all-features` exit 0 on head 19f32a09. CI run 45587259057 on this head passed Format and Lint. Its Test step failed with 122 failures in `--test it` (IR-1614).

## Examined scope

- `model_union.rs` Clock payload check (examined). The old check was `payload_type != "ix://filament/native/Clock"`. The new one is `NativeName::from_identity(payload_type) != Some(Known(Clock))`. `NATIVE_PREFIX` is `ix://filament/native/`, and `NativeTypeRef::from_name` maps only the exact text `Clock` to `Clock`. So the new check accepts exactly the one identity the old one did, and refuses every other string, including unknown native names (`Known(_)`/`Unknown`) and strings without the prefix (`None`). The removed `CLOCK_IDENTITY` constant has no other users.
- `model_members.rs` `NativeName`, `NativeNaming`, `names_a_native` and `read_semantic_ir` (examined). `Alias` is returned exactly where `kind == "alias"` was true. `Bound(_)` is returned exactly where the old code had a value-type meaning plus a `target.starts_with(NATIVE_PREFIX)`, because `strip_prefix` succeeds on the same inputs. `alias_count` is unchanged. Inside the reader, `Bound` maps one-to-one to the old `kind != "alias"` branch. `Unknown` is the old `from_name(..).is_none()` case, which refuses at `target`. `Known(Clock)` is the old literal `== ".../Clock"` test. The scalar refusal, the constraints vocabulary and the alias insertion all keep their order and positions.
- `TermBindingName::Level` and the `kind()` arm (examined). `from_wire` maps only the exact text `"level"` to `Level`. Nothing compares against `TermBindingName::Other` directly: every other use matches a named variant. `CheckedBindingName::kind()` folds `Level` into `Other(&self.raw)`, so the public `CheckedBindingNameKind` for `"level"` is still `Other("level")`. `operations.rs` `parameter_name_level` matches `Name`/`Level` through `internal_kind()`, which is computed from the same raw text, so it is equivalent to the old `"name"`/`"level"` match.
- `requirement_records.rs` `retain_rows` claim-form match (examined). The old code dispatched on the `semantic_form` text. The new code dispatches on `kinds[position]`. `kinds` is built one entry per node, in node order, from `CheckedNodeKind::decode(tag, semantic_form)`. `decode` is strict per tag and admission refuses any node it cannot decode. `validate` already requires each record to be `Claim(ValueValidity)` or `Claim(OperationContract)`. So `Claim(ValueValidity)` and `Claim(OperationContract)` select exactly the nodes the two text arms selected. `kinds.get(position)?` returns `None` where `nodes.get` did.
- `requirement_records.rs` `occurrence_role_from_wire` (examined). It has the same six literals and the same refusal (`None`) as the inline match, and they agree with the serde `snake_case` names of `CheckedOccurrenceRole`. The role is now mapped before the ordinal is read. Neither read has side effects and both paths return `None`, so this is equivalent. The function parses a wire literal, so it is a real intake edge.
- `#[string_edge]` placements (examined). `NativeName::from_identity` parses a wire or caller-row identity string, and `occurrence_role_from_wire` parses a record's literal. Both are real decode edges, not markers that only hide a comparison. `string-edge-rules.json` still has an empty `allow_list`.
- `intake.rs` `MAX_SAFE` renamed to `MAX_SAFE_DIGITS` (examined). The constant is the decimal digit string of 2^53-1, used for a length-then-lexicographic comparison, so the new name is accurate. It also no longer shares a name with the numeric `MAX_SAFE: u64` in the same file (line 568) and in `protocol_locations.rs`. Those numeric comparisons were the scanner's by-name false positive.
- Clippy rewrites (examined). `rank & rank.wrapping_neg()` is the same as `isolate_lowest_one()` for `usize`, and MSRV is 1.98.1. `map_or_else(|s| s, |f| f)` is the same as `unwrap_or_else(|s| s)` on `Result<usize, usize>`. The `filter_map(.. .then(..))` count was split into `filter` + `map` with the same count, which only sizes a table. Removing the unused `holder` parameter changes no call. In `certificate_identity` the `Vec<Box<_>>` became `Vec<_>`, and the drop is still iterative: each popped node's path is drained before it drops. `u64::try_from(u64)` was removed. `[x.clone()]` became `slice::from_ref`. Two functions were moved below test modules, and their bodies are byte-identical.
- `#[allow]` attributes (examined). `clippy::too_many_arguments` is allowed on three private functions, each with a reason. `clippy::large_enum_variant` is allowed on the public `SelectedModelUnionPayload`, where boxing would change a public type. `dead_code` is allowed on two shared-support module includes in the `ir1584_reader_memory` binary and on one helper that only the conformance binary calls. `#[cfg(feature = "test-support")]` was added on `rehome`/`hosted`, whose only caller is the feature-gated `charge_events` module. Each allow is item- or include-scoped. None are crate-wide.
- `tests/conformance_fspec/main.rs` `Cause::StaleNodeKey` arm (examined). This is the missing exhaustive arm, so the target compiles again.
- Untouched surface (examined). The diff has no changes to Makefile, `.github/workflows`, `string-edge-rules.json`, `Cargo.toml`/`Cargo.lock` pins or the lint tool.
- CI run 45587259057 Test failures, 122 in `--test it` (examined). 103 refuse with `InvalidPackage`/`StaleNodeKey` at `/semantic_graph/nodes/N/owner_model`. That refusal is raised only by `protocol_body.rs` `owner_model_path_matches`, from main's #1167, and this diff does not touch that file, structural keys or owner paths. The remaining 19 fall into these groups:
  - The public-item inventory drift is `mode_switch_bindings`, `release_bindings`, `task_set` and `read_checked_package_structural`. None of these is in the diff, and the diff changes no `pub` item.
  - The quire-canonical tag test reads `Cargo.toml`, which the diff does not change.
  - The `model_enum` WorkLimit failures come from `native_join_storage::model_enum_shape` and the enum runtime charge, neither of which changed. Only `model_union` was moved, and its body is identical.
  - The nominal `root_scale` pointer, the frame shape, the reader mutation, refinement codes, source-object, probabilistic-delay and whole-read-accounting admissions run on the same fixtures as sibling tests that print `StaleNodeKey`, or on paths the diff does not touch.

  None traces to this diff.
- The coder's out-of-CI test failures (examined):
  - The model lib tests `tc_048_semantic_ir_declaration_defects...` and `tc_048_a_reference_to_a_refused_node...` expect hyphenated ids to refuse. #1123 admits them on main, and the tests are unchanged.
  - The conformance tests `path_frame_reader_rejects_bad_links_or_record_bindings` and `tc_488_a_thousand_requirement_records...` fail in two ways. One is a table-level refusal from #1150's frame-order check in `derive_package_path_conditions`, where the diff changed only an equivalent sizing count. The other is a work overrun in `body_locations`, which the diff does not touch.

  None traces to this diff.

## Verdict

Clean. Every retyped decision accepts and refuses exactly what the old string comparison did, unknown values included. The new `Level` variant leaves the public `kind()` result unchanged. The two `#[string_edge]` markers sit on real decode edges. Each `allow` is narrow and has a reason, and the renamed constant is accurately named. The lint tool, its allow list, the Makefile, the workflows and the pins are untouched. The CI Test failures (IR-1614) and the coder's six local failures all trace to main's code or fixtures, not to this diff. The behaviour-equivalence verdict comes from reading the code. The review ran no cargo, because a run on this head alone cannot separate cause by base and head.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |
