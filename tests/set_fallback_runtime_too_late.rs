//! `set_fallback_runtime` after the fallback runtime already exists.

use async_compat::CompatExt;

/// Once a `Compat` has fallen back, the runtime it used stays in place: a
/// later configuration is refused rather than silently ignored, and nothing is
/// built from it.
#[test]
fn a_configuration_supplied_too_late_is_refused() {
    let spawned_on = || {
        futures::executor::block_on(
            async {
                tokio::spawn(async { std::thread::current().name().map(str::to_owned) })
                    .await
                    .unwrap()
            }
            .compat(),
        )
    };

    assert_eq!(spawned_on().as_deref(), Some("async-compat/tokio-1"));

    let mut configured = false;
    let err = async_compat::set_fallback_runtime(|builder| {
        configured = true;
        builder.thread_name("too-late");
    })
    .unwrap_err();
    assert!(matches!(
        err,
        async_compat::SetFallbackRuntimeError::AlreadySet
    ));
    assert!(!configured);

    assert_eq!(spawned_on().as_deref(), Some("async-compat/tokio-1"));

    // Nothing to shut down, so the error can be handled inside async code too.
    let outer = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    outer.block_on(async {
        drop(async_compat::set_fallback_runtime(|_| {}).unwrap_err());
    });
}
