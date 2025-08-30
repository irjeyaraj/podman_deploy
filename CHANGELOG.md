# Changelog

All notable changes to the Podman Deploy project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.2.0] - 2025-08-30

### Added
- **List Mode**: New `list` command to display all pods with their containers, status, and images
  - Shows both expected and actual container images
  - Displays real-time pod and container status from Podman
  - Provides comprehensive overview of deployment state

- **Prune Mode**: New `prune` command for image cleanup
  - Uses `podman image prune -a` to remove all unused images
  - Helps maintain clean container environment
  - Optimizes disk space usage

### Improved
- **Code Optimization**
  - Consolidated duplicate container argument building logic into reusable `build_container_args` function
  - Optimized string operations in `generate_pod_command` using format! and push_str
  - Removed unused description parameters from command execution functions
  - Improved memory efficiency by reducing string allocations

- **Upgrade Intelligence**
  - Enhanced image comparison logic with registry prefix normalization
  - Handles docker.io/library/ and docker.io/ prefixes automatically
  - Skips unnecessary upgrades when container images are already up-to-date
  - Improved upgrade detection accuracy

- **Performance Enhancements**
  - Streamlined command generation functions
  - Reduced redundant string operations
  - Optimized container creation workflow
  - More efficient error handling patterns

### Fixed
- Container image inspection now uses correct format template `{{.Config.Image}}`
- Improved image tag comparison for upgrade decisions
- Better handling of registry prefix variations in image names

### Documentation
- Updated README.md with complete mode documentation including new `list` and `prune` commands
- Corrected command syntax examples to reflect actual usage
- Added comprehensive examples for all available modes
- Enhanced feature descriptions and usage guidelines

## [0.1.0] - 2025-08-30

### Added
- **Initial Project Setup**
  - Created Rust project with Cargo.toml configuration
  - Added serde and serde_yaml dependencies for YAML configuration handling

- **Configuration Management**
  - YAML-based configuration file support
  - Application name configuration
  - Pod and container definitions
  - Container image specifications
  - Mount point configurations
  - Environment variable support
  - Port mapping configurations
  - Private registry support with authentication

- **Podman Integration**
  - Automatic Podman installation detection
  - Multi-distribution support (Ubuntu/Debian, Fedora/RedHat, ArchLinux)
  - Automatic Podman installation for supported distributions
  - Config file tracking of Podman installation status

- **Container and Pod Management**
  - Pod creation and management
  - Container creation within pods
  - Container lifecycle management (start, stop, upgrade)
  - Image pulling and management
  - Mount directory/file creation with data path prefixing
  - Port exposure at pod level

- **Command Line Interface**
  - Multiple operation modes: setup, upgrade, start, stop
  - Support for targeting specific pods or containers
  - Custom configuration file path support
  - Comprehensive usage help and error messages

- **Private Registry Support**
  - Registry authentication and login management
  - Automatic login attempt with provided credentials
  - Login status checking

- **Data Management**
  - Configurable data storage path
  - Automatic creation of mount directories and files
  - Intelligent file vs directory detection for mounts

### Features by Mode

#### Setup Mode
- Checks and installs Podman if needed
- Creates data directories and mount paths
- Creates all defined pods
- Pulls all required container images
- Stops containers and pods after setup

#### Upgrade Mode
- Checks container image versions against configuration
- Supports upgrading all containers or specific containers
- Pulls new images when needed
- Recreates containers with updated images

#### Start/Stop Modes
- Start or stop all pods
- Start or stop specific pods
- Proper error handling for non-existent pods

### Technical Improvements
- **Code Optimization**
  - Added helper functions for command execution
  - Improved error handling with custom AppResult type
  - Reduced code duplication in command patterns
  - Optimized string handling and memory usage
  - Added comprehensive inline documentation

- **Error Handling**
  - Consistent error messages across all operations
  - Graceful handling of missing dependencies
  - Informative error output for troubleshooting

- **Configuration Flexibility**
  - Multiple config file search locations
  - Support for custom config file paths
  - Automatic config file location detection

### Documentation
- **README.md**: Comprehensive project documentation including:
  - Feature overview and capabilities
  - Installation and build instructions
  - Complete usage guide with examples
  - Configuration format and parameter documentation
  - Error handling and troubleshooting information

- **Inline Documentation**: Added detailed comments to:
  - Data structures and their fields
  - Helper functions and their purposes
  - Complex logic sections

### Configuration Evolution
- **v1**: Basic application name and pod/container listings
- **v2**: Added mount point configurations
- **v3**: Added environment variable support
- **v4**: Added private registry configuration
- **v5**: Added Podman installation tracking
- **v6**: Added data path configuration
- **v7**: Added explicit image names (removed tags)
- **v8**: Added port mapping support
- **v9**: Added registry authentication details

### Security Considerations
- Registry credentials stored in configuration file
- Sudo access required for Podman installation
- Automatic directory creation with appropriate permissions

### Known Limitations
- Requires Linux operating system
- Limited to supported package managers (apt, dnf, yum, pacman)
- Registry passwords stored in plain text in configuration

### Dependencies
- serde 1.0 (with derive feature)
- serde_yaml 0.9

---

*Note: This project was developed iteratively with multiple feature additions and improvements. Each version built upon the previous functionality while maintaining backward compatibility where possible.*