use rps2::sync::atomic::{AtomicU32, Ordering};

fn very_expensive() -> u32 {
    42
}

#[rps2_libtest::test]
fn test_lazy_lock() {
    let counter = AtomicU32::new(0);

    let lazy = rps2::sync::LazyLock::new(|| {
        counter.fetch_add(1, Ordering::Relaxed);
        very_expensive()
    });

    // Make sure it didn't activate
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    assert_eq!(*lazy, 42);

    // Make sure it triggered properly
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    assert_eq!(*lazy, 42);

    // Make sure it didn't trigger again
    assert_eq!(counter.load(Ordering::Relaxed), 1);
}
