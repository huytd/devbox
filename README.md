# DevBox

Create isolated virtual development environments using Docker.

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

## Backend

DevBox uses Docker for containerization.

## Configuration

Each project has a `.devbox/config.json` file that stores:
- Container name (hash-based)
- Volume name
- Absolute path
- Creation timestamp