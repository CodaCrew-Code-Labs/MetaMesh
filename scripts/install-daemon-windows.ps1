# MetaMesh Daemon Installation Script for Windows
param(
    [string]$InstallDir = "C:\Program Files\MetaMesh",
    [int]$Port = 50051
)

$ErrorActionPreference = "Stop"

# Build the daemon
Write-Host "Building MetaMesh daemon..."
cargo build --release --bin metamesh-daemon

# Create install directory
Write-Host "Creating installation directory..."
New-Item -ItemType Directory -Force -Path $InstallDir
New-Item -ItemType Directory -Force -Path "$InstallDir\logs"

# Copy binary
Write-Host "Installing daemon binary..."
Copy-Item "target\release\metamesh-daemon.exe" "$InstallDir\metamesh-daemon.exe"

# Create Windows service
Write-Host "Creating Windows service..."
$serviceName = "MetaMeshDaemon"
$binaryPath = "$InstallDir\metamesh-daemon.exe --port $Port"

# Remove existing service if it exists
$existingService = Get-Service -Name $serviceName -ErrorAction SilentlyContinue
if ($existingService) {
    Stop-Service -Name $serviceName -Force
    sc.exe delete $serviceName
}

# Create new service
sc.exe create $serviceName binPath= $binaryPath start= auto
sc.exe description $serviceName "MetaMesh Daemon Service for address creation and key recovery"

# Start the service
Start-Service -Name $serviceName

Write-Host "Installation complete!"
Write-Host "Service '$serviceName' is now running on port $Port"
Write-Host "Check status with: Get-Service -Name $serviceName"
Write-Host "View logs at: $InstallDir\logs\"