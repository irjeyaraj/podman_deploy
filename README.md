# Podman Deploy

A Rust-based deployment tool for managing Podman containers and pods with configuration-driven setup.

## Features

- **Automated Podman Installation**: Automatically detects and installs Podman on Ubuntu/Debian, Fedora/RedHat, and ArchLinux systems
- **Configuration-Driven**: Use YAML configuration files to define applications, pods, containers, and their properties
- **Private Registry Support**: Built-in support for private container registries with authentication
- **Pod Management**: Create, start, stop, and manage Podman pods with multiple containers
- **Container Lifecycle**: Full container lifecycle management including upgrades and image pulling
- **Mount Management**: Automatic creation of host mount directories and files
- **Port Mapping**: Configure port exposures at the pod level
- **Flexible Configuration**: Support for custom configuration file locations

## Installation

### Prerequisites

- Rust (latest stable version)
- Linux operating system (Ubuntu/Debian, Fedora/RedHat, or ArchLinux)
- sudo privileges (for Podman installation)

### Building from Source

```bash
git clone <repository-url>
cd podman_deploy
cargo build --release
```

The binary will be available at `target/release/podman_deploy`.

## Usage

### Basic Syntax

```bash
podman_deploy <mode> [container_name/pod_name]
```

### Modes

- `setup`: Install podman, create directories, create pods, pull images, and stop containers/pods
- `list`: List all pods with their containers, status, and images
- `prune`: Prune unused and untagged images
- `upgrade`: Check container image versions and upgrade if needed for all containers
- `upgrade <container_name>`: Check and upgrade specific container if needed
- `start`: Start all pods
- `start <pod>`: Start specific pod
- `stop`: Stop all pods
- `stop <pod>`: Stop specific pod

### Examples

```bash
# Initial setup
podman_deploy setup

# List all pods and containers with status
podman_deploy list

# Prune unused images
podman_deploy prune

# Start all pods
podman_deploy start

# Start specific pod
podman_deploy start web-pod

# Stop all pods
podman_deploy stop

# Stop specific pod
podman_deploy stop database-pod

# Upgrade all containers
podman_deploy upgrade

# Upgrade specific container
podman_deploy upgrade nginx-container
```

## Configuration

### Config File Locations

The application searches for configuration files in the following order:

1. Custom path specified with `--config` option
2. `~/.config/podman_deploy/config.yaml`
3. `/etc/podman_deploy/config.yaml`
4. `./config.yaml` (current directory)

### Configuration Format

```yaml
application_name: "My Podman Application"
is_podman_installed: false
data_path: "./podman-data"
pods:
  - name: "web-pod"
    containers:
      - name: "nginx-container"
        image: "nginx:1.21"
        mounts:
          - "/var/www/html:/usr/share/nginx/html"
          - "/var/log/nginx:/var/log/nginx"
        env_vars:
          NGINX_HOST: "localhost"
          NGINX_PORT: "80"
        ports:
          - "80:80"
          - "443:443"
private_registry: "registry.example.com:5000"
registry_username: "myuser"
registry_password: "mypassword"
```

### Configuration Parameters

- `application_name`: Name of your application
- `is_podman_installed`: Boolean flag indicating if Podman is installed
- `data_path`: Directory where container data will be stored
- `pods`: Array of pod definitions
  - `name`: Pod name
  - `containers`: Array of container definitions
    - `name`: Container name
    - `image`: Container image (e.g., "nginx:1.21")
    - `mounts`: Array of mount strings in format "host_path:container_path"
    - `env_vars`: Key-value pairs of environment variables
    - `ports`: Array of port mappings in format "host_port:container_port"
- `private_registry`: Optional private registry URL
- `registry_username`: Optional registry username
- `registry_password`: Optional registry password

## Data Management

The application automatically creates the data directory structure based on your configuration. All host mount paths are created within the `data_path` directory, ensuring organized data storage.

### Mount Path Creation

- Directories are created for mount paths that don't contain file extensions
- Empty files are created for mount paths that appear to be files (contain extensions like .conf, .log, etc.)
- All paths are prefixed with the `data_path` configuration value

## Private Registry Support

If a private registry is configured, the application will:

1. Check if you're already logged in to the registry
2. Attempt automatic login using provided credentials
3. Display manual login instructions if credentials are not provided

## Error Handling

The application provides comprehensive error handling with descriptive messages for:

- Missing or invalid configuration files
- Podman installation failures
- Pod and container creation issues
- Image pulling problems
- Registry authentication failures

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Support

For issues and questions, please open an issue in the project repository.