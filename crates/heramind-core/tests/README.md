# HeraMind Core Test Suite

This directory contains all tests for the heramind-core crate, organized by test type.

## Test Categories

### Unit Tests (Component-level)

| File | Purpose | Coverage |
|------|---------|----------|
| `extension_test.rs` | Basic extension trait tests | Metadata, metrics, commands |
| `message_test.rs` | Message type tests | IPC messages |
| `config_validation_test.rs` | Configuration validation | Config parameters |
| `error_path_test.rs` | Error handling paths | ExtensionError variants |
| `concurrent_limiting.rs` | Concurrency control | Rate limiting |

### Integration Tests (Inter-component)

| File | Purpose | Coverage |
|------|---------|----------|
| `extension_registry_test.rs` | Extension registry operations | Load, unload, lookup |
| `extension_loader_test.rs` | Extension loading | Native/WASM loading |
| `extension_lifecycle_test.rs` | Lifecycle management | Init, start, stop, cleanup |
| `extension_command_test.rs` | Command execution | Execute, validate, respond |
| `extension_context_test.rs` | Extension context | Context creation, access |
| `extension_proxy_test.rs` | Extension proxy pattern | Isolated proxy behavior |
| `extension_event_test.rs` | Event handling | Publish, subscribe, dispatch |
| `extension_capability_test.rs` | Capability system | Permission checks, invoke |
| `eventbus_test.rs` | Event bus operations | Event routing |
| `event_dispatcher_test.rs` | Event dispatching | Multi-subscriber dispatch |
| `session_test.rs` | Session management | Create, update, cleanup |
| `isolated_manager_test.rs` | Process manager | Spawn, monitor, restart |

### E2E Tests (End-to-end)

| File | Purpose | Coverage |
|------|---------|----------|
| `extension_e2e_test.rs` | Full extension workflow | Load → Execute → Unload |
| `capability_e2e_test.rs` | Full capability flow | Request → Process → Response |
| `ipc_e2e_test.rs` | IPC communication | Message passing |
| `ipc_isolated_test.rs` | Isolated process IPC | Runner communication |
| `ipc_business_test.rs` | Business logic IPC | Real-world scenarios |
| `capability_integration_test.rs` | Capability integration | Multi-capability workflows |
| `capability_providers_test.rs` | Provider integration | All capability providers |
| `extension_stream_test.rs` | Streaming functionality | Video/data streams |
| `extension_event_subscription_test.rs` | Event subscription | Subscribe → Handle → Cleanup |
| `isolated_process_test.rs` | Process isolation | Spawn → Communicate → Cleanup |
| `extension_event_subscription_test.rs` | Event subscription lifecycle | Full event workflow |

## Running Tests

```bash
# Run all tests
cargo test -p heramind-core

# Run specific test category
cargo test -p heramind-core --test extension_e2e_test

# Run with verbose output
cargo test -p heramind-core -- --nocapture

# Run specific test
cargo test -p heramind-core test_extension_load
```

## Test Conventions

1. **Naming**: `test_<component>_<scenario>_<expected_result>`
2. **Async tests**: Use `#[tokio::test]` attribute
3. **Fixtures**: Use `test_event_bus/` directory for shared test data
4. **Mocking**: Use trait objects for dependency injection

## Test Dependencies

Tests may depend on:
- `fixtures/smoke-extension/` - For loading real extension binaries
- Test event bus in `test_event_bus/`

## Adding New Tests

1. Choose the appropriate category (unit/integration/e2e)
2. Follow naming conventions
3. Add documentation header explaining test purpose
4. Ensure tests are deterministic and isolated
