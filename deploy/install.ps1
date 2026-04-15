#Requires -RunAsAdministrator
# =============================================================================
# DDNS Remake — Installer (Windows)
# Requires: PowerShell 5.1+ and run as Administrator
# =============================================================================
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Constants ─────────────────────────────────────────────────────────────────
$REPO        = "Derrick-Program/DDNS-Remake"
$INSTALL_DIR = "C:\Program Files\DDNS"
$BIN_DIR     = "$INSTALL_DIR\bin"
$CONFIG_DIR  = "$INSTALL_DIR\config"
$DATA_DIR    = "$INSTALL_DIR\data"
$LOG_DIR     = "$INSTALL_DIR\logs"

# ── Color helpers ─────────────────────────────────────────────────────────────
function Write-Info  { param($msg) Write-Host "[INFO]  $msg" -ForegroundColor Cyan }
function Write-Ok    { param($msg) Write-Host "[OK]    $msg" -ForegroundColor Green }
function Write-Warn  { param($msg) Write-Host "[WARN]  $msg" -ForegroundColor Yellow }
function Write-Err   { param($msg) Write-Host "[ERR]   $msg" -ForegroundColor Red }
function Write-Bold  { param($msg) Write-Host $msg -ForegroundColor White }

function Die {
    param($msg)
    Write-Err $msg
    exit 1
}

# ── Version resolution ───────────────────────────────────────────────────────
# Uses DDNS_VERSION env var if set; otherwise fetches latest from GitHub API
# with up to 3 retries and exponential backoff.
function Resolve-Version {
    if ($env:DDNS_VERSION) {
        $script:VERSION = $env:DDNS_VERSION
        Write-Info "Using specified version: $script:VERSION"
    } else {
        Write-Info "Fetching latest release version from GitHub ..."

        $attempts = 3
        $delay    = 2

        for ($i = 1; $i -le $attempts; $i++) {
            try {
                $release = Invoke-RestMethod `
                    -Uri "https://api.github.com/repos/$REPO/releases/latest" `
                    -Headers @{ Accept = "application/vnd.github+json" } `
                    -TimeoutSec 10 `
                    -UseBasicParsing
                $script:VERSION = $release.tag_name
                Write-Ok "Latest version: $script:VERSION"
                break
            } catch {
                $errMsg = $_.Exception.Message
                if ($errMsg -match "rate limit") {
                    Write-Warn "GitHub API rate limit hit."
                }
                if ($i -lt $attempts) {
                    Write-Warn "Attempt $i/$attempts failed, retrying in ${delay}s ... ($errMsg)"
                    Start-Sleep -Seconds $delay
                    $delay *= 2
                } else {
                    Die "Could not fetch latest version after $attempts attempts.`nTip: set DDNS_VERSION to install a specific version, e.g.`n     `$env:DDNS_VERSION='v0.1.1'; .\install.ps1`nError: $errMsg"
                }
            }
        }
    }
    $script:BASE_URL = "https://github.com/$REPO/releases/download/$script:VERSION"
}

# ── Architecture detection ────────────────────────────────────────────────────
function Get-Arch {
    $arch = [System.Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE")
    switch ($arch) {
        "AMD64" { return "x64" }
        "ARM64" { return "arm64" }
        default  { Die "Unsupported Windows architecture: $arch" }
    }
}

# ── Binary availability check ─────────────────────────────────────────────────
function Assert-BinaryAvailable {
    param($binaryName)
    $url = "$script:BASE_URL/$binaryName"
    Write-Info "Checking availability of $binaryName ..."
    try {
        $response = Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing -ErrorAction Stop
        if ($response.StatusCode -ne 200) {
            Die "Binary not available at: $url"
        }
    } catch {
        Die "Binary not available: $url`nCheck available releases at: https://github.com/$REPO/releases"
    }
}

# ── Download binary ───────────────────────────────────────────────────────────
function Download-Binary {
    param($name, $dest)
    $url = "$script:BASE_URL/$name"
    Write-Info "Downloading $name ..."
    try {
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($url, $dest)
    } catch {
        Die "Failed to download $url`n$($_.Exception.Message)"
    }
    Write-Ok "Downloaded → $dest"
}

# ── Directory setup ───────────────────────────────────────────────────────────
function Setup-Directories {
    foreach ($dir in @($BIN_DIR, $CONFIG_DIR, $DATA_DIR, $LOG_DIR)) {
        if (-not (Test-Path $dir)) {
            New-Item -ItemType Directory -Path $dir -Force | Out-Null
        }
    }
    Write-Ok "Directories created under $INSTALL_DIR"
}

# ── Prompt helper ─────────────────────────────────────────────────────────────
function Prompt-Value {
    param($message, $default = "")
    if ($default -ne "") {
        $input = Read-Host "  $message [$default]"
        if ([string]::IsNullOrWhiteSpace($input)) { return $default }
        return $input
    } else {
        do {
            $input = Read-Host "  $message"
        } while ([string]::IsNullOrWhiteSpace($input))
        return $input
    }
}

# ── Random hex generator ──────────────────────────────────────────────────────
function Get-RandomHex {
    param($length = 32)
    $bytes = New-Object byte[] ($length / 2)
    [System.Security.Cryptography.RandomNumberGenerator]::Fill($bytes)
    return ($bytes | ForEach-Object { $_.ToString("x2") }) -join ""
}

# ── Add to system PATH ────────────────────────────────────────────────────────
function Add-ToPath {
    param($dir)
    $currentPath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($currentPath -notlike "*$dir*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$currentPath;$dir", "Machine")
        $env:Path += ";$dir"
        Write-Ok "Added $dir to system PATH"
    }
}

# =============================================================================
# SERVER INSTALLER
# =============================================================================
function Install-Server {
    Write-Bold "`n=== Installing DDNS Server ===`n"

    $arch = Get-Arch
    $binaryName = "ddns-server-windows-$arch.exe"

    Assert-BinaryAvailable $binaryName
    Setup-Directories
    Download-Binary $binaryName "$BIN_DIR\ddns-server.exe"
    Add-ToPath $BIN_DIR

    Write-Bold "`nServer Configuration"
    Write-Host "  (Press Enter to accept defaults)`n"

    $serverHost  = Prompt-Value "Listen host" "127.0.0.1"
    $serverPort  = Prompt-Value "Listen port" "8698"
    $cfApiKey    = Prompt-Value "Cloudflare API key" ""
    $dbPath      = Prompt-Value "Database path (DATABASE_URL)" "sqlite://$DATA_DIR\ddns.db"

    $cfgDir  = "$CONFIG_DIR\duacodie\ddns"
    $cfgFile = "$cfgDir\config.toml"
    New-Item -ItemType Directory -Path $cfgDir -Force | Out-Null

    @"
# DDNS Server Configuration
# Generated by installer on $(Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
# Edit with: ddns-server config set <key> <value>

[server]
host = "$serverHost"
port = $serverPort

[cloudflare]
api_key = "$cfApiKey"

# Add DNS zones below, or use: ddns-server config zone-add <zone>
# [[zones]]
# name = "example.com"
"@ | Set-Content -Encoding UTF8 $cfgFile

    $acl = Get-Acl $cfgFile
    $acl.SetAccessRuleProtection($true, $false)
    $rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
        "SYSTEM", "FullControl", "Allow"
    )
    $acl.AddAccessRule($rule)
    Set-Acl $cfgFile $acl
    Write-Ok "Config written to $cfgFile"

    [System.Environment]::SetEnvironmentVariable("XDG_CONFIG_HOME", $CONFIG_DIR, "Machine")
    [System.Environment]::SetEnvironmentVariable("DATABASE_URL", $dbPath, "Machine")

    _Install-WindowsService `
        -ServiceName   "DuacodieServer" `
        -DisplayName   "Duacodie DDNS Server" `
        -Description   "Duacodie Dynamic DNS Server - https://github.com/$REPO" `
        -BinaryPath    "`"$BIN_DIR\ddns-server.exe`" start"

    Write-Host ""
    Write-Ok "DDNS Server installed successfully!"
    Write-Bold "`nNext steps:"
    Write-Host "  1. Add a DNS zone : ddns-server config zone-add <your-domain>"
    Write-Host "  2. Add a user     : ddns-server server add-user -u <username>"
    Write-Host "  3. Review config  : $cfgFile"
    Write-Host "  4. Check status   : Get-Service DuacodieServer"
    Write-Warn "Note: JWT secret is auto-generated on each start. Tokens will be invalidated on restart."
}

# =============================================================================
# CLIENT INSTALLER
# =============================================================================
function Install-Client {
    Write-Bold "`n=== Installing DDNS Client ===`n"

    $arch = Get-Arch
    $binaryName = "ddns-client-windows-$arch.exe"

    Assert-BinaryAvailable $binaryName
    Setup-Directories
    Download-Binary $binaryName "$BIN_DIR\ddns-client.exe"
    Add-ToPath $BIN_DIR

    Write-Bold "`nClient Configuration"
    Write-Host "  (Press Enter to accept defaults)`n"

    $serverUrl     = Prompt-Value "DDNS server URL"             "http://127.0.0.1:8698"
    $checkInterval = Prompt-Value "IP check interval (seconds)" "60"

    # Write initial config — device_token is obtained via auth login below
    $cfgDir  = "$CONFIG_DIR\duacodie\ddns-client"
    $cfgFile = "$cfgDir\config.toml"
    New-Item -ItemType Directory -Path $cfgDir -Force | Out-Null

    @"
# DDNS Client Configuration
# Generated by installer on $(Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")

server_url = "$serverUrl"
device_token = ""
check_interval_secs = $checkInterval
domains = []
"@ | Set-Content -Encoding UTF8 $cfgFile

    $acl = Get-Acl $cfgFile
    $acl.SetAccessRuleProtection($true, $false)
    $rule = New-Object System.Security.AccessControl.FileSystemAccessRule(
        "SYSTEM", "FullControl", "Allow"
    )
    $acl.AddAccessRule($rule)
    Set-Acl $cfgFile $acl
    Write-Ok "Initial config written to $cfgFile"

    # Set XDG_CONFIG_HOME so auth login writes to the correct path
    [System.Environment]::SetEnvironmentVariable("XDG_CONFIG_HOME", $CONFIG_DIR, "Machine")
    $env:XDG_CONFIG_HOME = $CONFIG_DIR

    # Run auth login interactively — handles login → JWT → device registration → domain selection
    Write-Host ""
    Write-Bold "Device Registration"
    Write-Info "Please log in to register this device with the DDNS server."
    Write-Info "The device token will be saved automatically."
    Write-Host ""
    try {
        & "$BIN_DIR\ddns-client.exe" auth login --server $serverUrl
        Write-Ok "Device registered successfully."
    } catch {
        Write-Warn "Login failed or was skipped: $($_.Exception.Message)"
        Write-Warn "You can register manually later:"
        Write-Warn "  `$env:XDG_CONFIG_HOME='$CONFIG_DIR'; & '$BIN_DIR\ddns-client.exe' auth login"
    }

    _Install-WindowsService `
        -ServiceName   "DuacodieClient" `
        -DisplayName   "Duacodie DDNS Client" `
        -Description   "Duacodie Dynamic DNS Client Daemon - https://github.com/$REPO" `
        -BinaryPath    "`"$BIN_DIR\ddns-client.exe`" run"

    Write-Host ""
    Write-Ok "DDNS Client installed successfully!"
    Write-Bold "`nNext steps:"
    Write-Host "  1. Config     : $cfgFile"
    Write-Host "  2. Re-login   : `$env:XDG_CONFIG_HOME='$CONFIG_DIR'; & '$BIN_DIR\ddns-client.exe' auth login"
    Write-Host "  3. Status     : Get-Service DuacodieClient"
}

# ── Windows Service helper ────────────────────────────────────────────────────
function _Install-WindowsService {
    param(
        [string]$ServiceName,
        [string]$DisplayName,
        [string]$Description,
        [string]$BinaryPath
    )

    $existing = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($existing) {
        Write-Warn "Service '$ServiceName' already exists — removing old version ..."
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
        Start-Sleep -Seconds 2
    }

    Write-Info "Creating Windows Service: $DisplayName ..."
    $result = sc.exe create $ServiceName `
        binPath= $BinaryPath `
        start= auto `
        DisplayName= $DisplayName

    if ($LASTEXITCODE -ne 0) {
        Die "Failed to create service: $result"
    }

    sc.exe description $ServiceName $Description | Out-Null

    sc.exe failure $ServiceName reset= 86400 actions= restart/10000/restart/30000/restart/60000 | Out-Null

    Start-Service -Name $ServiceName
    Write-Ok "Service '$ServiceName' created and started."
}

# =============================================================================
# UNINSTALLER HELPERS
# =============================================================================
function _Remove-Service {
    param([string]$ServiceName)
    $s = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($s) {
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        sc.exe delete $ServiceName | Out-Null
        Write-Ok "Service '$ServiceName' removed."
    }
}

function _Remove-Server {
    Write-Info "Removing DDNS Server ..."
    _Remove-Service "DuacodieServer"
    Remove-Item -Force "$BIN_DIR\ddns-server.exe" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force "$CONFIG_DIR\duacodie\ddns" -ErrorAction SilentlyContinue
    Remove-Item -Force "$LOG_DIR\ddns-server*" -ErrorAction SilentlyContinue
    # Remove server-only env vars
    [System.Environment]::SetEnvironmentVariable("DATABASE_URL", $null, "Machine")
    Write-Ok "DDNS Server removed."
}

function _Remove-Client {
    Write-Info "Removing DDNS Client ..."
    _Remove-Service "DuacodieClient"
    Remove-Item -Force "$BIN_DIR\ddns-client.exe" -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force "$CONFIG_DIR\duacodie\ddns-client" -ErrorAction SilentlyContinue
    Remove-Item -Force "$LOG_DIR\ddns-client*" -ErrorAction SilentlyContinue
    Write-Ok "DDNS Client removed."
}

function _Cleanup-Shared {
    $hasServer = Test-Path "$BIN_DIR\ddns-server.exe"
    $hasClient = Test-Path "$BIN_DIR\ddns-client.exe"

    if (-not $hasServer -and -not $hasClient) {
        # Both gone — remove shared dirs and env vars
        if (Test-Path $INSTALL_DIR) {
            Remove-Item -Recurse -Force $INSTALL_DIR
            Write-Ok "Removed $INSTALL_DIR"
        }
        $currentPath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
        $newPath = ($currentPath -split ";" | Where-Object { $_ -ne $BIN_DIR }) -join ";"
        [System.Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
        Write-Ok "Removed $BIN_DIR from PATH."

        [System.Environment]::SetEnvironmentVariable("XDG_CONFIG_HOME", $null, "Machine")
        Write-Ok "Removed environment variable 'XDG_CONFIG_HOME'."
    } else {
        Write-Info "Shared directories kept (other component still installed)."
    }
}

# =============================================================================
# UNINSTALLER
# =============================================================================
function Uninstall-All {
    Write-Bold "`n=== Uninstall DDNS Remake ===`n"

    Write-Host "  What would you like to remove?"
    Write-Host ""
    Write-Host "  [1] Uninstall DDNS Server"
    Write-Host "  [2] Uninstall DDNS Client"
    Write-Host "  [3] Uninstall Both"
    Write-Host "  [4] Cancel"
    Write-Host ""
    $choice = Read-Host "  Select [1-4]"
    Write-Host ""

    switch ($choice) {
        "1" {
            $confirm = Read-Host "  Remove DDNS Server? [y/N]"
            if ($confirm -notmatch "^[yY]$") { Write-Info "Aborted."; return }
            _Remove-Server
            _Cleanup-Shared
        }
        "2" {
            $confirm = Read-Host "  Remove DDNS Client? [y/N]"
            if ($confirm -notmatch "^[yY]$") { Write-Info "Aborted."; return }
            _Remove-Client
            _Cleanup-Shared
        }
        "3" {
            $confirm = Read-Host "  Remove both Server and Client? [y/N]"
            if ($confirm -notmatch "^[yY]$") { Write-Info "Aborted."; return }
            _Remove-Server
            _Remove-Client
            _Cleanup-Shared
        }
        "4" { Write-Info "Cancelled."; return }
        default { Die "Invalid choice: $choice" }
    }

    Write-Ok "Uninstall complete."
}

# =============================================================================
# MAIN MENU
# =============================================================================
function Show-Menu {
    Resolve-Version

    Clear-Host
    Write-Host ""
    Write-Bold "╔════════════════════════════════════╗"
    Write-Bold "║   DDNS Remake Installer $script:VERSION    ║"
    Write-Bold "╚════════════════════════════════════╝"
    Write-Host ""
    Write-Host "  What would you like to do?"
    Write-Host ""
    Write-Host "  [1] Install DDNS Server"
    Write-Host "  [2] Install DDNS Client"
    Write-Host "  [3] Uninstall DDNS Remake"
    Write-Host "  [4] Exit"
    Write-Host ""
    $choice = Read-Host "  Select [1-4]"
    Write-Host ""

    switch ($choice) {
        "1" { Install-Server }
        "2" { Install-Client }
        "3" { Uninstall-All }
        "4" { Write-Info "Bye!"; exit 0 }
        default { Die "Invalid choice: $choice" }
    }
}

Show-Menu
