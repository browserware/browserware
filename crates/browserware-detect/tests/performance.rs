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

    // Assert performance target
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

    // Default browser detection should be even faster (typically <50ms)
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

    assert!(
        avg_duration.as_millis() < 100,
        "Average detection took {}ms, exceeds 100ms target",
        avg_duration.as_millis()
    );
}
