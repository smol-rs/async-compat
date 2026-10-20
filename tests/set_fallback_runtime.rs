//! Configuring the fallback runtime with `set_fallback_runtime`.
//!
//! The fallback runtime is process-wide and can only be set once, so each test
//! binary for it holds a single test.

use async_compat::CompatExt;

/// The configuration applies to the runtime `Compat` falls back to. The thread
/// name shows on the blocking pool with either scheduler, and with the
/// `multi-thread` feature a task spawned from inside `Compat` runs on the
/// single configured worker, where `block_in_place` works.
#[test]
fn the_fallback_runtime_is_built_from_the_configuration() {
    async_compat::set_fallback_runtime(|builder| {
        builder.worker_threads(1).thread_name("configured-runtime");
    })
    .unwrap();

    // Deliberately outside any tokio context, so `Compat` falls back.
    let name = futures::executor::block_on(
        async {
            tokio::task::spawn_blocking(|| std::thread::current().name().map(str::to_owned))
                .await
                .expect("blocking task panicked")
        }
        .compat(),
    );
    assert_eq!(name.as_deref(), Some("configured-runtime"));

    #[cfg(feature = "multi-thread")]
    {
        let (name, answer) = futures::executor::block_on(
            async {
                tokio::spawn(async {
                    let name = std::thread::current().name().map(str::to_owned);
                    (name, tokio::task::block_in_place(|| 42))
                })
                .await
                .expect("spawned task panicked")
            }
            .compat(),
        );
        assert_eq!(name.as_deref(), Some("configured-runtime"));
        assert_eq!(answer, 42);
    }
}
