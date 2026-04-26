# claude-code-tool (cct) installer for Windows
# Usage: irm https://raw.githubusercontent.com/punisher1/claude-code-tool/main/install.ps1 | iex
#        .\install.ps1 -Version 0.1.5 -Proxy http://127.0.0.1:11224

#Requires -Version 5.1

# TLS 1.2 兼容
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# 参数
[CmdletBinding()]
param(
    [string]$Version = "",
    [string]$InstallDir = "",
    [string]$Proxy = "",
    [switch]$SkipChecksum,
    [switch]$Help
)

# ── 常量 ──

$Repo = "punisher1/claude-code-tool"
$BinaryName = "cct.exe"
$GitHubApiBase = "https://api.github.com/repos/$Repo/releases"
$DefaultInstallDir = Join-Path $env:USERPROFILE ".cct\bin"
$ArtifactName = "cct-Windows-x86_64.zip"

# ── 输出函数 ──

function Write-Info($message) {
    Write-Host -ForegroundColor Green "[INFO]  $message"
}

function Write-Warn($message) {
    Write-Host -ForegroundColor Yellow "[WARN]  $message"
}

function Write-Err($message) {
    Write-Host -ForegroundColor Red "[ERROR] $message"
    exit 1
}

function Write-Banner {
    Write-Host "============================================"
    Write-Host "  claude-code-tool (cct) Installer"
    Write-Host "============================================"
    Write-Host ""
}

# ── 帮助 ──

function Show-Help {
    $helpText = @(
        "Usage: install.ps1 [options]"
        ""
        "Options:"
        "  -Version STRING      Install a specific version (e.g., 0.1.5)"
        "  -InstallDir STRING   Override installation directory"
        "  -Proxy STRING        Use HTTP proxy for downloads"
        "  -SkipChecksum        Skip SHA256 checksum verification"
        "  -Help                Show this help message"
        ""
        "Examples:"
        "  .\install.ps1"
        "  .\install.ps1 -Version 0.1.5"
        "  .\install.ps1 -Proxy http://127.0.0.1:11224"
    )
    $helpText | ForEach-Object { Write-Host $_ }
    exit 0
}

# ── 版本解析 ──

function Resolve-CctVersion {
    if ($Version -ne "") {
        $script:VersionTag = "v$(($Version -replace '^v',''))"
        $apiUrl = "$GitHubApiBase/tags/$script:VersionTag"
    } else {
        $apiUrl = "$GitHubApiBase/latest"
    }

    Write-Info "Querying GitHub API for release information..."

    $headers = @{ "User-Agent" = "cct-installer" }

    try {
        $proxyParam = @{}
        if ($Proxy -ne "") {
            $proxyParam = @{ Proxy = $Proxy }
        }
        $response = Invoke-RestMethod -Uri $apiUrl -Headers $headers @proxyParam -ErrorAction Stop
    } catch {
        $statusCode = 0
        if ($_.Exception.Response) {
            $statusCode = [int]$_.Exception.Response.StatusCode
        }
        if ($statusCode -eq 403) {
            Write-Err "GitHub API rate limit exceeded. Try again later or use -Version to specify a tag."
        } else {
            Write-Err "Failed to fetch release info: $_"
        }
    }

    $script:VersionTag = $response.tag_name
    $script:DownloadBaseUrl = "https://github.com/$Repo/releases/download/$script:VersionTag"

    Write-Info "Installing version: $script:VersionTag"
}

# ── 下载 ──

function Download-File {
    param(
        [string]$Url,
        [string]$OutputPath
    )

    $proxyParam = @{}
    if ($Proxy -ne "") {
        $proxyParam = @{ Proxy = $Proxy }
    }

    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutputPath @proxyParam -ErrorAction Stop
    } catch {
        Write-Err "Failed to download ${Url}: $_"
    }
}

function Download-Artifacts {
    $script:TempDir = Join-Path $env:TEMP "cct-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $script:TempDir -Force | Out-Null

    $script:ArchivePath = Join-Path $script:TempDir $ArtifactName
    $script:ChecksumPath = Join-Path $script:TempDir "$ArtifactName.sha256"

    $archiveUrl = "$script:DownloadBaseUrl/$ArtifactName"
    $checksumUrl = "$script:DownloadBaseUrl/$ArtifactName.sha256"

    Write-Info "Downloading $ArtifactName..."
    Download-File -Url $archiveUrl -OutputPath $script:ArchivePath

    if (-not $SkipChecksum) {
        Write-Info "Downloading checksum..."
        Download-File -Url $checksumUrl -OutputPath $script:ChecksumPath
    }
}

# ── 校验 ──

function Verify-Checksum {
    if ($SkipChecksum) {
        Write-Warn "Checksum verification skipped."
        return
    }

    $expectedHash = (Get-Content $script:ChecksumPath -Raw).Trim()
    # 处理两种格式：裸 hash 或 "hash  filename"
    $expectedHash = ($expectedHash -split '\s+')[0]

    $actualHash = (Get-FileHash -Path $script:ArchivePath -Algorithm SHA256).Hash.ToLower()
    $expectedHash = $expectedHash.ToLower()

    if ($actualHash -ne $expectedHash) {
        Write-Err "Checksum mismatch!`n  Expected: $expectedHash`n  Got:      $actualHash`n  The file may be corrupted."
    }

    Write-Info "Checksum verified."
}

# ── 安装目录 ──

function Resolve-InstallDir {
    if ($InstallDir -eq "") {
        $script:InstallDir = $DefaultInstallDir
    } else {
        $script:InstallDir = $InstallDir
    }

    if (-not (Test-Path $script:InstallDir)) {
        New-Item -ItemType Directory -Path $script:InstallDir -Force | Out-Null
        Write-Info "Created directory: $($script:InstallDir)"
    }
}

# ── 安装二进制 ──

function Install-Binary {
    Write-Info "Extracting archive..."
    Expand-Archive -Path $script:ArchivePath -DestinationPath $script:TempDir -Force

    $extractedBinary = Join-Path $script:TempDir $BinaryName

    if (-not (Test-Path $extractedBinary)) {
        Write-Err "Binary not found in archive: $extractedBinary"
    }

    $destPath = Join-Path $script:InstallDir $BinaryName

    if (Test-Path $destPath) {
        Write-Info "Replacing existing installation at $destPath"
    }

    Copy-Item -Path $extractedBinary -Destination $destPath -Force

    Write-Info "Installed $BinaryName to $destPath"
}

# ── PATH 管理 ──

function Ensure-Path {
    $installDir = $script:InstallDir
    $pathParts = $env:PATH -split [System.IO.Path]::PathSeparator

    if ($installDir -in $pathParts) {
        Write-Info "$installDir is already in PATH."
        return
    }

    # 添加到当前会话
    $env:PATH = "$installDir$([System.IO.Path]::PathSeparator)$env:PATH"

    # 持久化到用户环境变量
    $userPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$installDir*") {
        [System.Environment]::SetEnvironmentVariable(
            "PATH",
            "$installDir$([System.IO.Path]::PathSeparator)$userPath",
            "User"
        )
        Write-Info "Added $installDir to user PATH (persistent)."
    }

    Write-Info "Added $installDir to PATH for this session."
}

# ── 安装后验证 ──

function Verify-Installation {
    $cctPath = Join-Path $script:InstallDir $BinaryName

    if (Test-Path $cctPath) {
        try {
            $ver = & $cctPath --version 2>$null
            Write-Info "Verification: cct $ver"
        } catch {
            Write-Info "Verification: $cctPath exists."
        }
    } else {
        Write-Warn "Installation verification failed."
    }
}

# ── 清理 ──

function Cleanup {
    if ($script:TempDir -and (Test-Path $script:TempDir)) {
        Remove-Item -Path $script:TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

# ── 主流程 ──

if ($Help) { Show-Help }

try {
    Write-Banner
    Resolve-InstallDir
    Resolve-CctVersion
    Download-Artifacts
    Verify-Checksum
    Install-Binary
    Ensure-Path
    Verify-Installation

    Write-Host ""
    Write-Info "Installation complete! Run 'cct --help' to get started."
} finally {
    Cleanup
}
