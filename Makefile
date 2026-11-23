# =============================================================================
# Filament Contract IR Makefile
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the contract conformance runner for the domain corpus,
# and quire for specifications.
# =============================================================================

CARGO ?= cargo
# --locked only when no local patch is active: a patch rewrites the resolution.
LOCKED ?= $(if $(wildcard .cargo/config.toml),,--locked)
QUIRE ?= quire
STRING_EDGE ?= fsl-string-edge
# FILAMENT_SPEC_LANGUAGE_DIR names a checkout of CURRENT FSL origin/main (built with
# its own CARGO_TARGET_DIR); a stale or detached checkout emits an older wire and
# fails these lanes. No default is provided.
FILAMENT_SPEC_LANGUAGE_BIN ?= $(FILAMENT_SPEC_LANGUAGE_DIR)/target/debug/fsl
FSL_EMIT_CHECKED_V2_BIN ?= $(FILAMENT_SPEC_LANGUAGE_DIR)/target/debug/examples/emit_checked_v2

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make figures          - Regenerate docs/images from docs/images/figures.json (ixd-figures)"
	@echo "  make figures-check    - Fail when docs/images are stale; writes nothing"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make corpus           - Run the native published conformance corpus"
	@echo "  make check-corpus     - Alias for corpus (ecosystem-compatible name)"
	@echo "  make spec             - Validate and cover all Quire artifacts"
	@echo "  make release-check    - Run every local release gate"
	@echo "  make test             - Run cargo test"
	@echo "  make conformance-fspec-compile - Compile the FSpec conformance target without fixtures (CI gate)"
	@echo "  make conformance-fspec - FSpec's positive fixtures admit, adverse mutations refuse as recorded, selection identity derives (needs FILAMENT_SPECIFICATION_DIR; not in ci)"
	@echo "  make conformance-fsl-ir703 - Fresh ConfigVersion V2 producer/IR accessor check (needs FILAMENT_SPEC_LANGUAGE_DIR and its current binaries)"
	@echo "  make conformance-fsl-ir769 - Fresh FSL scalar result context through admitted IR (needs FILAMENT_SPEC_LANGUAGE_DIR and its current binary)"
	@echo "  make conformance-fsl-ir779 - FSL FR-062 RR-13 path conditions through admitted IR (needs FILAMENT_SPEC_LANGUAGE_DIR and its current binary)"
	@echo "  make conformance-fsl-exact-producer - Real FSL selected identities and execution points through IR (needs FILAMENT_SPEC_LANGUAGE_DIR and FILAMENT_SPECIFICATION_DIR; sequential native helpers)"
	@echo "  make conformance-fsl-ir760 - FSL-emitted Population<T>[N] through the default-feature model consumer (needs FILAMENT_SPEC_LANGUAGE_DIR)"
	@echo "  make conformance-fsl-ir1052 - Fresh FSL Boolean transition V2 readback (needs FSL emitter)"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - Run all cargo-deny policy checks"
	@echo "  make use-local        - Patch first-party git deps to sibling checkouts (.cargo/config.toml)"
	@echo "  make use-remote       - Remove the local patch file and restore Cargo.lock; build from GitHub"
	@echo "  make ci               - All local release gates"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: string-edge
string-edge:
	$(STRING_EDGE) --base-dir . --crate-root src --crate-root crates/filament-contract-model/src --rules string-edge-rules.json

.PHONY: lint
lint: string-edge
	$(CARGO) clippy $(LOCKED) --workspace --all-targets --features conformance-cli -- -D warnings
	# The workspace lane turns on the model's test-only fault-injection feature
	# through the root dev-dependency; this lane checks the model as a consumer
	# builds it, with the feature off.
	$(CARGO) clippy $(LOCKED) -p filament-contract-model -- -D warnings

.PHONY: corpus
corpus:
	$(CARGO) run $(LOCKED) --quiet --features conformance-cli --bin filament-contract-conformance -- run --corpus corpus/contract-v0.1 --schemas schemas

.PHONY: check-corpus
check-corpus: corpus

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'plan/**/*.md' 'reviews/**/*.md' --summary
	$(QUIRE) coverage --scope . --strict

.PHONY: test
test:
	$(CARGO) test $(LOCKED) --workspace --all-targets --features conformance-cli -- --include-ignored
	# The model's own doctests with its test-only fault-injection feature off:
	# they prove a default build does not export that surface (FR-019-AC-4).
	$(CARGO) test $(LOCKED) -p filament-contract-model --doc
	$(CARGO) test $(LOCKED) -p filament-contract-model --test source_obligation_public
	# The root crate's doctests, which `--all-targets` leaves out: the
	# `compile_fail` probes of the non-success constructor (FR-030-AC-6).
	$(CARGO) test $(LOCKED) -p filament-contract-ir --doc

# FR-038-AC-107/176: FSpec's CheckedPackage V2 fixtures, structural node ids,
# owners and complete bodies agree with the production reader. Reads the
# checkout named by FILAMENT_SPECIFICATION_DIR and fails, never skips, when it is
# unset, empty, or holds no proposals/checked-package-v2/fixtures/, or when a
# fixture does not admit. Fixture execution stays outside `test` and `ci`:
# they have no checkout.
.PHONY: conformance-fspec-compile
conformance-fspec-compile:
	$(CARGO) test $(LOCKED) --workspace --features conformance-cli --no-run --test conformance_fspec --test monitor_plan_public --test temporal_binding_public --test it

.PHONY: conformance-fspec
conformance-fspec:
	$(CARGO) test $(LOCKED) --workspace --lib ir1083_
	$(CARGO) test $(LOCKED) -p filament-contract-model --lib --features fspec-conformance selected_closure_walk -- --include-ignored
	$(CARGO) test $(LOCKED) --test conformance_fspec
	$(CARGO) test $(LOCKED) -p filament-contract-model --test monitor_plan_public
	@output=$$(FSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) --test conformance_fspec tc_048_fspec_all_positive_packages_and_owner_identities -- --exact 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required FSpec positive package identity test did not run exactly once' >&2; exit 1; }
	@output=$$(FSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) --features conformance-cli --test it checked_package_v2_owners::tc_229_fspec_ 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 2 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required FSpec population fixture tests did not run exactly once' >&2; exit 1; }
	@output=$$(FSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) --features conformance-cli --test it checked_package_v2_protocol_body::compensation_clock_rows_follow_published_cases -- --exact 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required FSpec compensation clock vector test did not run exactly once' >&2; exit 1; }
	@output=$$(FSPEC_CONFORMANCE=1 $(CARGO) test $(LOCKED) -p filament-contract-model --test temporal_binding_public default_feature_consumer_borrows_admitted_clock_rows -- --exact 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required FSpec temporal binding public consumer did not run exactly once' >&2; exit 1; }

# TC-445's producer-origin case is explicit: the FSL checkout supplies its own
# current fixture and compiler executables, and the test records the revision
# and commands. Ordinary IR tests have no external FSL build dependency.
.PHONY: conformance-fsl-ir703
conformance-fsl-ir703:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo 'FILAMENT_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(FILAMENT_SPEC_LANGUAGE_DIR)/target/debug/examples/config_version_fixtures" || { echo 'build the current FSL config_version_fixtures example first' >&2; exit 1; }
	@test -x "$(FILAMENT_SPEC_LANGUAGE_DIR)/target/debug/fsl" || { echo 'build the current FSL fsl binary first' >&2; exit 1; }
	@output=$$(IR703_FSL_CONFORMANCE=1 $(CARGO) test $(LOCKED) --workspace --test it checked_package_v2_comparison_operands::tc_445_fresh_fsl_config_version_has_shared_field_with_post_and_pre_observations -- --exact --nocapture 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required TC-445 FSL producer test did not run exactly once' >&2; exit 1; }

.PHONY: conformance-fsl-ir769
conformance-fsl-ir769:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo 'FILAMENT_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(FILAMENT_SPEC_LANGUAGE_BIN)" || { echo 'build the current FSL fsl binary first' >&2; exit 1; }
	@output=$$(FILAMENT_SPEC_LANGUAGE_BIN="$(FILAMENT_SPEC_LANGUAGE_BIN)" $(CARGO) test $(LOCKED) --test conformance_fsl_ir769 -- --nocapture 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required TC-447 FSL producer test did not run exactly once' >&2; exit 1; }

.PHONY: conformance-fsl-exact-producer-compile conformance-fsl-exact-producer
conformance-fsl-exact-producer-compile:
	$(CARGO) test $(LOCKED) --workspace --no-run --test conformance_fsl_exact_producer

# Finish outer Cargo before its native binary starts the isolated Cargo helper.
conformance-fsl-exact-producer:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo 'FILAMENT_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -n "$(FILAMENT_SPECIFICATION_DIR)" || { echo 'FILAMENT_SPECIFICATION_DIR is required' >&2; exit 1; }
	@artifacts=$$($(CARGO) test $(LOCKED) --workspace --no-run --test conformance_fsl_exact_producer --message-format=json) || exit $$?; \
	binary=$$(printf '%s\n' "$$artifacts" | python3 -c 'import json,sys; rows=[json.loads(line) for line in sys.stdin]; bins=[row["executable"] for row in rows if row.get("reason")=="compiler-artifact" and row.get("target",{}).get("name")=="conformance_fsl_exact_producer" and row.get("executable")]; assert len(bins)==1, bins; print(bins[0])') || exit $$?; \
	"$$binary" --nocapture --test-threads=1

.PHONY: conformance-fsl-ir779
conformance-fsl-ir779:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo 'FILAMENT_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(FILAMENT_SPEC_LANGUAGE_BIN)" || { echo 'build the current FSL fsl binary first' >&2; exit 1; }
	@output=$$(FILAMENT_SPEC_LANGUAGE_DIR="$(FILAMENT_SPEC_LANGUAGE_DIR)" FILAMENT_SPEC_LANGUAGE_BIN="$(FILAMENT_SPEC_LANGUAGE_BIN)" FILAMENT_SPEC_LANGUAGE_TARGET_DIR="$(FILAMENT_SPEC_LANGUAGE_TARGET_DIR)" $(CARGO) test $(LOCKED) --test conformance_fsl_ir779 -- --nocapture 2>&1); \
	status=$$?; printf '%s\n' "$$output"; \
	[ "$$status" -eq 0 ] || exit "$$status"; \
	passed=$$(printf '%s\n' "$$output" | grep -c '^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured;' || true); \
	[ "$$passed" -eq 1 ] || { echo 'required TC-450 FSL producer test did not run exactly once' >&2; exit 1; }

.PHONY: conformance-fsl-ir760
conformance-fsl-ir760:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo 'FILAMENT_SPEC_LANGUAGE_DIR is required: a checkout of current FSL origin/main, not a stale detached checkout' >&2; exit 1; }
	@FILAMENT_SPEC_LANGUAGE_DIR="$(FILAMENT_SPEC_LANGUAGE_DIR)" FILAMENT_SPEC_LANGUAGE_TARGET_DIR="$(CURDIR)/target/fsl-producer" CARGO_TARGET_DIR="$(CURDIR)/target" $(CARGO) test $(LOCKED) --manifest-path crates/filament-contract-model/Cargo.toml --test selected_population_public -- --nocapture
	@FILAMENT_SPEC_LANGUAGE_DIR="$(FILAMENT_SPEC_LANGUAGE_DIR)" FILAMENT_SPEC_LANGUAGE_TARGET_DIR="$(CURDIR)/target/fsl-producer" CARGO_TARGET_DIR="$(CURDIR)/target" $(CARGO) test $(LOCKED) --test conformance_fsl_ir760 -- --nocapture

.PHONY: conformance-fsl-ir1052
conformance-fsl-ir1052:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo 'FILAMENT_SPEC_LANGUAGE_DIR is required' >&2; exit 1; }
	@test -x "$(FSL_EMIT_CHECKED_V2_BIN)" || { echo 'FSL_EMIT_CHECKED_V2_BIN must name the current FSL emit_checked_v2 example binary' >&2; exit 1; }
	@FILAMENT_SPEC_LANGUAGE_DIR="$(FILAMENT_SPEC_LANGUAGE_DIR)" FSL_EMIT_CHECKED_V2_BIN="$(FSL_EMIT_CHECKED_V2_BIN)" $(CARGO) test $(LOCKED) --test conformance_fsl_ir1052 -- --nocapture

.PHONY: build
build:
	$(CARGO) build $(LOCKED) --workspace --release
	$(CARGO) build $(LOCKED) -p filament-contract-ir --release --features conformance-cli --bin filament-contract-conformance

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

# The Quire CLI owns first-party Git lock coherence (FR-055).
.PHONY: deny
deny:
	$(CARGO) deny check
	$(QUIRE) check-lock Cargo.lock

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

# =============================================================================
# Local development against sibling checkouts
#
# `use-local` writes a gitignored .cargo/config.toml that patches each
# first-party git dependency to its working tree at $(SIBLINGS)/<repo>, uncommitted
# edits included. `use-local` first snapshots Cargo.lock to the gitignored
# .cargo/Cargo.lock.pre-local; `use-remote` deletes the config and restores the
# lock from that snapshot (and does nothing to the lock if there is none). So the
# lock returns to its state before the first `use-local`; lock changes made while
# a patch is active are discarded, and a `cargo update -p` made without a patch is kept.
# Format: <repo>:<crate>:<crate-dir>; entries are grouped by repo here, in any
# order, so each repo gets exactly one [patch] table.
# SIBLINGS is the directory holding the sibling clones: the parent of the main
# checkout, so it is also right from a linked worktree. Override to relocate.
# =============================================================================

SIBLINGS ?= $(abspath $(shell git rev-parse --path-format=absolute --git-common-dir)/../..)
LOCAL_PATCHES ?= filament-verification-contracts:filament-verification-contracts:. ix-trace-rs:ix-trace-rs:. quire-canonical:quire-canonical:. quire-canonical:quire-canonical-derive:quire-canonical-derive filament-walk:filament-walk:.

.PHONY: use-local
use-local:
	@set -e; mkdir -p .cargo; \
	for spec in $(LOCAL_PATCHES); do \
	  if [ "$$(printf '%s' "$$spec" | tr -cd ':' | wc -c)" != 2 ] || printf '%s' "$$spec" | grep -q '::\|^:\|:$$'; then \
	    echo "use-local: malformed LOCAL_PATCHES entry '$$spec' (want repo:crate:dir)" >&2; exit 1; \
	  fi; \
	  repo=$${spec%%:*}; rest=$${spec#*:}; dir=$${rest#*:}; \
	  if [ ! -f "$(SIBLINGS)/$$repo/$$dir/Cargo.toml" ]; then \
	    echo "use-local: $(SIBLINGS)/$$repo is not cloned (no Cargo.toml at $(SIBLINGS)/$$repo/$$dir); clone agent-ix/$$repo next to this repo" >&2; exit 1; \
	  fi; \
	done; \
	[ -f .cargo/Cargo.lock.pre-local ] || cp Cargo.lock .cargo/Cargo.lock.pre-local; \
	: > .cargo/config.toml; \
	repos=$$(for spec in $(LOCAL_PATCHES); do printf '%s\n' "$${spec%%:*}"; done | awk '!seen[$$0]++'); \
	first=1; \
	for repo in $$repos; do \
	  [ "$$first" = 1 ] || printf '\n' >> .cargo/config.toml; first=0; \
	  printf '[patch."https://github.com/agent-ix/%s"]\n' "$$repo" >> .cargo/config.toml; \
	  for spec in $(LOCAL_PATCHES); do \
	    [ "$${spec%%:*}" = "$$repo" ] || continue; \
	    rest=$${spec#*:}; crate=$${rest%%:*}; dir=$${rest#*:}; \
	    printf '%s = { path = "%s/%s/%s" }\n' "$$crate" "$(SIBLINGS)" "$$repo" "$$dir" >> .cargo/config.toml; \
	  done; \
	done; echo "wrote .cargo/config.toml"; \
	meta=$$(mktemp); \
	if ! $(CARGO) metadata --format-version 1 >/dev/null 2>"$$meta"; then \
	  cat "$$meta" >&2; rm -f "$$meta" .cargo/config.toml; \
	  [ ! -f .cargo/Cargo.lock.pre-local ] || { cp .cargo/Cargo.lock.pre-local Cargo.lock; rm -f .cargo/Cargo.lock.pre-local; }; \
	  echo "use-local: cargo metadata failed under the patch" >&2; exit 1; \
	fi; \
	if grep -q 'patch .* was not used' "$$meta"; then \
	  cat "$$meta" >&2; rm -f "$$meta" .cargo/config.toml; \
	  [ ! -f .cargo/Cargo.lock.pre-local ] || { cp .cargo/Cargo.lock.pre-local Cargo.lock; rm -f .cargo/Cargo.lock.pre-local; }; \
	  echo "use-local: a patch was not used; the sibling's version does not satisfy the requirement" >&2; exit 1; \
	fi; \
	rm -f "$$meta"

.PHONY: use-remote
use-remote:
	rm -f .cargo/config.toml
	@if [ -f .cargo/Cargo.lock.pre-local ]; then mv .cargo/Cargo.lock.pre-local Cargo.lock; echo "restored Cargo.lock from the pre-local snapshot"; fi

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test conformance-fspec-compile corpus spec deny cargo-audit

.PHONY: release-check
release-check: ci

# TC-041 / FR-028-AC-4: explicit native production configurations. Reuses the
# worktree target; this named lane also runs the build control by itself.
.PHONY: model-boundary-builds
model-boundary-builds:
	$(CARGO) test $(LOCKED) --workspace --test it cycle_free_model::production_default_all_feature_and_msrv_builds -- --ignored --exact --nocapture

.PHONY: conformance-fsl-ir1384
conformance-fsl-ir1384:
	@test -n "$(FILAMENT_SPEC_LANGUAGE_DIR)" || { echo "conformance-fsl-ir1384 reads the owner's domain package: set FILAMENT_SPEC_LANGUAGE_DIR"; exit 2; }
	cp Cargo.lock tests/conformance_fsl_ir1384/Cargo.lock
	FILAMENT_SPEC_LANGUAGE_DIR="$(FILAMENT_SPEC_LANGUAGE_DIR)" $(CARGO) test --workspace --manifest-path tests/conformance_fsl_ir1384/Cargo.toml -- --nocapture

# =============================================================================
# Documentation figures
# =============================================================================
# docs/images/figures.json defines the figures; ixd-figures (agent-ix/ix-diagrams) renders them.
# Install it with: cargo install --git https://github.com/agent-ix/ix-diagrams ixd-figures

IXD_FIGURES ?= ixd-figures

.PHONY: figures
figures:
	$(IXD_FIGURES) generate --definitions docs/images/figures.json

.PHONY: figures-check
figures-check:
	$(IXD_FIGURES) check --definitions docs/images/figures.json
