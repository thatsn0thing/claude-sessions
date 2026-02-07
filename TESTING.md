# Testing Documentation

## Test Coverage

The project includes comprehensive unit tests covering all core functionality.

## Test Results

```
running 10 tests
✓ test_list_sessions_empty ... ok
✓ test_manager_creation ... ok  
✓ test_session_creation ... ok
✓ test_session_info_serialization ... ok
✓ test_session_serialization ... ok
✓ test_session_unique_ids ... ok
✓ test_start_session_invalid_dir ... ok
✓ test_start_session_valid_dir ... ok
✓ test_stop_nonexistent_session ... ok
⊙ test_full_session_lifecycle ... ignored

test result: ok. 9 passed; 0 failed; 1 ignored
```

## Running Tests

### Run all tests
```bash
cargo test
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run ignored tests (integration tests)
```bash
cargo test -- --ignored
```

## Test Coverage

### Unit Tests (9 tests)

#### Session Model Tests
- **test_session_creation**: Verifies session metadata is initialized correctly
- **test_session_unique_ids**: Ensures each session gets a unique UUID
- **test_session_serialization**: Tests JSON serialization of sessions
- **test_session_info_serialization**: Tests SessionInfo serialization

#### SessionManager Tests
- **test_manager_creation**: Verifies manager initializes with empty state
- **test_list_sessions_empty**: Tests listing sessions on new manager
- **test_start_session_invalid_dir**: Validates directory existence check
- **test_start_session_valid_dir**: Tests starting session in valid directory
- **test_stop_nonexistent_session**: Ensures error when stopping non-existent session

### Integration Tests (1 test, ignored by default)

- **test_full_session_lifecycle**: Full end-to-end test of start → list → stop
  - Requires `claude` command to be installed
  - Run with: `cargo test -- --ignored`

## Test Strategy

### Phase 1 Testing

**What we test:**
- Session metadata creation
- UUID uniqueness
- Directory validation
- Manager state management
- Error handling
- Serialization/deserialization

**What we don't test (yet):**
- PTY process spawning (requires `claude` command)
- Actual subprocess communication
- Session I/O capture
- Daemon mode (Phase 2)

### Phase 2 Testing Roadmap

Future tests will cover:
1. Daemon startup/shutdown
2. IPC communication (Unix socket)
3. Session persistence to disk
4. Multiple concurrent sessions
5. Session reconnection
6. PTY I/O multiplexing
7. Error recovery

## Test Helpers

### TempDir Helper
```rust
fn create_test_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}
```

Creates temporary directories that are automatically cleaned up after tests.

## Known Limitations

1. **No PTY testing**: Can't test actual Claude subprocess without `claude` binary
2. **No daemon tests**: Phase 2 feature, not implemented yet
3. **No concurrent session tests**: Would require daemon mode
4. **No performance tests**: Not critical for Phase 1 MVP

## Manual Testing Checklist

For features that can't be easily unit tested:

- [ ] Start a session in a real project directory
- [ ] Verify Claude actually launches in PTY
- [ ] Verify session shows in `list`
- [ ] Verify `stop` actually terminates Claude process
- [ ] Test with multiple concurrent sessions
- [ ] Test Ctrl+C handling (graceful shutdown)
- [ ] Test with invalid/missing `claude` binary

## Continuous Integration

Recommended CI pipeline:

```yaml
test:
  script:
    - cargo test
    - cargo test -- --ignored  # Only if claude is installed
    - cargo clippy -- -D warnings
    - cargo fmt -- --check
```

## Test Maintenance

### Adding New Tests

1. Add test function to `src/tests.rs`
2. Use descriptive test names: `test_<feature>_<scenario>`
3. Include assertions with clear failure messages
4. Run `cargo test` to verify

### Test Organization

Tests are organized by module:
- **Session tests**: Test data models and creation
- **Manager tests**: Test business logic and state
- **Integration tests**: Test full workflows (marked `#[ignore]`)

## Debugging Failed Tests

```bash
# Run specific test
cargo test test_session_creation

# Run with debug output
cargo test -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test
```

## Performance

Test suite runs in ~0.01s (9 tests).

All tests are fast because they:
- Don't spawn real processes (except ignored integration test)
- Use in-memory data structures
- Mock external dependencies

---

**Last updated**: Phase 1 MVP  
**Test Status**: ✅ All passing (9/9)
