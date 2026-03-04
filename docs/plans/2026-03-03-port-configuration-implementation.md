# Port Configuration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add CLI flags `-p`/`--port` to `devbox up` command for specifying custom port mappings

**Architecture:** Extend clap argument parsing in main.rs, pass port mappings through commands.rs to backend.rs where docker container creation happens

**Tech Stack:** Rust, clap (CLI parsing), Docker

---

### Task 1: Add port argument to CLI

**Files:**
- Modify: `src/main.rs:16-24`

**Step 1: Update the Up command variant to accept ports**

Replace the simple `Up` variant with one that accepts port arguments:

```rust
#[derive(Subcommand)]
enum Commands {
    /// Create or attach to a devbox environment
    Up {
        #[arg(short = 'p', long = "port", value_name = "HOST:CONTAINER")]
        ports: Vec<String>,
    },
    /// Stop the devbox without removing it
    Down,
    /// Destroy the devbox and all associated resources
    Destroy,
}
```

**Step 2: Update main function to pass ports**

Modify the match statement in `main()` to handle the new `Up` variant signature:

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Up { ports } => commands::up(ports),
        Commands::Down => commands::down(),
        Commands::Destroy => commands::destroy(),
    }
}
```

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "cli: add port argument to Up command"
```

---

### Task 2: Update commands.rs to accept and validate ports

**Files:**
- Modify: `src/commands.rs:5-51`

**Step 1: Update function signature**

Change `pub fn up()` to `pub fn up(ports: Vec<String>) -> Result<()>`

**Step 2: Handle default port behavior**

Add logic to use default port mapping if none specified:

```rust
// Use default port if no ports specified
let port_mappings = if ports.is_empty() {
    vec!["3000:3000".to_string()]
} else {
    ports
};
```

**Step 3: Pass port mappings to backend**

Update the call to `backend.create_container()` to pass the port mappings. This will require modifying the trait first (Task 3).

**Step 4: Commit**

```bash
git add src/commands.rs
git commit -m "commands: pass port mappings to backend"
```

---

### Task 3: Update DevEnvBackend trait to accept ports

**Files:**
- Modify: `src/backend.rs:4-12`

**Step 1: Update trait method signatures**

Modify the trait to accept port mappings:

```rust
pub trait DevEnvBackend: Send + Sync {
    fn check_available(&self) -> bool;
    fn create_container(&self, config: &DevBoxConfig, ports: &[String]) -> Result<()>;
    fn start_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn attach_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn stop_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn container_exists(&self, config: &DevBoxConfig) -> bool;
    fn is_container_running(&self, config: &DevBoxConfig) -> bool;
}
```

**Step 2: Update all implementations**

Update `create_container` call in `commands.rs` to pass port mappings.

**Step 3: Commit**

```bash
git add src/backend.rs
git commit -m "backend: update trait to accept port mappings"
```

---

### Task 4: Implement port mapping logic in DockerBackend

**Files:**
- Modify: `src/backend.rs:51-72`

**Step 1: Build docker args with port mappings**

Replace the hardcoded `-p 3000:3000` with loop through port mappings:

```rust
fn create_container(&self, config: &DevBoxConfig, ports: &[String]) -> Result<()> {
    // Create named volume
    std::process::Command::new("docker")
        .args(["volume", "create", &config.volume_name])
        .output()?;

    // Build docker run args with port mappings
    let mut args = vec![
        "run", "-d",
        "--name", &config.container_name,
        "-v", &format!("{}:/workspaces", config.absolute_path),
        "-w", "/workspaces",
        "ubuntu:latest",
        "tail", "-f", "/dev/null"
    ];

    // Add port mappings (order matters - must come before image name)
    let mut final_args = vec![
        "run", "-d",
        "--name", &config.container_name,
    ];

    for port in ports {
        final_args.push("-p");
        final_args.push(port);
    }

    final_args.extend(vec![
        "-v", &format!("{}:/workspaces", config.absolute_path),
        "-w", "/workspaces",
        "ubuntu:latest",
        "tail", "-f", "/dev/null"
    ]);

    std::process::Command::new("docker")
        .args(&final_args)
        .output()?;

    Ok(())
}
```

**Step 2: Commit**

```bash
git add src/backend.rs
git commit -m "backend: implement port mapping logic for docker"
```

---

### Task 5: Add tests for port configuration

**Files:**
- Create: `src/tests.rs` or add to existing test modules

**Step 1: Test CLI argument parsing**

Add test in `main.rs` or a new test file:

```rust
#[test]
fn test_up_command_with_ports() {
    let args = ["devbox", "up", "-p", "8080:3000", "-p", "5000:5000"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Up { ports } => {
            assert_eq!(ports, vec!["8080:3000", "5000:5000"]);
        }
        _ => panic!("Expected Up variant"),
    }
}

#[test]
fn test_up_command_default_ports() {
    let args = ["devbox", "up"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Up { ports } => {
            assert!(ports.is_empty()); // Empty means use default
        }
        _ => panic!("Expected Up variant"),
    }
}
```

**Step 2: Run tests**

```bash
cargo test
```

**Step 3: Commit**

```bash
git add src/
git commit -m "tests: add port configuration tests"
```

---

### Task 6: Verify implementation

**Step 1: Build the project**

```bash
cargo build --release
```

**Step 2: Test CLI help**

```bash
./target/release/devbox up --help
```

Expected output should show `-p, --port <HOST:CONTAINER>` option.

**Step 3: Commit final changes**

```bash
git add .
git commit -m "feat: add port configuration support"
```

---

## Execution Order

1. Task 1: CLI argument parsing
2. Task 3: Update trait (before Task 2 since commands depends on it)
3. Task 2: Commands logic
4. Task 4: Backend implementation
5. Task 5: Tests
6. Task 6: Verification

## Notes

- Port mappings format: `HOST_PORT:CONTAINER_PORT` (e.g., `8080:3000`)
- Multiple ports supported via repeated `-p` flags
- Default to `3000:3000` if no ports specified for backward compatibility