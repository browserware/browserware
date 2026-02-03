//! Performance test for browser detection
//! Target: Detection should complete in <100ms
//!
//! Run with: cargo test --test performance --release -- --nocapture

use browserware_detect::{detect_browsers, detect_default_browser};
use std::time::Instant;

#[test]
fn detect_browsers_performance() {
    // Warm up (first run may involve OS caching)
    let _ = detect_browsers();

    // Measure actual performance
    let start = Instant::now();
    let browsers = detect_browsers();
    let duration = start.elapsed();

    println!("\nPerformance Results:");
    println!("  Detected {} browsers in {:?}", browsers.len(), duration);
    println!("  Target: <100ms");

    // Log individual browsers for verification
    println!("\nDetected browsers:");
    for browser in &browsers {
        println!(
            "  - {} ({}) at {}",
            browser.name,
            browser.id,
            browser.executable.display()
        );
    }

    // Warn if performance target is exceeded, but don't fail the test
    // This avoids flaky test failures in CI/debug builds while still reporting timing
    if duration.as_millis() >= 100 {
        eprintln!(
            "WARNING: Detection took {}ms, exceeds 100ms target (not failing in non-release builds)",
            duration.as_millis()
        );
    }

    // Only enforce performance in release builds
    #[cfg(not(debug_assertions))]
    assert!(
        duration.as_millis() < 100,
        "Detection took {}ms, exceeds 100ms target",
        duration.as_millis()
    );
}

#[test]
fn detect_default_browser_performance() {
    // Warm up
    let _ = detect_default_browser();

    // Measure actual performance
    let start = Instant::now();
    let default = detect_default_browser();
    let duration = start.elapsed();

    println!("\nDefault Browser Detection:");
    if let Some(browser) = default {
        println!("  Found: {} ({}) in {duration:?}", browser.name, browser.id);
    } else {
        println!("  No default browser in {duration:?}");
    }
    println!("  Target: <50ms");

    // Warn if performance target is exceeded, but don't fail the test
    if duration.as_millis() >= 50 {
        eprintln!(
            "WARNING: Default detection took {}ms, exceeds 50ms target (not failing in non-release builds)",
            duration.as_millis()
        );
    }

    // Only enforce performance in release builds
    #[cfg(not(debug_assertions))]
    assert!(
        duration.as_millis() < 50,
        "Default detection took {}ms, exceeds 50ms target",
        duration.as_millis()
    );
}

#[test]
fn multiple_detection_calls_performance() {
    // Test that repeated calls maintain performance (no resource leaks)
    const ITERATIONS: usize = 10;

    let mut durations = Vec::new();
    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _browsers = detect_browsers();
        durations.push(start.elapsed());
    }

    #[allow(clippy::cast_possible_truncation)]
    let avg_duration = durations.iter().sum::<std::time::Duration>() / ITERATIONS as u32;
    let max_duration = durations.iter().max().unwrap();

    println!("\nRepeated Detection Performance ({ITERATIONS} iterations):");
    println!("  Average: {avg_duration:?}");
    println!("  Maximum: {max_duration:?}");
    println!("  Target: <100ms average");

    // Warn if performance target is exceeded, but don't fail the test
    if avg_duration.as_millis() >= 100 {
        eprintln!(
            "WARNING: Average detection took {}ms, exceeds 100ms target (not failing in non-release builds)",
            avg_duration.as_millis()
        );
    }

    // Only enforce performance in release builds
    #[cfg(not(debug_assertions))]
    assert!(
        avg_duration.as_millis() < 100,
        "Average detection took {}ms, exceeds 100ms target",
        avg_duration.as_millis()
    );
}
