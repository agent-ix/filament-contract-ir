// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-1584 reader memory. A counting global allocator records the live bytes
//! and every byte requested while a strict V2 read runs.
//!
//! The ceiling test bounds three multiples of the package bytes for a chain of
//! 4,000 and 8,000 nodes, and requires them not to grow when the package
//! doubles:
//! - the cumulative read storage the budget charged;
//! - the live heap peak during the read;
//! - the heap the admitted package retains.
//!
//! It also requires every requested byte to be charged. With
//! `test-support`, an ignored lane prints the same figures by read stage,
//! attributing each charge, and the live peak reached while its stage was
//! active, through the native budget observer.

#[path = "it/support/mod.rs"]
mod support;

#[path = "it/support/fspec_read_proof.rs"]
mod fspec_read_proof;
use fspec_read_proof::read_and_account;

use filament_contract_model::{
    CheckedPackageReadLimits, CheckedPackageV2, CheckedPackageV2BudgetedReadResult,
    NativeJoinBudget,
};
use ix_trace_rs::trace;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Mutex;
use support::checked_package::{evidence_for, v2_all_families, v2_reference_chain};

/// Net bytes the armed thread allocated: a block allocated before arming and
/// released while armed counts negative.
static LIVE: AtomicI64 = AtomicI64::new(0);
/// The largest `LIVE` since the active stage began.
static STAGE_PEAK: AtomicI64 = AtomicI64::new(0);
/// The largest `LIVE` since the read began.
static PEAK: AtomicI64 = AtomicI64::new(0);
/// Every byte the armed thread requested from the allocator, a reallocation
/// counted at its full new size, as the read's budget charges it.
static REQUESTED: AtomicU64 = AtomicU64::new(0);
/// The counters are process-wide: one measured read at a time.
static MEASURING: Mutex<()> = Mutex::new(());

thread_local! {
    /// Whether this thread's allocations go to the counters above. Only the
    /// reading thread is armed, only around its read and only while it holds
    /// `MEASURING` (FR-038-AC-511 measures "on the reading thread"), so the
    /// harness and other tests' threads never touch them.
    static ARMED: Cell<bool> = const { Cell::new(false) };
    /// This thread's own request total, for the shared support's
    /// `measure_fspec_read_requests`, which other tests call in parallel.
    static OWN_REQUESTS: Cell<Option<u64>> = const { Cell::new(None) };
}

fn armed() -> bool {
    ARMED.try_with(Cell::get).unwrap_or(false)
}

fn note_own_request(bytes: usize) {
    let _ = OWN_REQUESTS.try_with(|own| {
        if let Some(total) = own.get() {
            own.set(Some(total.saturating_add(bytes as u64)));
        }
    });
}

/// Runs `operation` with this thread's allocations counted.
fn counted<T>(operation: impl FnOnce() -> T) -> T {
    struct Disarm;
    impl Drop for Disarm {
        fn drop(&mut self) {
            ARMED.set(false);
        }
    }
    ARMED.set(true);
    let _disarm = Disarm;
    operation()
}

fn live() -> i64 {
    LIVE.load(Ordering::Relaxed)
}

/// `value` above `baseline`, in bytes; zero below it.
fn above(value: i64, baseline: i64) -> u64 {
    u64::try_from(value.saturating_sub(baseline)).unwrap_or(0)
}

struct Counting;

fn note_request(bytes: usize) {
    note_own_request(bytes);
    if !armed() {
        return;
    }
    REQUESTED.fetch_add(bytes as u64, Ordering::Relaxed);
    let bytes = i64::try_from(bytes).unwrap_or(i64::MAX);
    let live = LIVE
        .fetch_add(bytes, Ordering::Relaxed)
        .saturating_add(bytes);
    STAGE_PEAK.fetch_max(live, Ordering::Relaxed);
    PEAK.fetch_max(live, Ordering::Relaxed);
}

fn note_release(bytes: usize) {
    if armed() {
        LIVE.fetch_sub(i64::try_from(bytes).unwrap_or(i64::MAX), Ordering::Relaxed);
    }
}

// SAFETY: every method forwards its arguments unchanged to `System`, which
// upholds the `GlobalAlloc` contract; the counters only observe sizes.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's layout contract is forwarded unchanged.
        let block = unsafe { System.alloc(layout) };
        if !block.is_null() {
            note_request(layout.size());
        }
        block
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller's layout contract is forwarded unchanged.
        let block = unsafe { System.alloc_zeroed(layout) };
        if !block.is_null() {
            note_request(layout.size());
        }
        block
    }

    unsafe fn dealloc(&self, block: *mut u8, layout: Layout) {
        // SAFETY: `block` was allocated by `System` with `layout`.
        unsafe { System.dealloc(block, layout) };
        note_release(layout.size());
    }

    unsafe fn realloc(&self, block: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: `block` was allocated by `System` with `layout`.
        let moved = unsafe { System.realloc(block, layout, new_size) };
        if !moved.is_null() {
            // A move holds both blocks while it copies: the new block is
            // live before the old one is released.
            note_request(new_size);
            note_release(layout.size());
        }
        moved
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// One measured read.
struct Measured {
    package_bytes: u64,
    /// Read storage the budget admitted.
    charged: u64,
    /// Bytes requested from the allocator while the read ran.
    requested: u64,
    /// Largest live heap above the pre-read baseline.
    peak: u64,
    /// Live heap the admitted package holds above the baseline.
    retained: u64,
}

impl Measured {
    fn multiples(&self) -> (f64, f64, f64) {
        (
            ratio(self.charged, self.package_bytes),
            ratio(self.peak, self.package_bytes),
            ratio(self.retained, self.package_bytes),
        )
    }
}

fn ratio(value: u64, bytes: u64) -> f64 {
    value as f64 / bytes as f64
}

fn generous_limits(nodes: usize, bytes: usize) -> CheckedPackageReadLimits {
    let count = u64::try_from(nodes).expect("node count");
    CheckedPackageReadLimits {
        bytes: u64::try_from(bytes).expect("byte count") + 1024,
        nodes: count * 2,
        edges: count * 16,
        occurrences: count * 16,
        diagnostics: 1 << 20,
        work: count * 1024,
    }
}

/// Reads a chain of `length` nodes, calling `read` around the measured read
/// (the stage lane installs its observer there).
fn measure(
    length: usize,
    read: impl FnOnce(
        &mut dyn FnMut() -> CheckedPackageV2BudgetedReadResult,
    ) -> CheckedPackageV2BudgetedReadResult,
) -> Measured {
    let base = v2_all_families();
    let evidence = evidence_for(&base);
    let nodes = length
        + base["semantic_graph"]["nodes"]
            .as_array()
            .expect("nodes")
            .len();
    drop(base);
    let (bytes, _) = v2_reference_chain(length);
    let limits = generous_limits(nodes, bytes.len());
    let mut budget = NativeJoinBudget::new(limits.work, u64::MAX).with_read_storage_limit(u64::MAX);
    let baseline = live();
    STAGE_PEAK.store(baseline, Ordering::Relaxed);
    PEAK.store(baseline, Ordering::Relaxed);
    let requested_before = REQUESTED.load(Ordering::Relaxed);
    let result = counted(|| {
        read(&mut || CheckedPackageV2::read_with_budget(&bytes, limits, &evidence, &mut budget))
    });
    let requested = REQUESTED.load(Ordering::Relaxed) - requested_before;
    let peak = above(PEAK.load(Ordering::Relaxed), baseline);
    let retained = above(live(), baseline);
    let CheckedPackageV2BudgetedReadResult::Admitted(package) = result else {
        panic!("the chain admits: {result:?}");
    };
    assert_eq!(package.graph().nodes.len(), nodes);
    drop(package);
    Measured {
        package_bytes: bytes.len() as u64,
        charged: budget.read_storage_consumed(),
        requested,
        peak,
        retained,
    }
}

/// Cumulative read storage, at most this many times the package bytes
/// (measured 29.59x at 4,026 nodes and 29.54x at 8,026 after IR-1584;
/// 29.84x and 29.80x before).
const CHARGED_CEILING: f64 = 31.0;
/// Live heap peak during the read (measured 10.85x; the canonical writer's
/// staged copies over the retained document).
const PEAK_CEILING: f64 = 11.5;
/// Heap the admitted package retains (measured 8.47x; the shared document is
/// 6.36x of it).
const RETAINED_CEILING: f64 = 9.0;
/// How far a multiple may move when the package doubles: rounding of
/// capacity steps, not growth.
const DOUBLING_SLACK: f64 = 0.05;

/// FR-038 read storage is truthful (every requested byte is charged) and
/// linear: the charged, peak and retained multiples stay under their
/// ceilings and do not grow when the package doubles.
#[trace("TC-048", "FR-038-AC-214")]
#[test]
fn reader_memory_multiples_stay_under_ceilings_and_do_not_grow_with_size() {
    let _one = MEASURING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let small = measure(4_000, |read| read());
    let large = measure(8_000, |read| read());
    for measured in [&small, &large] {
        assert_eq!(
            measured.requested, measured.charged,
            "every requested byte is charged"
        );
        let (charged, peak, retained) = measured.multiples();
        eprintln!(
            "package_bytes={} charged={charged:.2}x live_peak={peak:.2}x retained={retained:.2}x",
            measured.package_bytes
        );
        assert!(charged <= CHARGED_CEILING, "charged {charged:.2}x");
        assert!(peak <= PEAK_CEILING, "live peak {peak:.2}x");
        assert!(retained <= RETAINED_CEILING, "retained {retained:.2}x");
    }
    let (small_charged, small_peak, small_retained) = small.multiples();
    let (large_charged, large_peak, large_retained) = large.multiples();
    assert!(large_charged <= small_charged + DOUBLING_SLACK);
    assert!(large_peak <= small_peak + DOUBLING_SLACK);
    assert!(large_retained <= small_retained + DOUBLING_SLACK);
}

#[cfg(feature = "test-support")]
mod by_stage {
    use super::*;
    use filament_contract_model::{
        with_native_budget_observer, NativeBudgetEvent, NativeBudgetResource,
        NativeJoinLocusSource, PackageReadStage,
    };
    use std::sync::atomic::AtomicUsize;

    /// Attribution buckets: every `PackageReadStage`, then "other".
    const BUCKETS: usize = 24;
    const OTHER: usize = BUCKETS - 1;
    const NO_STAGE: usize = usize::MAX;

    static ACTIVE: AtomicUsize = AtomicUsize::new(NO_STAGE);
    static CHARGED: [AtomicU64; BUCKETS] = [const { AtomicU64::new(0) }; BUCKETS];
    static STAGE_PEAKS: [AtomicI64; BUCKETS] = [const { AtomicI64::new(0) }; BUCKETS];

    /// The stages in read order.
    const STAGES: [(PackageReadStage, &str); 17] = [
        (PackageReadStage::Envelope, "envelope"),
        (PackageReadStage::Canonical, "canonical"),
        (PackageReadStage::WireDecode, "wire_decode"),
        (PackageReadStage::WireChecks, "wire_checks"),
        (PackageReadStage::NodeIndex, "node_index"),
        (PackageReadStage::Lock, "lock"),
        (PackageReadStage::ModelDocument, "model_document"),
        (PackageReadStage::Graph, "graph"),
        (PackageReadStage::SourceMap, "source_map"),
        (PackageReadStage::Equivalence, "equivalence"),
        (PackageReadStage::Extension, "extension"),
        (PackageReadStage::Capabilities, "capabilities"),
        (PackageReadStage::Diagnostics, "diagnostics"),
        (PackageReadStage::OperationEffects, "operation_effects"),
        (PackageReadStage::OperationBodies, "operation_bodies"),
        (PackageReadStage::CheckedTermData, "checked_term_data"),
        (PackageReadStage::Retained, "retained"),
    ];

    fn bucket(event: &NativeBudgetEvent) -> usize {
        match event.locus.map(|locus| locus.source) {
            Some(NativeJoinLocusSource::PackageRead(stage, _)) => stage as usize,
            _ => OTHER,
        }
    }

    /// Closes the active stage's live peak and opens `next`.
    fn switch_to(next: usize) {
        let previous = ACTIVE.swap(next, Ordering::Relaxed);
        if previous != NO_STAGE && previous != next {
            STAGE_PEAKS[previous].fetch_max(STAGE_PEAK.load(Ordering::Relaxed), Ordering::Relaxed);
            STAGE_PEAK.store(live(), Ordering::Relaxed);
        }
    }

    /// The observer: allocates nothing.
    fn observe(event: NativeBudgetEvent) {
        if event.resource != NativeBudgetResource::ReadStorage || !event.admitted {
            return;
        }
        let stage = bucket(&event);
        switch_to(stage);
        CHARGED[stage].fetch_add(
            u64::try_from(event.amount).unwrap_or(u64::MAX),
            Ordering::Relaxed,
        );
    }

    /// Prints the stage table for 2,000, 4,000 and 8,000 chained nodes:
    /// each stage's charged read storage and the live peak while it was
    /// active, as multiples of the package bytes.
    #[test]
    #[ignore = "IR-1584 measurement lane; run explicitly"]
    fn reader_memory_by_stage() {
        let _one = MEASURING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for length in [2_000, 4_000, 8_000] {
            for (charged, peak) in CHARGED.iter().zip(&STAGE_PEAKS) {
                charged.store(0, Ordering::Relaxed);
                peak.store(0, Ordering::Relaxed);
            }
            ACTIVE.store(NO_STAGE, Ordering::Relaxed);
            let mut baseline = 0;
            let measured = measure(length, |read| {
                baseline = live();
                let result = with_native_budget_observer(observe, read);
                switch_to(NO_STAGE);
                result
            });
            let bytes = measured.package_bytes;
            let (charged, peak, retained) = measured.multiples();
            eprintln!(
                "chain={length} package_bytes={bytes} charged={charged:.2}x live_peak={peak:.2}x retained={retained:.2}x requested_equals_charged={}",
                measured.requested == measured.charged
            );
            let mut seen = [false; BUCKETS];
            let rows = STAGES
                .iter()
                .map(|(stage, name)| (*stage as usize, *name))
                .chain([(OTHER, "other")]);
            for (index, name) in rows {
                if std::mem::replace(&mut seen[index], true) {
                    continue;
                }
                let charged = CHARGED[index].load(Ordering::Relaxed);
                if charged > 0 {
                    let peak = above(STAGE_PEAKS[index].load(Ordering::Relaxed), baseline);
                    eprintln!(
                        "  stage={name:<18} charged={:>6.2}x live_peak={:>6.2}x",
                        ratio(charged, bytes),
                        ratio(peak, bytes),
                    );
                }
            }
        }
    }

    /// The exact arena bytes of the subtree at `root` at the element sizes of
    /// quire-canonical v0.3.1: a 32-byte slot per value, an 8-byte item per
    /// array element, a 24-byte member per object member, and decoded text.
    fn arena_bytes(root: quire_canonical::NodeRef<'_>) -> u64 {
        let (mut slots, mut items, mut members, mut text) = (0u64, 0u64, 0u64, 0u64);
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            slots += 1;
            match node.node() {
                quire_canonical::Node::Array(children) => {
                    for child in children {
                        items += 1;
                        stack.push(child);
                    }
                }
                quire_canonical::Node::Object(children) => {
                    for (name, child) in children {
                        members += 1;
                        text += name.len() as u64;
                        stack.push(child);
                    }
                }
                quire_canonical::Node::String(value) => text += value.len() as u64,
                quire_canonical::Node::Number(number) => text += number.text().len() as u64,
                _ => {}
            }
        }
        slots * 32 + items * 8 + members * 24 + text
    }

    /// Prints the package document's retained bytes, its exact arena size,
    /// and the exact arena size of the node bodies the admitted package keeps
    /// handles to.
    #[test]
    #[ignore = "IR-1584 measurement lane; run explicitly"]
    fn document_arena_sizes() {
        for length in [2_000, 8_000] {
            let (bytes, _) = v2_reference_chain(length);
            let document = quire_canonical::read(&bytes, u64::MAX).expect("read");
            let input = bytes.len() as u64;
            let retained = document.retained_bytes() as u64;
            let exact = arena_bytes(document.root());
            let bodies: u64 = document
                .root()
                .get("semantic_graph")
                .and_then(|graph| graph.get("nodes"))
                .map(|nodes| match nodes.node() {
                    quire_canonical::Node::Array(items) => items
                        .filter_map(|node| node.get("body"))
                        .map(arena_bytes)
                        .sum(),
                    _ => 0,
                })
                .unwrap_or(0);
            eprintln!(
                "chain={length} input={input} retained_bytes={:.2}x exact_arena={:.2}x body_arena={:.2}x",
                ratio(retained, input),
                ratio(exact, input),
                ratio(bodies, input),
            );
        }
    }
}

/// The bytes this thread requests while `read` runs; nested scopes restore
/// the enclosing count.
fn measure_fspec_read_requests(read: impl FnOnce()) -> u64 {
    let outer = OWN_REQUESTS.replace(Some(0));
    read();
    let requested = OWN_REQUESTS.replace(outer).unwrap_or(0);
    if let Some(total) = outer {
        OWN_REQUESTS.set(Some(total.saturating_add(requested)));
    }
    requested
}

/// Runs `operation` without counting this thread's requests.
fn without_fspec_instrumentation<T>(operation: impl FnOnce() -> T) -> T {
    let outer = OWN_REQUESTS.replace(None);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation));
    OWN_REQUESTS.set(outer);
    match result {
        Ok(value) => value,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
