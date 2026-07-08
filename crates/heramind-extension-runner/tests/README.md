# Extension Runner Tests

Tests for the heramind-extension-runner, which is the isolated process that loads and executes extensions.

## Test Files

| File | Purpose |
|------|---------|
| `runner_ipc_test.rs` | IPC protocol tests for runner-main process communication |

## Key Test Areas

1. **IPC Protocol**
   - Message framing (length prefix)
   - Serialization/deserialization
   - Request/response matching

2. **Extension Loading**
   - Native library loading
   - WASM module loading
   - Error handling

3. **Command Execution**
   - Request routing
   - Response formatting
   - Error propagation

## Running Tests

```bash
# Run all runner tests
cargo test -p heramind-extension-runner

# Run specific test
cargo test -p heramind-extension-runner --test runner_ipc_test
```

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                   HeraMind Main Process                       │
│  - ExtensionService                                           │
│  - Sends IPC messages via stdin                              │
│  - Receives responses via stdout                             │
└──────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│               Extension Runner Process                        │
│  - Loads extension binary                                    │
│  - Routes commands to extension                              │
│  - Returns results via IPC                                   │
└──────────────────────────────────────────────────────────────┘
```

## Dependencies

- Uses `heramind-extension-sdk` for shared IPC types
- Does NOT depend on `heramind-core` (for ABI isolation)
