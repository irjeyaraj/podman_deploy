use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Helper function to execute podman commands with consistent error handling
fn execute_podman_command(args: &[&str], _description: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let status = Command::new("podman")
        .args(args)
        .status()?;
    
    if status.success() {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Helper function to execute system commands with consistent error handling
fn execute_system_command(cmd: &str, args: &[&str], _description: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let status = Command::new(cmd)
        .args(args)
        .status()?;
    
    if status.success() {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Error type for the application
type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Supported Linux distributions for automatic Podman installation
#[derive(Debug)]
enum OSType {
    Ubuntu,
    Debian,
    Fedora,
    RedHat,
    ArchLinux,
    Unknown,
}

/// Container configuration structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Container {
    /// Container name
    name: String,
    /// Container image (e.g., "nginx:1.21")
    image: String,
    /// Mount points in format "host_path:container_path"
    mounts: Vec<String>,
    /// Environment variables as key-value pairs
    env_vars: HashMap<String, String>,
    /// Port mappings in format "host_port:container_port"
    ports: Vec<String>,
}

/// Pod configuration structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Pod {
    /// Pod name
    name: String,
    /// List of containers in the pod
    containers: Vec<Container>,
}

/// Main application configuration structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Config {
    /// Application name
    application_name: String,
    /// Flag indicating if Podman is installed
    is_podman_installed: bool,
    /// Base directory for data storage
    data_path: String,
    /// List of pods to manage
    pods: Vec<Pod>,
    /// Optional private registry URL
    private_registry: Option<String>,
    /// Optional registry username
    registry_username: Option<String>,
    /// Optional registry password
    registry_password: Option<String>,
}

fn detect_os() -> OSType {
    match fs::read_to_string("/etc/os-release") {
        Ok(content) => {
            let content_lower = content.to_lowercase();
            if content_lower.contains("ubuntu") {
                OSType::Ubuntu
            } else if content_lower.contains("debian") {
                OSType::Debian
            } else if content_lower.contains("fedora") {
                OSType::Fedora
            } else if content_lower.contains("red hat") || content_lower.contains("rhel") || content_lower.contains("centos") {
                OSType::RedHat
            } else if content_lower.contains("arch") {
                OSType::ArchLinux
            } else {
                OSType::Unknown
            }
        }
        Err(_) => OSType::Unknown,
    }
}

fn is_podman_installed() -> bool {
    execute_podman_command(&["--version"], "Check podman version")
        .unwrap_or(false)
}

fn install_podman(os_type: &OSType) -> AppResult<()> {
    println!("Installing podman for {:?}...", os_type);
    
    let success = match os_type {
        OSType::Ubuntu | OSType::Debian => {
            // First update package list
            if !execute_system_command("sudo", &["apt", "update"], "Update package list")? {
                return Err("Failed to update package list".into());
            }
            
            // Then install podman
            execute_system_command("sudo", &["apt", "install", "-y", "podman"], "Install podman")?
        }
        OSType::Fedora => {
            execute_system_command("sudo", &["dnf", "install", "-y", "podman"], "Install podman")?
        }
        OSType::RedHat => {
            execute_system_command("sudo", &["yum", "install", "-y", "podman"], "Install podman")?
        }
        OSType::ArchLinux => {
            execute_system_command("sudo", &["pacman", "-S", "--noconfirm", "podman"], "Install podman")?
        }
        OSType::Unknown => {
            return Err("Unsupported OS for automatic podman installation".into());
        }
    };
    
    if success {
        println!("Podman installed successfully!");
        Ok(())
    } else {
        Err("Failed to install podman".into())
    }
}

fn update_config_podman_status(config_path: &str, mut config: Config) -> AppResult<()> {
    config.is_podman_installed = true;
    let updated_yaml = serde_yaml::to_string(&config)?;
    fs::write(config_path, updated_yaml)?;
    println!("Config file updated: is_podman_installed set to true");
    Ok(())
}

fn check_and_install_podman(config_path: &str, config: &mut Config) -> AppResult<()> {
    if config.is_podman_installed {
        println!("Config indicates podman is installed, skipping installation check.");
        return Ok(());
    }
    
    if is_podman_installed() {
        println!("Podman is already installed.");
        update_config_podman_status(config_path, config.clone())?;
        config.is_podman_installed = true;
        return Ok(());
    }
    
    println!("Podman is not installed. Detecting OS...");
    let os_type = detect_os();
    println!("Detected OS: {:?}", os_type);
    
    install_podman(&os_type)?;
    update_config_podman_status(config_path, config.clone())?;
    config.is_podman_installed = true;
    Ok(())
}

fn load_config(config_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    println!("Loading configuration from {}...", config_path);
    let config_content = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;
    println!("Configuration loaded successfully.");
    Ok(config)
}

fn check_and_create_data_path(data_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking data path: {}", data_path);
    
    if Path::new(data_path).exists() {
        println!("Data path already exists: {}", data_path);
    } else {
        println!("Data path does not exist, creating: {}", data_path);
        fs::create_dir_all(data_path)?;
        println!("Data path created successfully: {}", data_path);
    }
    
    Ok(())
}

fn create_mount_paths(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating mount paths within data directory...");
    
    for pod in &config.pods {
        for container in &pod.containers {
            for mount in &container.mounts {
                // Parse mount string (format: "local_path:container_path")
                if let Some((local_path, _)) = mount.split_once(':') {
                    // Create full path within data directory
                    let full_path = format!("{}{}", config.data_path, local_path);
                    let path = Path::new(&full_path);
                    
                    // Determine if this should be a file or directory
                    // If the path ends with a file extension or contains common file patterns, treat as file
                    let is_file = local_path.contains('.') && 
                        (local_path.ends_with(".conf") || local_path.ends_with(".log") || 
                         local_path.ends_with(".txt") || local_path.ends_with(".json") ||
                         local_path.ends_with(".yaml") || local_path.ends_with(".yml"));
                    
                    if is_file {
                        // Create parent directory first
                        if let Some(parent) = path.parent() {
                            if !parent.exists() {
                                println!("Creating directory for file: {}", parent.display());
                                fs::create_dir_all(parent)?;
                            }
                        }
                        // Create empty file if it doesn't exist
                        if !path.exists() {
                            println!("Creating empty file: {}", full_path);
                            fs::write(&full_path, "")?;
                        } else {
                            println!("File already exists: {}", full_path);
                        }
                    } else {
                        // Create directory
                        if !path.exists() {
                            println!("Creating directory: {}", full_path);
                            fs::create_dir_all(&full_path)?;
                        } else {
                            println!("Directory already exists: {}", full_path);
                        }
                    }
                }
            }
        }
    }
    
    println!("All mount paths created successfully");
    Ok(())
}

fn is_logged_into_registry(registry: &str) -> bool {
    match Command::new("podman")
        .args(&["login", "--get-login", registry])
        .output()
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn login_to_registry(registry: &str, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Logging into private registry: {}", registry);
    
    let status = Command::new("podman")
        .args(&["login", registry, "-u", username, "-p", password])
        .status()?;
    
    if status.success() {
        println!("Successfully logged into registry: {}", registry);
        Ok(())
    } else {
        Err(format!("Failed to login to registry: {}", registry).into())
    }
}

fn configure_private_registry(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(registry) = &config.private_registry {
        println!("Private registry configured: {}", registry);
        
        if let (Some(username), Some(password)) = (&config.registry_username, &config.registry_password) {
            println!("Registry authentication details available for user: {}", username);
            
            // Check if already logged in
            if is_logged_into_registry(registry) {
                println!("Already logged into registry: {}", registry);
            } else {
                println!("Not logged into registry, attempting login...");
                login_to_registry(registry, username, password)?;
            }
        } else {
            println!("Note: Use 'podman login {}' to authenticate with the registry when needed.", registry);
        }
    } else {
        println!("No private registry configured.");
    }
    Ok(())
}

fn pod_exists(pod_name: &str) -> bool {
    match Command::new("podman")
        .args(&["pod", "exists", pod_name])
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn generate_pod_command(pod: &Pod) -> String {
    let mut args = vec!["podman", "pod", "create", "--name", &pod.name];
    let mut port_args = Vec::new();
    
    // Collect all unique ports from containers in the pod
    let mut all_ports = HashSet::new();
    for container in &pod.containers {
        for port in &container.ports {
            all_ports.insert(port.as_str());
        }
    }
    
    // Add port mappings to pod creation
    for port in all_ports {
        port_args.push("-p");
        port_args.push(port);
    }
    
    args.extend(port_args);
    args.join(" ")
}

fn generate_container_command(pod_name: &str, container: &Container, data_path: &str) -> String {
    let mut args = vec![
        "podman".to_string(),
        "run".to_string(), 
        "-d".to_string(), 
        "--pod".to_string(), 
        pod_name.to_string(), 
        "--name".to_string(), 
        container.name.clone()
    ];
    
    // Add environment variables
    for (key, value) in &container.env_vars {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }
    
    // Add mount points with data_path prefix
    for mount in &container.mounts {
        args.push("-v".to_string());
        // Parse mount string and prefix host path with data_path
        if let Some((host_path, container_path)) = mount.split_once(':') {
            let prefixed_mount = format!("{}{}:{}", data_path, host_path, container_path);
            args.push(prefixed_mount);
        } else {
            args.push(mount.clone());
        }
    }
    
    // Use the explicit image name from config
    args.push(container.image.clone());
    
    args.join(" ")
}

fn display_pod_commands(config: &Config) {
    println!("\n=== Commands to Create Pods and Containers ===");
    
    for pod in &config.pods {
        println!("\n--- Pod: {} ---", pod.name);
        println!("Pod creation command:");
        println!("{}", generate_pod_command(pod));
        
        println!("\nContainer creation commands:");
        for container in &pod.containers {
            println!("# Container: {}", container.name);
            println!("{}", generate_container_command(&pod.name, container, &config.data_path));
            println!();
        }
    }
}

fn create_pod(pod: &Pod, data_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating pod: {}", pod.name);
    
    // Display the command that will be executed
    println!("Executing command: {}", generate_pod_command(pod));
    
    // Create the pod first with port mappings
    let mut pod_args = vec!["pod", "create", "--name", &pod.name];
    
    // Collect all unique ports from containers in the pod
    let mut all_ports = std::collections::HashSet::new();
    for container in &pod.containers {
        for port in &container.ports {
            all_ports.insert(port.clone());
        }
    }
    
    // Add port mappings to pod creation args
    let mut port_args = Vec::new();
    for port in all_ports {
        port_args.push("-p".to_string());
        port_args.push(port);
    }
    
    // Convert port args to string refs and combine with pod_args
    let port_refs: Vec<&str> = port_args.iter().map(|s| s.as_str()).collect();
    pod_args.extend(port_refs);
    
    let pod_status = Command::new("podman")
        .args(&pod_args)
        .status()?;
    
    if !pod_status.success() {
        return Err(format!("Failed to create pod: {}", pod.name).into());
    }
    
    println!("Pod '{}' created successfully", pod.name);
    
    // Create containers in the pod
    for container in &pod.containers {
        create_container_in_pod(&pod.name, container, data_path)?;
    }
    
    Ok(())
}

fn create_container_in_pod(pod_name: &str, container: &Container, data_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating container '{}' in pod '{}'", container.name, pod_name);
    
    // Display the command that will be executed
    println!("Executing command: {}", generate_container_command(pod_name, container, data_path));
    
    let mut args = vec![
        "run".to_string(), "-d".to_string(), "--pod".to_string(), pod_name.to_string(), 
        "--name".to_string(), container.name.clone()
    ];
    
    // Add environment variables
    for (key, value) in &container.env_vars {
        args.push("-e".to_string());
        args.push(format!("{}={}", key, value));
    }
    
    // Add mount points with data_path prefix
    for mount in &container.mounts {
        args.push("-v".to_string());
        // Parse mount string and prefix host path with data_path
        if let Some((host_path, container_path)) = mount.split_once(':') {
            let prefixed_mount = format!("{}{}:{}", data_path, host_path, container_path);
            args.push(prefixed_mount);
        } else {
            args.push(mount.clone());
        }
    }
    
    // Use the explicit image name from config
    args.push(container.image.clone());
    
    // Convert to string refs for Command
    let string_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    
    let status = Command::new("podman")
        .args(&string_args)
        .status()?;
    
    if status.success() {
        println!("Container '{}' created successfully in pod '{}'", container.name, pod_name);
        Ok(())
    } else {
        Err(format!("Failed to create container '{}' in pod '{}'", container.name, pod_name).into())
    }
}

fn check_and_create_pods(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking and creating pods...");
    
    for pod in &config.pods {
        if pod_exists(&pod.name) {
            println!("Pod '{}' already exists", pod.name);
        } else {
            println!("Pod '{}' does not exist, creating it...", pod.name);
            create_pod(pod, &config.data_path)?;
        }
    }
    
    println!("All pods checked and created as needed");
    Ok(())
}

fn pull_images(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Pulling all required images...");
    
    for pod in &config.pods {
        for container in &pod.containers {
            println!("Pulling image: {}", container.image);
            
            let status = Command::new("podman")
                .args(&["pull", &container.image])
                .status()?;
            
            if status.success() {
                println!("Successfully pulled image: {}", container.image);
            } else {
                println!("Warning: Failed to pull image: {}", container.image);
            }
        }
    }
    
    println!("Image pulling process completed");
    Ok(())
}

fn start_pod(config: &Config, pod_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Find the pod in config
    let pod = config.pods.iter().find(|p| p.name == pod_name);
    
    match pod {
        Some(_pod_config) => {
            println!("Starting pod: {}", pod_name);
            
            let status = Command::new("podman")
                .args(&["pod", "start", pod_name])
                .status()?;
            
            if status.success() {
                println!("Pod '{}' started successfully", pod_name);
                Ok(())
            } else {
                Err(format!("Failed to start pod: {}", pod_name).into())
            }
        }
        None => {
            Err(format!("Pod '{}' not found in configuration", pod_name).into())
        }
    }
}

fn start_all_pods(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting all pods...");
    
    for pod in &config.pods {
        println!("Starting pod: {}", pod.name);
        
        let status = Command::new("podman")
            .args(&["pod", "start", &pod.name])
            .status()?;
        
        if status.success() {
            println!("Pod '{}' started successfully", pod.name);
        } else {
            println!("Warning: Failed to start pod '{}' (may not exist or already running)", pod.name);
        }
    }
    
    println!("All pods started");
    Ok(())
}

fn stop_pod(config: &Config, pod_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Find the pod in config
    let pod = config.pods.iter().find(|p| p.name == pod_name);
    
    match pod {
        Some(_pod_config) => {
            println!("Stopping pod: {}", pod_name);
            
            let status = Command::new("podman")
                .args(&["pod", "stop", pod_name])
                .status()?;
            
            if status.success() {
                println!("Pod '{}' stopped successfully", pod_name);
                Ok(())
            } else {
                Err(format!("Failed to stop pod: {}", pod_name).into())
            }
        }
        None => {
            Err(format!("Pod '{}' not found in configuration", pod_name).into())
        }
    }
}

fn get_container_current_image(container_name: &str) -> Option<String> {
    match Command::new("podman")
        .args(&["inspect", container_name, "--format", "{{.Image}}"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let image = String::from_utf8_lossy(&output.stdout).trim().to_string();
                Some(image)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn container_exists(container_name: &str) -> bool {
    match Command::new("podman")
        .args(&["container", "exists", container_name])
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn needs_upgrade(container: &Container) -> bool {
    if !container_exists(&container.name) {
        println!("Container '{}' does not exist, no upgrade needed", container.name);
        return false;
    }
    
    match get_container_current_image(&container.name) {
        Some(current_image) => {
            let expected_image = &container.image;
            if current_image == *expected_image {
                println!("Container '{}' is already using the correct image: {}", container.name, current_image);
                false
            } else {
                println!("Container '{}' needs upgrade: current='{}', expected='{}'", 
                    container.name, current_image, expected_image);
                true
            }
        }
        None => {
            println!("Could not determine current image for container '{}', assuming upgrade needed", container.name);
            true
        }
    }
}

fn stop_container(container_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Stopping container: {}", container_name);
    
    let status = Command::new("podman")
        .args(&["stop", container_name])
        .status()?;
    
    if status.success() {
        println!("Container '{}' stopped successfully", container_name);
        Ok(())
    } else {
        Err(format!("Failed to stop container: {}", container_name).into())
    }
}

fn remove_container(container_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing container: {}", container_name);
    
    let status = Command::new("podman")
        .args(&["rm", container_name])
        .status()?;
    
    if status.success() {
        println!("Container '{}' removed successfully", container_name);
        Ok(())
    } else {
        Err(format!("Failed to remove container: {}", container_name).into())
    }
}

fn pull_image(image: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Pulling image: {}", image);
    
    let status = Command::new("podman")
        .args(&["pull", image])
        .status()?;
    
    if status.success() {
        println!("Successfully pulled image: {}", image);
        Ok(())
    } else {
        Err(format!("Failed to pull image: {}", image).into())
    }
}

fn upgrade_container(pod_name: &str, container: &Container, data_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Upgrading container '{}' in pod '{}'", container.name, pod_name);
    
    // Pull the new image first
    pull_image(&container.image)?;
    
    // Stop the existing container
    stop_container(&container.name)?;
    
    // Remove the existing container
    remove_container(&container.name)?;
    
    // Create the container with the new image
    create_container_in_pod(pod_name, container, data_path)?;
    
    println!("Container '{}' upgraded successfully", container.name);
    Ok(())
}

fn stop_containers_and_pods(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    println!("Stopping all containers and pods...");
    
    // Stop all containers first
    for pod in &config.pods {
        for container in &pod.containers {
            println!("Stopping container: {}", container.name);
            
            let status = Command::new("podman")
                .args(&["stop", &container.name])
                .status()?;
            
            if status.success() {
                println!("Container '{}' stopped successfully", container.name);
            } else {
                println!("Warning: Failed to stop container '{}' (may not be running)", container.name);
            }
        }
    }
    
    // Stop all pods
    for pod in &config.pods {
        println!("Stopping pod: {}", pod.name);
        
        let status = Command::new("podman")
            .args(&["pod", "stop", &pod.name])
            .status()?;
        
        if status.success() {
            println!("Pod '{}' stopped successfully", pod.name);
        } else {
            println!("Warning: Failed to stop pod '{}' (may not be running)", pod.name);
        }
    }
    
    println!("All containers and pods stopped");
    Ok(())
}


fn setup_mode(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Running Setup Mode ===");
    
    // Load configuration first
    let mut config = load_config(config_path)?;
    
    // 1. Check for podman installation and install podman
    println!("\nStep 1: Checking Podman installation...");
    check_and_install_podman(config_path, &mut config)?;
    
    // 2. Check for all directories exist, if they don't then create them
    println!("\nStep 2: Checking and creating data path...");
    check_and_create_data_path(&config.data_path)?;
    
    println!("Creating mount paths...");
    create_mount_paths(&config)?;
    
    // Configure private registry if specified
    println!("\nConfiguring private registry...");
    if let Err(e) = configure_private_registry(&config) {
        eprintln!("Warning: Error configuring private registry: {}", e);
    }
    
    // 3. Create the pods
    println!("\nStep 3: Creating pods...");
    display_pod_commands(&config);
    check_and_create_pods(&config)?;
    
    // 4. Pull all images that are required
    println!("\nStep 4: Pulling all required images...");
    pull_images(&config)?;
    
    // 5. Stop the containers and pods
    println!("\nStep 5: Stopping containers and pods...");
    stop_containers_and_pods(&config)?;
    
    println!("\n=== Setup completed successfully ===");
    Ok(())
}


fn upgrade_mode(config_path: &str, container_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Running Upgrade Mode ===");
    
    let config = load_config(config_path)?;
    
    let mut upgraded_any = false;
    
    match container_name {
        Some(target_container) => {
            println!("Upgrading specific container: {}", target_container);
            
            // Find the container across all pods
            let mut found_container = false;
            for pod in &config.pods {
                for container in &pod.containers {
                    if container.name == target_container {
                        found_container = true;
                        println!("\nChecking container '{}' in pod '{}'", container.name, pod.name);
                        
                        if needs_upgrade(container) {
                            upgrade_container(&pod.name, container, &config.data_path)?;
                            upgraded_any = true;
                        }
                        break;
                    }
                }
                if found_container {
                    break;
                }
            }
            
            if !found_container {
                return Err(format!("Container '{}' not found in configuration", target_container).into());
            }
        }
        None => {
            println!("Upgrading all containers...");
            
            for pod in &config.pods {
                println!("\nChecking pod: {}", pod.name);
                
                for container in &pod.containers {
                    if needs_upgrade(container) {
                        upgrade_container(&pod.name, container, &config.data_path)?;
                        upgraded_any = true;
                    }
                }
            }
        }
    }
    
    if !upgraded_any {
        println!("\nNo containers needed upgrading - all are up to date!");
    } else {
        println!("\nUpgrade process completed successfully!");
    }
    
    println!("=== Upgrade completed successfully ===");
    Ok(())
}

fn start_mode(config_path: &str, pod_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Running Start Mode ===");
    
    let config = load_config(config_path)?;
    
    match pod_name {
        Some(name) => {
            println!("Starting specific pod: {}", name);
            start_pod(&config, name)?;
        }
        None => {
            println!("Starting all pods...");
            start_all_pods(&config)?;
        }
    }
    
    println!("=== Start completed successfully ===");
    Ok(())
}

fn stop_mode(config_path: &str, pod_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Running Stop Mode ===");
    
    let config = load_config(config_path)?;
    
    match pod_name {
        Some(name) => {
            println!("Stopping specific pod: {}", name);
            stop_pod(&config, name)?;
        }
        None => {
            println!("Stopping all pods...");
            stop_containers_and_pods(&config)?;
        }
    }
    
    println!("=== Stop completed successfully ===");
    Ok(())
}

fn find_config_file(custom_path: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    // If custom path is provided, use it directly
    if let Some(path) = custom_path {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        } else {
            return Err(format!("Config file not found at specified path: {}", path).into());
        }
    }
    
    // Search in fallback locations
    let search_paths = vec![
        format!("{}/.config/podman_deploy/config.yaml", env::var("HOME").unwrap_or_default()),
        "/etc/podman_deploy/config.yaml".to_string(),
        "./config.yaml".to_string(),
    ];
    
    for path in &search_paths {
        if Path::new(path).exists() {
            println!("Found config file at: {}", path);
            return Ok(path.clone());
        }
    }
    
    Err("Config file not found in any of the search locations".into())
}

fn print_usage() {
    println!("Usage: podman_deploy [--config <path>] <mode> [container_name/pod_name]");
    println!("Options:");
    println!("  --config <path>           - Specify custom config file path");
    println!();
    println!("Modes:");
    println!("  setup                     - Install podman, create directories, create pods, pull images, and stop containers/pods");
    println!("  upgrade                   - Check container image versions and upgrade if needed for all containers");
    println!("  upgrade <container_name>  - Check and upgrade specific container if needed");
    println!("  start                     - Start all pods");
    println!("  start <pod>               - Start specific pod");
    println!("  stop                      - Stop all pods");
    println!("  stop <pod>                - Stop specific pod");
    println!();
    println!("Config file search locations (in order):");
    println!("  1. ~/.config/podman_deploy/config.yaml");
    println!("  2. /etc/podman_deploy/config.yaml");
    println!("  3. ./config.yaml (current directory)");
}

fn main() {
    println!("=== Starting Podman Deployment Application ===");
    
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments for --config option
    let mut custom_config_path: Option<&str> = None;
    let mut mode_arg_index = 1;
    let mut pod_name: Option<&str> = None;
    
    // Check for --config option
    if args.len() >= 3 && args[1] == "--config" {
        custom_config_path = Some(&args[2]);
        mode_arg_index = 3;
    }
    
    // Validate argument count based on whether --config was used
    let min_args = if custom_config_path.is_some() { 4 } else { 2 };
    let max_args = if custom_config_path.is_some() { 5 } else { 3 };
    
    if args.len() < min_args || args.len() > max_args {
        eprintln!("Error: Invalid number of arguments");
        print_usage();
        std::process::exit(1);
    }
    
    // Extract mode and optional pod/container name
    if mode_arg_index >= args.len() {
        eprintln!("Error: Mode not specified");
        print_usage();
        std::process::exit(1);
    }
    
    let mode = &args[mode_arg_index];
    
    // Check for pod/container name parameter
    if args.len() > mode_arg_index + 1 {
        pod_name = Some(&args[mode_arg_index + 1]);
    }
    
    // Find the config file
    let config_path = match find_config_file(custom_config_path) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
    
    let result = match mode.as_str() {
        "setup" => {
            if pod_name.is_some() {
                eprintln!("Error: 'setup' mode does not accept pod name parameter");
                print_usage();
                std::process::exit(1);
            }
            setup_mode(&config_path)
        }
        "upgrade" => upgrade_mode(&config_path, pod_name),
        "start" => start_mode(&config_path, pod_name),
        "stop" => stop_mode(&config_path, pod_name),
        _ => {
            eprintln!("Error: Invalid mode '{}'", mode);
            print_usage();
            std::process::exit(1);
        }
    };
    
    match result {
        Ok(()) => println!("\n=== Application completed successfully ==="),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
