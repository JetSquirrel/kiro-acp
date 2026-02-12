# kiro-acp

ACP (Agent Client Protocol) adapter for Kiro CLI, enabling integration with ACP-compatible clients like Zed Editor.

## Overview

This project implements an ACP adapter that bridges Kiro CLI with ACP-compatible clients. It allows editors and IDEs that support the ACP protocol to use Kiro as an AI coding assistant.

## Architecture

```
┌─────────────┐     JSON-RPC/stdin/stdout     ┌─────────────────┐
│  ACP Client │ ◄──────────────────────────► │  ACP Adapter    │
│ (e.g., Zed) │                               │  (kiro-acp)     │
└─────────────┘                               └────────┬────────┘
                                                       │
                                                       ▼
                                              ┌─────────────────┐
                                              │   Kiro CLI      │
                                              │   (subprocess)  │
                                              └─────────────────┘
```

## Features

- ACP protocol implementation for Kiro CLI
- JSON-RPC 2.0 communication over stdin/stdout
- Session management
- Async I/O using Tokio
- Process management for Kiro CLI subprocess

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/kiro-acp`.

## Configuration

### Environment Variables

- `KIRO_PATH`: Path to the Kiro CLI executable (optional)

The adapter will automatically search for Kiro CLI in:
1. `KIRO_PATH` environment variable
2. System PATH
3. `~/.kiro/bin/kiro`
4. `~/.local/bin/kiro`
5. `/usr/local/bin/kiro`

### Zed Editor Integration

Add to your Zed settings (`settings.json`):

```json
{
  "agent": {
    "external_agents": [
      {
        "name": "Kiro",
        "command": "/path/to/kiro-acp",
        "args": []
      }
    ]
  }
}
```

## Development

### Project Structure

```
src/
├── main.rs           # Entry point
├── agent.rs          # ACP Agent implementation
├── protocol/         # ACP protocol definitions
│   ├── mod.rs        # Connection handling
│   ├── messages.rs   # JSON-RPC message types
│   └── session.rs    # Session types
├── bridge/           # Kiro CLI bridge
│   ├── mod.rs        # Bridge interface
│   ├── process.rs    # Process management
│   └── parser.rs     # Output parsing
└── utils.rs          # Utility functions
```

### Dependencies

- `tokio`: Async runtime
- `serde` / `serde_json`: JSON serialization
- `anyhow` / `thiserror`: Error handling
- `tracing`: Logging
- `uuid`: Session ID generation
- `which`: Executable lookup
- `dirs`: Directory paths
- `nix` (Unix only): Signal handling

## License

MIT

## References

- [ACP Protocol Specification](https://github.com/zed-industries/codex/tree/acp)
- [Kiro CLI Documentation](https://kiro.dev/docs/cli/)
