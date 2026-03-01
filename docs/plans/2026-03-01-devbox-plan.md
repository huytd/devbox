# DevBox CLI Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a CLI tool that creates isolated virtual development environments using Docker or Lima (auto-detected).

**Architecture:** Each project folder gets its own container with persistent state. The CLI auto-selects backend (Lima if available, Docker otherwise), manages container lifecycle, and provides attach/detach functionality.

**Tech Stack:** Rust, clap (CLI parsing), serde/serde_json (config handling), docker-rs (container management), sha2 (hashing)

---

## Task 1: Project Setup & Dependencies

**Files:**
- Modify: `Cargo.toml` - Add dependencies
- Modify: `src/main.rs` - Entry point structure

**Step 1: Update Cargo.toml with required dependencies**

```toml
[package]
name = "devbox"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
tokio = { version = "1.35", features = ["full"] }
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.10"
```

**Step 2: Run `cargo check` to verify dependencies compile**

Run: `cargo check`
Expected: No errors, all dependencies resolved

**Step 3: Create basic CLI structure in main.rs**

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "devbox")]
#[command(about = "Create and manage isolated dev environments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create or attach to a devbox environment
    Up,
    /// Stop the devbox without removing it
    Down,
    /// Destroy the devbox and all associated resources
    Destroy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Up => up(),
        Commands::Down => down(),
        Commands::Destroy => destroy(),
    }
}
```

**Step 4: Run `cargo run -- --help` to verify CLI structure**

Run: `cargo run -- --help`
Expected: Shows help with up, down, destroy subcommands

**Step 5: Commit initial setup**

```bash
git add Cargo.toml src/main.rs
git commit -m "feat: add project dependencies and CLI structure"
```

---

## Task 2: Config File Handling

**Files:**
- Create: `src/config.rs` - Config module
- Modify: `src/lib.rs` or `src/main.rs` - Module structure

**Step 1: Write config structs and file operations in src/config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevBoxConfig {
    pub container_name: String,
    pub volume_name: String,
    pub absolute_path: String,
    pub backend: BackendType,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackendType {
    Docker,
    Lima,
}

impl DevBoxConfig {
    pub fn new(path: &str, backend: BackendType) -> Self {
        let hash = compute_hash(path);
        Self {
            container_name: format!("devbox-{}", hash),
            volume_name: format!("devbox-data-{}", hash),
            absolute_path: path.to_string(),
            backend,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let config_path = path.join(".devbox").join("config.json");
        let content = fs::read_to_string(&config_path)
            .context(format!("Failed to read config at {:?}", config_path))?;
        serde_json::from_str(&content)
            .context("Failed to parse config JSON")
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let devbox_dir = path.join(".devbox");
        fs::create_dir_all(&devbox_dir)?;
        let config_path = devbox_dir.join("config.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn exists(path: &Path) -> bool {
        path.join(".devbox").join("config.json").exists()
    }

    fn compute_hash(path: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)[..8].to_string()
    }
}
```

**Step 2: Add chrono dependency to Cargo.toml**

Edit `Cargo.toml`: Add `chrono = "0.4"` to dependencies

**Step 3: Run `cargo check` to verify config module compiles**

Run: `cargo check`
Expected: No errors

**Step 4: Write tests for config module in src/config.rs**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_new() {
        let config = DevBoxConfig::new("/tmp/test", BackendType::Docker);
        assert!(config.container_name.starts_with("devbox-"));
        assert!(config.volume_name.starts_with("devbox-data-"));
        assert_eq!(config.backend, BackendType::Docker);
    }

    #[test]
    fn test_config_save_and_load() {
        let dir = tempdir().unwrap();
        let config = DevBoxConfig::new(dir.path().to_str().unwrap(), BackendType::Docker);
        config.save(dir.path()).unwrap();

        let loaded = DevBoxConfig::load(dir.path()).unwrap();
        assert_eq!(loaded.container_name, config.container_name);
        assert_eq!(loaded.absolute_path, config.absolute_path);
    }

    #[test]
    fn test_config_exists() {
        let dir = tempdir().unwrap();
        assert!(!DevBoxConfig::exists(dir.path()));

        let config = DevBoxConfig::new(dir.path().to_str().unwrap(), BackendType::Docker);
        config.save(dir.path()).unwrap();
        assert!(DevBoxConfig::exists(dir.path()));
    }

    #[test]
    fn test_hash_consistency() {
        let hash1 = DevBoxConfig::compute_hash("/tmp/test");
        let hash2 = DevBoxConfig::compute_hash("/tmp/test");
        assert_eq!(hash1, hash2);

        let hash3 = DevBoxConfig::compute_hash("/tmp/different");
        assert_ne!(hash1, hash3);
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test config`
Expected: All 4 tests pass

**Step 6: Commit config module**

```bash
git add src/config.rs Cargo.toml
git commit -m "feat: add config file handling with serde and sha2"
```

---

## Task 3: Backend Detection & Trait Implementation

**Files:**
- Create: `src/backend.rs` - Backend trait and implementations
- Modify: `src/lib.rs` or `src/main.rs` - Module structure

**Step 1: Write backend trait in src/backend.rs**

```rust
use crate::config::{DevBoxConfig, BackendType};
use anyhow::Result;

pub trait DevEnvBackend: Send + Sync {
    fn check_available(&self) -> bool;
    fn create_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn attach_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn stop_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn container_exists(&self, config: &DevBoxConfig) -> bool;
    fn is_container_running(&self, config: &DevBoxConfig) -> bool;
}

pub struct DockerBackend;
pub struct LimaBackend;

impl BackendType {
    pub fn detect() -> Self {
        if LimaBackend.check_available_impl() {
            BackendType::Lima
        } else {
            BackendType::Docker
        }
    }

    pub fn create_backend(&self) -> Box<dyn DevEnvBackend> {
        match self {
            BackendType::Docker => Box::new(DockerBackend),
            BackendType::Lima => Box::new(LimaBackend),
        }
    }

    fn check_available_impl() -> bool {
        which::which("limactl").is_ok()
    }
}
```

**Step 2: Add `which` crate to Cargo.toml**

Edit `Cargo.toml`: Add `which = "5.0"` to dependencies

**Step 3: Implement DockerBackend methods**

```rust
impl DevEnvBackend for DockerBackend {
    fn check_available(&self) -> bool {
        which::which("docker").is_ok()
    }

    fn container_exists(&self, config: &DevBoxConfig) -> bool {
        // Check if container exists using docker ps -a
        let output = std::process::Command::new("docker")
            .args(["ps", "-aq", "--filter", &format!("name={}", config.container_name)])
            .output()
            .expect("Failed to execute docker command");

        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    }

    fn is_container_running(&self, config: &DevBoxConfig) -> bool {
        let output = std::process::Command::new("docker")
            .args(["ps", "--filter", &format!("name={}", config.container_name)])
            .output()
            .expect("Failed to execute docker command");

        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    }

    fn create_container(&self, config: &DevBoxConfig) -> Result<()> {
        // Create named volume
        std::process::Command::new("docker")
            .args(["volume", "create", &config.volume_name])
            .output()?;

        // Run container in detached mode with volume mount
        std::process::Command::new("docker")
            .args([
                "run", "-d",
                "--name", &config.container_name,
                "-v", &format!("{}:/workspaces", config.volume_name),
                "-w", "/workspaces",
                "ubuntu:latest",
                "bash", "--login"
            ])
            .output()?;

        Ok(())
    }

    fn attach_container(&self, config: &DevBoxConfig) -> Result<()> {
        // Attach to running container with interactive bash
        let output = std::process::Command::new("docker")
            .args([
                "exec", "-it",
                &config.container_name,
                "bash"
            ])
            .status()?;

        if !output.success() {
            anyhow::bail!("Container attachment failed");
        }

        Ok(())
    }

    fn stop_container(&self, config: &DevBoxConfig) -> Result<()> {
        let output = std::process::Command::new("docker")
            .args(["stop", &config.container_name])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Container stop failed");
        }

        Ok(())
    }
}
```

**Step 4: Implement LimaBackend methods**

```rust
impl DevEnvBackend for LimaBackend {
    fn check_available(&self) -> bool {
        which::which("limactl").is_ok()
    }

    fn check_available_impl() -> bool {
        which::which("limactl").is_ok()
    }

    fn container_exists(&self, config: &DevBoxConfig) -> bool {
        // Lima uses docker inside the VM
        let output = std::process::Command::new("limactl")
            .args(["shell", "default", "docker", "ps", "-aq", "--filter", &format!("name={}", config.container_name)])
            .output()
            .expect("Failed to execute limactl command");

        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    }

    fn is_container_running(&self, config: &DevBoxConfig) -> bool {
        let output = std::process::Command::new("limactl")
            .args(["shell", "default", "docker", "ps", "--filter", &format!("name={}", config.container_name)])
            .output()
            .expect("Failed to execute limactl command");

        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    }

    fn create_container(&self, config: &DevBoxConfig) -> Result<()> {
        // Create volume inside Lima VM
        std::process::Command::new("limactl")
            .args([
                "shell", "default", "docker", "volume", "create", &config.volume_name
            ])
            .output()?;

        // Run container in detached mode
        std::process::Command::new("limactl")
            .args([
                "shell", "default", "docker", "run", "-d",
                "--name", &config.container_name,
                "-v", &format!("{}:/workspaces", config.volume_name),
                "-w", "/workspaces",
                "ubuntu:latest",
                "bash", "--login"
            ])
            .output()?;

        Ok(())
    }

    fn attach_container(&self, config: &DevBoxConfig) -> Result<()> {
        let output = std::process::Command::new("limactl")
            .args([
                "shell", "default", "docker", "exec", "-it",
                &config.container_name,
                "bash"
            ])
            .status()?;

        if !output.success() {
            anyhow::bail!("Container attachment failed");
        }

        Ok(())
    }

    fn stop_container(&self, config: &DevBoxConfig) -> Result<()> {
        let output = std::process::Command::new("limactl")
            .args([
                "shell", "default", "docker", "stop", &config.container_name
            ])
            .output()?;

        if !output.status.success() {
            anyhow::bail!("Container stop failed");
        }

        Ok(())
    }
}
```

**Step 5: Add module declaration to lib.rs or main.rs**

Edit `src/main.rs`: Add `mod config; mod backend;` at the top

**Step 6: Run `cargo check` to verify backend module compiles**

Run: `cargo check`
Expected: No errors

**Step 7: Write tests for backend detection**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_detect() {
        let backend = BackendType::detect();
        // Should be either Docker or Lima based on system
        match backend {
            BackendType::Docker => assert!(DockerBackend.check_available()),
            BackendType::Lima => assert!(LimaBackend.check_available()),
        }
    }

    #[test]
    fn test_backend_creation() {
        let backend_type = BackendType::detect();
        let backend = backend_type.create_backend();

        assert!(backend.check_available());
    }
}
```

**Step 8: Run tests to verify they pass**

Run: `cargo test backend`
Expected: Tests pass (may vary based on system availability)

**Step 9: Commit backend module**

```bash
git add src/backend.rs
git commit -m "feat: add backend trait with Docker and Lima implementations"
```

---

## Task 4: Command Implementations (up, down, destroy)

**Files:**
- Create: `src/commands.rs` - Command implementations
- Modify: `src/main.rs` - Connect commands to CLI

**Step 1: Write command implementations in src/commands.rs**

```rust
use crate::config::{DevBoxConfig, BackendType};
use crate::backend::DevEnvBackend;
use std::path::Path;
use anyhow::{Result, Context};

pub fn up() -> Result<()> {
    let cwd = std::env::current_dir()
        .context("Failed to get current directory")?;

    let path = cwd.to_str()
        .context("Current directory path is not valid UTF-8")?;

    // Detect backend
    let backend_type = BackendType::detect();
    let backend = backend_type.create_backend();

    // Check if config exists
    let config = if DevBoxConfig::exists(Path::new(path)) {
        DevBoxConfig::load(Path::new(path))
            .context("Failed to load existing devbox config")?
    } else {
        // Create new config
        let new_config = DevBoxConfig::new(path, backend_type);
        new_config.save(Path::new(path))
            .context("Failed to save new devbox config")?;
        new_config
    };

    // Check if container exists
    if !backend.container_exists(&config) {
        println!("Creating new devbox for: {}", path);
        backend.create_container(&config)
            .context("Failed to create container")?;
    } else if !backend.is_container_running(&config) {
        println!("Starting existing devbox...");
        backend.stop_container(&config).ok(); // Ensure clean state first
        backend.create_container(&config)
            .context("Failed to start container")?;
    }

    // Attach to container
    println!("Attaching to devbox...");
    backend.attach_container(&config)
        .context("Failed to attach to container")?;

    Ok(())
}

pub fn down() -> Result<()> {
    let cwd = std::env::current_dir()
        .context("Failed to get current directory")?;

    let path = cwd.to_str()
        .context("Current directory path is not valid UTF-8")?;

    if !DevBoxConfig::exists(Path::new(path)) {
        anyhow::bail!("No devbox config found in this directory");
    }

    let config = DevBoxConfig::load(Path::new(path))
        .context("Failed to load devbox config")?;

    let backend_type = BackendType::detect();
    let backend = backend_type.create_backend();

    if !backend.container_exists(&config) {
        anyhow::bail!("No container found for this devbox");
    }

    println!("Stopping devbox...");
    backend.stop_container(&config)
        .context("Failed to stop container")?;

    println!("Devbox stopped.");
    Ok(())
}

pub fn destroy() -> Result<()> {
    let cwd = std::env::current_dir()
        .context("Failed to get current directory")?;

    let path = cwd.to_str()
        .context("Current directory path is not valid UTF-8")?;

    if !DevBoxConfig::exists(Path::new(path)) {
        anyhow::bail!("No devbox config found in this directory");
    }

    let config = DevBoxConfig::load(Path::new(path))
        .context("Failed to load devbox config")?;

    let backend_type = BackendType::detect();
    let backend = backend_type.create_backend();

    // Stop container if running
    if backend.container_exists(&config) {
        if backend.is_container_running(&config) {
            println!("Stopping devbox...");
            backend.stop_container(&config).ok();
        }

        // Remove container
        std::process::Command::new("docker")
            .args(["rm", "-f", &config.container_name])
            .output().ok();

        // Remove volume
        std::process::Command::new("docker")
            .args(["volume", "rm", "-f", &config.volume_name])
            .output().ok();
    }

    // Remove config
    let devbox_dir = Path::new(path).join(".devbox");
    if devbox_dir.exists() {
        std::fs::remove_dir_all(&devbox_dir)?;
    }

    println!("Devbox destroyed.");
    Ok(())
}
```

**Step 2: Update main.rs to use commands module**

Edit `src/main.rs`: Replace the placeholder functions with imports from commands module

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;
mod config;
mod backend;
mod commands;

#[derive(Parser)]
#[command(name = "devbox")]
#[command(about = "Create and manage isolated dev environments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create or attach to a devbox environment
    Up,
    /// Stop the devbox without removing it
    Down,
    /// Destroy the devbox and all associated resources
    Destroy,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Up => commands::up(),
        Commands::Down => commands::down(),
        Commands::Destroy => commands::destroy(),
    }
}
```

**Step 3: Run `cargo check` to verify all modules compile together**

Run: `cargo check`
Expected: No errors

**Step 4: Test CLI help output**

Run: `cargo run -- --help`
Expected: Shows devbox with up, down, destroy subcommands

**Step 5: Run `cargo test` to verify all tests pass**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit command implementations**

```bash
git add src/commands.rs src/main.rs
git commit -m "feat: implement up, down, destroy commands"
```

---

## Task 5: Error Handling & User Experience Improvements

**Files:**
- Modify: `src/config.rs` - Better error messages
- Modify: `src/backend.rs` - Better error handling
- Modify: `src/commands.rs` - User-friendly output

**Step 1: Add better error handling in commands.rs**

```rust
pub fn up() -> Result<()> {
    // ... existing code ...

    // Check backend availability
    if !backend.check_available() {
        anyhow::bail!(
            "Backend '{}' is not available. Please install Docker or Lima.",
            match backend_type {
                BackendType::Docker => "Docker",
                BackendType::Lima => "Lima",
            }
        );
    }

    // ... rest of code ...
}
```

**Step 2: Add permission error handling**

In `src/backend.rs`, add helper for Docker permission errors:

```rust
fn handle_docker_permission_error() -> anyhow::Error {
    anyhow::anyhow!(
        "Docker permission denied. Try adding your user to the docker group:\n\
         sudo usermod -aG docker $USER\n\
         Then log out and log back in."
    )
}
```

**Step 3: Add progress output**

In `src/commands.rs`, add more informative messages:

```rust
println!("Devbox backend: {}", match backend_type {
    BackendType::Docker => "Docker",
    BackendType::Lima => "Lima",
});
println!("Container: {}", config.container_name);
println!("Volume: {}", config.volume_name);
```

**Step 4: Test error handling**

Run: `cargo run -- up` in a directory without Docker/Lima
Expected: Clear error message about missing backend

**Step 5: Commit improvements**

```bash
git add src/commands.rs src/backend.rs
git commit -m "feat: improve error handling and user experience"
```

---

## Task 6: Documentation & Final Polish

**Files:**
- Create: `README.md` - Project documentation
- Modify: `src/main.rs` - Add better help text

**Step 1: Write README.md**

```markdown
# DevBox

Create isolated virtual development environments using Docker or Lima.

## Installation

Build from source:

```bash
cargo build --release
```

Install to cargo bin:

```bash
cargo install --path .
```

## Usage

1. Create a project folder and cd into it:
   ```bash
   mkdir myproject && cd myproject
   ```

2. Start your devbox:
   ```bash
   devbox up
   ```

3. Install dependencies, tools, etc. inside the container.

4. Exit when done:
   ```bash
   exit
   ```

5. Next time you run `devbox up`, you'll reconnect to the same environment.

## Commands

- `devbox up` - Create or attach to devbox
- `devbox down` - Stop devbox (data persists)
- `devbox destroy` - Remove devbox completely

## Backend Detection

DevBox automatically detects and uses:
- **Lima** if `limactl` is available (macOS with stronger isolation)
- **Docker** otherwise (universal availability)

## Configuration

Each project has a `.devbox/config.json` file that stores:
- Container name (hash-based)
- Volume name
- Absolute path
- Backend type
- Creation timestamp
```

**Step 2: Add better help text to CLI**

Edit `src/main.rs`: Update command descriptions

```rust
#[derive(Subcommand)]
enum Commands {
    /// Create or attach to a devbox environment
    /// Automatically detects Docker or Lima backend
    Up,
    /// Stop the devbox without removing data
    Down,
    /// Destroy the devbox and all associated resources
    Destroy,
}
```

**Step 3: Run final tests**

Run: `cargo test`
Expected: All tests pass

Run: `cargo check --release`
Expected: No warnings or errors

**Step 4: Build release binary**

Run: `cargo build --release`
Expected: Binary at `target/release/devbox`

**Step 5: Final commit**

```bash
git add README.md
git commit -m "docs: add README and final polish"
```

---

**Plan complete.** All tasks are now defined with specific file paths, code snippets, commands to run, and expected outputs. Each task follows TDD principles (write test → verify fail → implement → verify pass → commit).