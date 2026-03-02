use crate::config::{DevBoxConfig, BackendType};
use anyhow::Result;

pub trait DevEnvBackend: Send + Sync {
    fn check_available(&self) -> bool;
    fn create_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn start_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn attach_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn stop_container(&self, config: &DevBoxConfig) -> Result<()>;
    fn container_exists(&self, config: &DevBoxConfig) -> bool;
    fn is_container_running(&self, config: &DevBoxConfig) -> bool;
}

pub struct DockerBackend;
pub struct LimaBackend;

impl BackendType {
    pub fn detect() -> Self {
        if check_lima_available() {
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
}

fn check_lima_available() -> bool {
    which::which("limactl").is_ok()
}

impl DevEnvBackend for DockerBackend {
    fn check_available(&self) -> bool {
        which::which("docker").is_ok()
    }

    fn container_exists(&self, config: &DevBoxConfig) -> bool {
        let output = std::process::Command::new("docker")
            .args(["ps", "-aq", "--filter", &format!("name={}", config.container_name)])
            .output()
            .expect("Failed to execute docker command");

        !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    }

    fn is_container_running(&self, config: &DevBoxConfig) -> bool {
        let output = std::process::Command::new("docker")
            .args(["ps", "--filter", &format!("name={}", config.container_name), "--format", "{{.Names}}"])
            .output()
            .expect("Failed to execute docker command");

        String::from_utf8_lossy(&output.stdout).trim() == config.container_name
    }

    fn create_container(&self, config: &DevBoxConfig) -> Result<()> {
        // Create named volume
        std::process::Command::new("docker")
            .args(["volume", "create", &config.volume_name])
            .output()?;

        // Run container in detached mode with volume mount and port exposure
        // Keep container running by using a long-running process
        std::process::Command::new("docker")
            .args([
                "run", "-d",
                "--name", &config.container_name,
                "-p", "3000:3000",
                "-v", &format!("{}:/workspaces", config.absolute_path),
                "-w", "/workspaces",
                "ubuntu:latest",
                "tail", "-f", "/dev/null"
            ])
            .output()?;

        Ok(())
    }

    fn start_container(&self, config: &DevBoxConfig) -> Result<()> {
        // Start existing stopped container
        std::process::Command::new("docker")
            .args(["start", &config.container_name])
            .output()?;

        // Wait for container to be running (up to 3 seconds)
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if self.is_container_running(config) {
                break;
            }
        }

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

impl DevEnvBackend for LimaBackend {
    fn check_available(&self) -> bool {
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
            .args(["shell", "default", "docker", "ps", "--filter", &format!("name={}", config.container_name), "--format", "{{.Names}}"])
            .output()
            .expect("Failed to execute limactl command");

        String::from_utf8_lossy(&output.stdout).trim() == config.container_name
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

    fn start_container(&self, config: &DevBoxConfig) -> Result<()> {
        // Start existing stopped container inside Lima VM
        std::process::Command::new("limactl")
            .args([
                "shell", "default", "docker", "start", &config.container_name
            ])
            .output()?;

        // Wait for container to be running (up to 3 seconds)
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if self.is_container_running(config) {
                break;
            }
        }

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