# DevBox Design Document

## Overview

A CLI tool that creates isolated virtual development environments from the current folder using Docker or Lima (auto-detected). When a devbox container is started, users can interact with it to install dependencies, dev tools, and other software. The container is isolated from the host system but has internet access.

## Requirements

1. **Isolation**: Container must be isolated from host system
2. **Internet Access**: Containers have internet access
3. **Cross-platform**: Works wherever Docker or Lima is available
4. **Persistent State**: Data persists when container stops, restored on next `devbox up`
5. **Multiple Projects**: Can run multiple devboxes simultaneously for different folders

## User Workflow

1. User creates a folder on host machine and `cd`s into it
2. User runs `devbox up`
3. Devbox is created (if new) or started (if existing)
4. User is connected to the container immediately
5. When user exits, container stops but data persists
6. Next time user runs `devbox up` in same folder, they reconnect to the same devbox

## Architecture

### Backend Selection

The tool auto-detects and chooses between:
- **Lima** (if `limactl` is available) - For stronger VM-level isolation
- **Docker** (fallback) - Universal availability

```
devbox up
    ↓
Check for lima/limactl binary
    │
    ├─ Found → Use Lima backend
    └─ Not found → Use Docker backend
```

### Container Setup

- **Container name**: `devbox-{hash-of-absolute-path}` (64-bit hash)
- **Volume**: Named volume `devbox-data-{hash}` mounted to `/workspaces`
- **Image**: Generic base image (e.g., ubuntu:latest or debian:bullseye)
- **Network**: Default Docker network, containers have internet access
- **Working directory**: Mounted from host folder to `/workspaces` inside container

### Metadata Structure (.devbox/config.json)

```json
{
  "container_name": "devbox-abc123",
  "volume_name": "devbox-data-abc123",
  "absolute_path": "/home/user/project",
  "backend": "docker|lima",
  "created_at": "2026-03-01T10:00:00Z"
}
```

### Commands

| Command | Description |
|---------|-------------|
| `devbox up` | Create/start and attach to devbox |
| `devbox down` | Stop container without removing it |
| `devbox destroy` | Stop + remove container and volume |

## Technical Implementation

### Rust Dependencies

- **clap** - CLI argument parsing
- **serde/serde_json** - Config file handling
- **docker-rs** or manual Docker API calls - Container management
- **sha2** - Hash function for container naming

### Backend Trait Interface

```rust
trait DevEnvBackend {
    fn check_available(&self) -> bool;
    fn create_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn attach_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn stop_container(&self, config: &DevBoxConfig) -> Result<()>;
}

struct DockerBackend;
struct LimaBackend;
```

### Workflow Flow

```
devbox up
    ↓
Read .devbox/config.json (if exists)
    ↓
Call Backend API to check container status
    ↓
┌─────────────────────────────────────┐
│ Container Found?                    │
└─────────────────────────────────────┘
    │              │
    yes            no
    │              │
    ↓              ↓
Is running?      Create new container
    │              │
    │              ↓
    yes    ┌──────────────┐
    │      │ Start it     │
    ↓      └──────────────┘
    no      │
    │       ↓
    ↓   Attach to container
Attach via `docker exec -it bash` or `limactl shell docker exec`
    │
    ↓
User exits → Stop container
```

### Error Handling

- **Docker daemon not running** → Clear error message suggesting to start Docker
- **Permission denied** → Suggest adding user to docker group (or lima group)
- **Container already attached elsewhere** → Detach other session first or show error
- **Backend not available** → Fall back to alternative backend if available

## Data Persistence

- Host folder is mounted as volume to container's `/workspaces` directory
- Any installations, dependencies, tools installed inside container persist in container layers
- Container stops on exit but data remains accessible on next `devbox up`

## Security Considerations

- Containers run with no privileged capabilities
- No host filesystem exposure beyond the project folder mount
- Network isolation via Docker/Lima networking
- Hash-based naming prevents collisions and reveals no path information