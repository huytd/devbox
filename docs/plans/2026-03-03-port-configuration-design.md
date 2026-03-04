# Port Configuration Design

## Overview

This design adds the ability to specify custom port mappings when starting a devbox container via CLI flags.

## Motivation

The current implementation hardcodes port mapping `-p 3000:3000`, which limits flexibility for projects that need different ports or multiple ports exposed (e.g., web server, API, debugger).

## Design

### CLI Changes

Extend the `Up` subcommand with optional port flags:

```rust
#[derive(Subcommand)]
enum Commands {
    /// Create or attach to a devbox environment
    Up {
        #[arg(short = 'p', long = "port", value_name = "HOST:CONTAINER")]
        ports: Vec<String>,
    },
}
```

### Usage Examples

```bash
# Single port mapping
devbox up -p 8080:3000

# Multiple ports
devbox up -p 8080:3000 -p 5000:5000

# Web + API + Debugger
devbox up -p 80:3000 -p 9229:9229 -p 5432:5432
```

### Container Creation

The `create_container` function will accept a vector of port mappings and iterate through them, adding `-p` flags for each mapping when running the docker container.

**Before:**
```rust
std::process::Command::new("docker")
    .args([
        "run", "-d",
        "--name", &config.container_name,
        "-p", "3000:3000",  // hardcoded
        ...
    ])
```

**After:**
```rust
let mut args = vec![
    "run", "-d",
    "--name", &config.container_name,
];

// Add port mappings
for port_mapping in &ports {
    args.push("-p");
    args.push(port_mapping);
}

// ... rest of args
```

### Backward Compatibility

When no ports are specified, the default behavior is to use `-p 3000:3000` to maintain backward compatibility with existing usage.

## Trade-offs Considered

1. **CLI flags vs config file**: Chose CLI flags over config files for simplicity and flexibility - users can specify different ports per invocation without managing config state.

2. **Default port behavior**: When no ports specified, default to 3000:3000 rather than exposing no ports. This maintains expected behavior for projects expecting a web server on port 3000.

## Testing Strategy

- Unit tests for CLI argument parsing
- Integration tests for docker command generation with various port configurations
- Verify multiple ports are correctly passed to docker