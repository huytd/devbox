use crate::config::{DevBoxConfig, BackendType};
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

    // Check backend availability
    if !backend.check_available() {
        anyhow::bail!("Backend 'Docker' is not available. Please install Docker.");
    }

    // Check if config exists
    let config = if DevBoxConfig::exists(Path::new(path)) {
        DevBoxConfig::load(Path::new(path))
            .context("Failed to load existing devbox config")?
    } else {
        // Create new config
        let new_config = DevBoxConfig::new(path, backend_type);
        println!("Container: {}", new_config.container_name);
        println!("Volume: {}", new_config.volume_name);
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
        backend.start_container(&config)
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