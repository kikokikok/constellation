# Deadlock Prevention Guide

This document outlines the patterns and best practices for preventing deadlocks in the Constellation hybrid agent system.

## Root Causes of Fixed Deadlocks

### 1. Recursive Mutex Deadlocks
**Problem**: Same thread locking same mutex multiple times
**Example**: `update_with_task_result` → `check_alerts` → `add_alert` (all locking same mutex)
**Solution**: Move logic outside of nested locking contexts

### 2. Self-Deadlocks in ResourceManager
**Problem**: `allocate_resources` → `estimate_performance` → `calculate_resource_utilization` (all locking same mutex)
**Solution**: Pass locked references as parameters instead of re-locking

## Lock Ordering Patterns

### Consistent Lock Ordering
Always acquire locks in this order:
1. `resource_pool`
2. `budget_tracker` 
3. `performance_history`
4. `scaling_controller`

### Reference Passing Pattern
Instead of:
```rust
fn allocate_resources(&self) {
    let pool = self.resource_pool.lock().unwrap();
    let performance = self.estimate_performance(); // Locks again!
}
```

Use:
```rust
fn allocate_resources(&self) {
    let pool = self.resource_pool.lock().unwrap();
    let history = self.performance_history.lock().unwrap();
    let performance = self.estimate_performance(&pool, &history);
}

fn estimate_performance(&self, pool: &ResourcePool, history: &PerformanceHistory) {
    // Use references, don't lock
}
```

## Best Practices

### 1. Avoid Nested Locking
- Never call a method that locks the same mutex from within a locked context
- Extract logic into helper functions that take references

### 2. Use Lock Guards Wisely
- Keep lock guards as short as possible
- Extract data needed for calculations before releasing locks

### 3. Document Lock Dependencies
- Add comments showing which locks are acquired in each method
- Document lock ordering requirements

### 4. Test for Deadlocks
- Run `cargo test -- --test-threads=1` to detect deadlocks
- Use `RUST_BACKTRACE=1` to get stack traces when tests hang

## Fixed Examples

### PerformanceMonitor (crates/constellation-core/src/hybrid/performance_monitor.rs:55)
**Before**: `update_with_task_result` → `check_alerts` → `add_alert` (self-deadlock)
**After**: `update_with_task_result` directly calls `add_alert` without nested locking

### ResourceManager (crates/constellation-core/src/hybrid/resource_manager.rs:189)
**Before**: `allocate_resources` → `estimate_performance` → `calculate_resource_utilization` (recursive deadlock)
**After**: All methods take `&ResourcePool` and `&PerformanceHistory` parameters

## CI Enforcement

The project now includes:
1. `check-ci.sh` - Full CI simulation script
2. `.opencode/plugin/ci-enforcer.js` - Pre-commit hook
3. `.opencode/plugin/ci-reminder.js` - Commit reminder

## Testing Strategy

1. **Unit Tests**: All 71 hybrid tests pass without deadlocks
2. **CI Checks**: Formatting, clippy, compilation, and tests
3. **Concurrency Testing**: Run with `--test-threads=1` to detect deadlocks

## Future Prevention

1. **Code Review**: Check for nested locking patterns
2. **Static Analysis**: Use clippy to detect potential deadlocks
3. **Documentation**: Keep this guide updated with new patterns