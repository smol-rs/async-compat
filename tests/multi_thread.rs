//! Behaviour of the fallback runtime under the `multi-thread` feature.
//!
//! These live in their own test binary rather than in `src/lib.rs`: the unit
//! test there asserts the fallback runtime has *not* been created yet, and
//! anything that touches `TOKIO1` in the same process would race with it.

#![cfg(feature = "multi-thread")]

use async_compat::CompatExt;

/// The reason the feature exists. `tokio::task::block_in_place` panics with
/// `can call blocking only when running on the multi-threaded runtime` unless
/// it runs on a multi-threaded worker, so a task spawned from inside `Compat`
/// can only call it when the fallback runtime is multi-threaded.
#[test]
fn block_in_place_works_in_a_task_spawned_from_compat() {
    // Deliberately outside any tokio context, so `Compat` falls back to the
    // global runtime rather than reusing an ambient one.
    let answer = futures::executor::block_on(
        async {
            tokio::spawn(async { tokio::task::block_in_place(|| 42) })
                .await
                .expect("spawned task panicked")
        }
        .compat(),
    );

    assert_eq!(answer, 42);
}

/// The fallback runtime a `Compat` future enters is the multi-threaded one.
#[test]
fn fallback_runtime_is_multi_threaded() {
    let flavor = futures::executor::block_on(
        async { tokio::runtime::Handle::current().runtime_flavor() }.compat(),
    );

    assert_eq!(flavor, tokio::runtime::RuntimeFlavor::MultiThread);
}
