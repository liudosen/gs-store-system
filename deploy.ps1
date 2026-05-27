param(
    [ValidateSet('all', 'backend', 'frontend')]
    [string]$Target = 'all',
    [string]$FrontendDir,
    [switch]$SkipFrontendBuild
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$ProjectRoot = $PSScriptRoot
$BackendDir = Join-Path $ProjectRoot "gs-store-system-backend"
$DockerExe = "C:\Program Files\Docker\Docker\resources\bin\docker.exe"
$AppName = "gs-store-system"
$PackageName = "$AppName-linux-x86_64"
$ReleaseDir = Join-Path $BackendDir "release"
$PackageDir = Join-Path $ReleaseDir $PackageName
$PackageFile = Join-Path $ReleaseDir "$PackageName.tar.gz"
$MergedEnvFile = Join-Path $ReleaseDir "$PackageName.env"
$FrontendArchiveName = "frontend-dist.tar.gz"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host $Message
}

function Assert-Command {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command not found: $Name"
    }
}

function Resolve-EnvFile {
    foreach ($name in @(".env", "env")) {
        $path = Join-Path $ProjectRoot $name
        if (Test-Path -LiteralPath $path) {
            return (Resolve-Path -LiteralPath $path).Path
        }
    }
    throw "Project env file not found. Expected .env or env under $ProjectRoot"
}

function Read-ProjectEnv {
    param([string]$Path)
    $map = @{}
    Get-Content -LiteralPath $Path | ForEach-Object {
        $line = $_
        if ([string]::IsNullOrWhiteSpace($line)) { return }
        if ($line.TrimStart().StartsWith("#")) { return }
        $idx = $line.IndexOf("=")
        if ($idx -lt 1) { return }
        $key = $line.Substring(0, $idx).Trim()
        $value = $line.Substring($idx + 1)
        $map[$key] = $value
    }
    return $map
}

function Get-EnvValue {
    param(
        [System.Collections.IDictionary]$Map,
        [string]$Name,
        [string]$Default = ""
    )
    if ($Map.Contains($Name) -and -not [string]::IsNullOrWhiteSpace([string]$Map[$Name])) {
        return [string]$Map[$Name]
    }
    return $Default
}

function Get-RequiredProjectEnv {
    param(
        [System.Collections.IDictionary]$Map,
        [string]$Name
    )
    $value = Get-EnvValue -Map $Map -Name $Name
    if ([string]::IsNullOrWhiteSpace($value)) {
        throw "Required value '$Name' was not found in project env file."
    }
    return $value
}

function Normalize-Migrations {
    $migrations = Join-Path $BackendDir "migrations"
    if (-not (Test-Path -LiteralPath $migrations)) { return }
    Get-ChildItem -LiteralPath $migrations -Recurse -File -Filter *.sql | ForEach-Object {
        $content = [System.IO.File]::ReadAllText($_.FullName)
        $content = $content -replace "`r`n", "`n"
        [System.IO.File]::WriteAllText($_.FullName, $content, [System.Text.UTF8Encoding]::new($false))
    }
}

function Write-BackendEnvFile {
    param(
        [System.Collections.IDictionary]$Map,
        [string]$Destination
    )
    if (-not (Test-Path -LiteralPath (Split-Path $Destination -Parent))) {
        New-Item -ItemType Directory -Force -Path (Split-Path $Destination -Parent) | Out-Null
    }

    $orderedKeys = @(
        'DATABASE_HOST',
        'DATABASE_PORT',
        'DATABASE_NAME',
        'DATABASE_USER',
        'DATABASE_PASSWORD',
        'REDIS_HOST',
        'REDIS_PORT',
        'REDIS_USERNAME',
        'REDIS_PASSWORD',
        'REDIS_DB',
        'JWT_SECRET',
        'JWT_EXPIRY_HOURS',
        'SERVER_HOST',
        'SERVER_PORT',
        'BCRYPT_COST',
        'ADMIN_USERNAME',
        'ADMIN_PASSWORD',
        'WEIXIN_APPID',
        'WEIXIN_SECRET',
        'JK_SELLER_USERNAME',
        'JK_SELLER_PASSWORD',
        'OSS_ENDPOINT',
        'OSS_ACCESS_KEY_ID',
        'OSS_ACCESS_KEY_SECRET',
        'OSS_BUCKET',
        'OSS_DOMAIN',
        'LOG_DIR',
        'LOG_MAX_FILE_SIZE',
        'LOG_MAX_AGE_DAYS'
    )

    $lines = foreach ($key in $orderedKeys) {
        "{0}={1}" -f $key, (Get-EnvValue -Map $Map -Name $key)
    }
    Set-Content -LiteralPath $Destination -Value $lines -Encoding ASCII
}

function Get-Docker {
    if (Test-Path -LiteralPath $DockerExe) {
        return $DockerExe
    }
    $cmd = Get-Command docker -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    throw "Docker not found. Install Docker Desktop first."
}

function Invoke-BackendBuild {
    param([System.Collections.IDictionary]$Map)

    Assert-Command tar
    $docker = Get-Docker

    Write-Step "[backend 1/4] Normalizing migrations"
    Normalize-Migrations

    Write-Step "[backend 2/4] Building Docker image"
    & $docker info *> $null
    if ($LASTEXITCODE -ne 0) {
        throw "Docker engine is not running."
    }

    Push-Location $BackendDir
    try {
        $proxy = "http://host.docker.internal:7890"
        & $docker build `
            --build-arg "HTTP_PROXY=$proxy" `
            --build-arg "HTTPS_PROXY=$proxy" `
            --build-arg "ALL_PROXY=socks5://host.docker.internal:7890" `
            -f Dockerfile.linux-build `
            -t "$AppName-builder" `
            .
        if ($LASTEXITCODE -ne 0) {
            throw "Docker build failed."
        }

        Write-Step "[backend 3/4] Extracting Linux artifacts"
        $artifactDir = Join-Path $BackendDir "target\linux-artifacts"
        if (Test-Path -LiteralPath $artifactDir) {
            Remove-Item -LiteralPath $artifactDir -Recurse -Force
        }
        New-Item -ItemType Directory -Force -Path (Join-Path $BackendDir "target") | Out-Null

        $containerName = "gs-store-system-build-$([System.Guid]::NewGuid().ToString('N'))"
        & $docker create --name $containerName "$AppName-builder" | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create Docker container."
        }
        try {
            & $docker cp "${containerName}:/app/." $artifactDir
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to extract Linux artifacts."
            }
        }
        finally {
            & $docker rm $containerName *> $null
        }

        Copy-Item -LiteralPath (Join-Path $artifactDir $AppName) -Destination (Join-Path $BackendDir "target\$AppName") -Force

        Write-Step "[backend 4/4] Packaging release"
        if (Test-Path -LiteralPath $PackageDir) { Remove-Item -LiteralPath $PackageDir -Recurse -Force }
        if (Test-Path -LiteralPath $PackageFile) { Remove-Item -LiteralPath $PackageFile -Force }
        New-Item -ItemType Directory -Force -Path $PackageDir | Out-Null

        Copy-Item -LiteralPath (Join-Path $BackendDir "target\$AppName") -Destination (Join-Path $PackageDir $AppName) -Force
        Get-ChildItem -LiteralPath $artifactDir -Filter "libonnxruntime.so*" -File | ForEach-Object {
            Copy-Item -LiteralPath $_.FullName -Destination $PackageDir -Force
        }
        $migrations = Join-Path $BackendDir "migrations"
        if (Test-Path -LiteralPath $migrations) {
            Copy-Item -LiteralPath $migrations -Destination (Join-Path $PackageDir "migrations") -Recurse -Force
        }

        tar -czf $PackageFile -C $ReleaseDir $PackageName
        if ($LASTEXITCODE -ne 0) {
            throw "Packaging failed."
        }

        Write-BackendEnvFile -Map $Map -Destination $MergedEnvFile
        Write-Host "Backend package: $PackageFile"
    }
    finally {
        Pop-Location
    }
}

function Invoke-Python {
    param(
        [string]$Script,
        [System.Collections.IDictionary]$Environment
    )
    Assert-Command python
    $old = @{}
    foreach ($key in $Environment.Keys) {
        $old[$key] = [Environment]::GetEnvironmentVariable($key, 'Process')
        [Environment]::SetEnvironmentVariable($key, [string]$Environment[$key], 'Process')
    }
    try {
        $Script | python -
        if ($LASTEXITCODE -ne 0) {
            throw "Python deployment helper failed."
        }
    }
    finally {
        foreach ($key in $Environment.Keys) {
            [Environment]::SetEnvironmentVariable($key, $old[$key], 'Process')
        }
    }
}

function Invoke-BackendDeploy {
    param([System.Collections.IDictionary]$Map)
    $server = Get-EnvValue -Map $Map -Name "DEPLOY_SERVER" -Default "47.103.220.84"
    $user = Get-EnvValue -Map $Map -Name "DEPLOY_USER" -Default "root"
    $password = Get-RequiredProjectEnv -Map $Map -Name "DEPLOY_SSH_PASSWORD"
    $deployDir = Get-EnvValue -Map $Map -Name "DEPLOY_DIR" -Default "/root/workspace/gs-store-system"
    $port = Get-EnvValue -Map $Map -Name "DEPLOY_PORT" -Default (Get-EnvValue -Map $Map -Name "SERVER_PORT" -Default "8081")
    $remoteEnv = Get-EnvValue -Map $Map -Name "REMOTE_SYSTEM_ENV" -Default "/etc/gs-store-system.env"
    $remotePackage = "$PackageName.tar.gz"

    Write-Step "[backend deploy] Uploading package and restarting service"
    $script = @'
import os
import posixpath
import sys
import time

import paramiko

server = os.environ["DEPLOY_SERVER"]
user = os.environ["DEPLOY_USER"]
password = os.environ["DEPLOY_SSH_PASSWORD"]
deploy_dir = os.environ["DEPLOY_DIR"]
port = int(os.environ["DEPLOY_PORT"])
remote_env = os.environ["REMOTE_SYSTEM_ENV"]
package_file = os.environ["PACKAGE_FILE"]
env_file = os.environ["MERGED_ENV_FILE"]
remote_package = os.environ["REMOTE_PACKAGE"]
app_name = os.environ["APP_NAME"]
package_name = os.environ["PACKAGE_NAME"]

def log(message):
    print(message, flush=True)

def run(ssh, command, description):
    log(f"[deploy] {description} ...")
    stdin, stdout, stderr = ssh.exec_command(command)
    channel = stdout.channel
    while True:
        while channel.recv_ready():
            print(channel.recv(4096).decode("utf-8", "replace"), end="", flush=True)
        while channel.recv_stderr_ready():
            print(channel.recv_stderr(4096).decode("utf-8", "replace"), end="", file=sys.stderr, flush=True)
        if channel.exit_status_ready():
            code = channel.recv_exit_status()
            while channel.recv_ready():
                print(channel.recv(4096).decode("utf-8", "replace"), end="", flush=True)
            while channel.recv_stderr_ready():
                print(channel.recv_stderr(4096).decode("utf-8", "replace"), end="", file=sys.stderr, flush=True)
            if code:
                raise SystemExit(f"{description} failed with exit code {code}")
            log(f"[deploy] {description} done")
            return
        time.sleep(0.05)

ssh = paramiko.SSHClient()
ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
ssh.connect(server, username=user, password=password, look_for_keys=False, allow_agent=False, timeout=30)
try:
    log(f"[deploy] Connected to {server} as {user}")
    run(ssh, f"mkdir -p {deploy_dir}/logs", "Creating remote logs directory")
    log("[deploy] Uploading package and environment file ...")
    sftp = ssh.open_sftp()
    sftp.put(package_file, posixpath.join(deploy_dir, remote_package))
    sftp.put(env_file, remote_env)
    sftp.close()
    log("[deploy] Upload complete")
    run(
        ssh,
        f"tr -d '\\r' < {remote_env} > {remote_env}.tmp && mv {remote_env}.tmp {remote_env} && chmod 600 {remote_env}",
        "Normalizing remote environment file",
    )

    start_cmd = f"""set -e
cd {deploy_dir}
echo '[remote] Loading environment'
if [ -f {remote_env} ]; then
    set -a
    . {remote_env}
    set +a
fi
echo '[remote] Stopping existing process on port {port}'
PID=$(lsof -ti :{port} 2>/dev/null || true)
if [ -n "$PID" ]; then
    kill -15 $PID || true
    sleep 2
    PID=$(lsof -ti :{port} 2>/dev/null || true)
    if [ -n "$PID" ]; then
        kill -9 $PID || true
    fi
fi
echo '[remote] Extracting package'
tar -xzf {remote_package} -C .
if [ -d {package_name} ]; then
    cp -f {package_name}/{app_name} ./{app_name}
    cp -f {package_name}/libonnxruntime.so* ./ 2>/dev/null || true
    if [ -d {package_name}/migrations ]; then
        rm -rf ./migrations
        cp -R {package_name}/migrations ./migrations
    fi
    rm -rf {package_name}
fi
chmod +x {app_name}
rm -f {remote_package}
echo '[remote] Starting service'
LD_LIBRARY_PATH="{deploy_dir}:$LD_LIBRARY_PATH" nohup ./{app_name} > logs/app.log 2>&1 &
echo '[remote] Waiting for service health'
READY=0
for _ in $(seq 1 60); do
    if curl -fsS --max-time 2 http://127.0.0.1:{port}/health > /dev/null 2>&1; then
        READY=1
        break
    fi
    sleep 1
done
if [ "$READY" -eq 1 ]; then
    echo '[remote] Deployment complete'
else
    echo '[remote] Service failed to start'
    tail -n 30 logs/app.log
    exit 1
fi"""
    run(ssh, start_cmd, "Restarting remote service")
    log("[deploy] Backend deployment finished")
finally:
    ssh.close()
'@

    Invoke-Python -Script $script -Environment @{
        DEPLOY_SERVER = $server
        DEPLOY_USER = $user
        DEPLOY_SSH_PASSWORD = $password
        DEPLOY_DIR = $deployDir
        DEPLOY_PORT = $port
        REMOTE_SYSTEM_ENV = $remoteEnv
        PACKAGE_FILE = (Resolve-Path -LiteralPath $PackageFile).Path
        MERGED_ENV_FILE = (Resolve-Path -LiteralPath $MergedEnvFile).Path
        REMOTE_PACKAGE = $remotePackage
        APP_NAME = $AppName
        PACKAGE_NAME = $PackageName
    }
}

function Invoke-FrontendDeploy {
    param([System.Collections.IDictionary]$Map)
    Assert-Command npm
    Assert-Command tar

    $server = Get-EnvValue -Map $Map -Name "DEPLOY_SERVER" -Default "47.103.220.84"
    $user = Get-EnvValue -Map $Map -Name "DEPLOY_USER" -Default "root"
    $password = Get-RequiredProjectEnv -Map $Map -Name "DEPLOY_SSH_PASSWORD"
    $remoteBase = Get-EnvValue -Map $Map -Name "DEPLOY_FRONTEND_DIR" -Default "/root/workspace/gs-store-system/backend"

    if ([string]::IsNullOrWhiteSpace($FrontendDir)) {
        $FrontendDir = Join-Path $ProjectRoot "frontend"
    }
    $frontendPath = (Resolve-Path -LiteralPath $FrontendDir -ErrorAction SilentlyContinue)
    if (-not $frontendPath) {
        throw "Frontend directory not found: $FrontendDir"
    }
    $frontendPath = $frontendPath.Path
    $packageJson = Join-Path $frontendPath "package.json"
    $distDir = Join-Path $frontendPath "dist"

    Write-Step "[frontend 1/4] Building frontend"
    if (-not $SkipFrontendBuild) {
        if (-not (Test-Path -LiteralPath $packageJson)) {
            throw "Frontend package.json not found: $packageJson"
        }
        Push-Location $frontendPath
        try {
            if (-not (Test-Path -LiteralPath "node_modules")) {
                npm install
            }
            npm run build
        }
        finally {
            Pop-Location
        }
    }
    else {
        Write-Host "Skipping frontend build."
    }
    if (-not (Test-Path -LiteralPath $distDir)) {
        throw "Frontend dist directory not found: $distDir"
    }

    Write-Step "[frontend 2/4] Packaging dist"
    if (-not (Test-Path -LiteralPath $ReleaseDir)) {
        New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
    }
    $archivePath = Join-Path $ReleaseDir $FrontendArchiveName
    if (Test-Path -LiteralPath $archivePath) {
        Remove-Item -LiteralPath $archivePath -Force
    }
    tar -czf $archivePath -C $frontendPath dist
    if ($LASTEXITCODE -ne 0) {
        throw "Frontend packaging failed."
    }

    Write-Step "[frontend 3/4] Uploading dist"
    $script = @'
import os
import posixpath
import paramiko

server = os.environ["DEPLOY_SERVER"]
user = os.environ["DEPLOY_USER"]
password = os.environ["DEPLOY_SSH_PASSWORD"]
remote_base = os.environ["DEPLOY_REMOTE_BASE"]
local_archive = os.environ["DEPLOY_ARCHIVE_PATH"]
archive_name = os.path.basename(local_archive)
remote_archive = posixpath.join(remote_base, archive_name)
remote_dist = posixpath.join(remote_base, "dist")

client = paramiko.SSHClient()
client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
client.connect(server, username=user, password=password, look_for_keys=False, allow_agent=False, timeout=30)
try:
    sftp = client.open_sftp()
    stdin, stdout, stderr = client.exec_command(f"mkdir -p {remote_base}")
    if stdout.channel.recv_exit_status() != 0:
        raise SystemExit("failed to create remote frontend directory")
    sftp.put(local_archive, remote_archive)
    sftp.close()

    commands = [
        f"rm -rf {remote_dist}",
        f"cd {remote_base} && tar -xzf {archive_name}",
        f"rm -f {remote_archive}",
        f"test -f {remote_dist}/index.html && echo '[remote] frontend dist extraction ok'",
    ]
    for command in commands:
        stdin, stdout, stderr = client.exec_command(command)
        code = stdout.channel.recv_exit_status()
        out = stdout.read().decode("utf-8", "replace")
        err = stderr.read().decode("utf-8", "replace")
        if out:
            print(out, end="")
        if err:
            print(err, end="")
        if code != 0:
            raise SystemExit(f"remote command failed ({code}): {command}")
finally:
    client.close()
'@
    Invoke-Python -Script $script -Environment @{
        DEPLOY_SERVER = $server
        DEPLOY_USER = $user
        DEPLOY_SSH_PASSWORD = $password
        DEPLOY_REMOTE_BASE = $remoteBase
        DEPLOY_ARCHIVE_PATH = (Resolve-Path -LiteralPath $archivePath).Path
    }

    Write-Step "[frontend 4/4] Deployment complete"
    Write-Host "Remote frontend dist: $remoteBase/dist"
}

$EnvFile = Resolve-EnvFile
$ProjectEnv = Read-ProjectEnv -Path $EnvFile

Write-Host "=========================================="
Write-Host "  Welfare Store Deployment"
Write-Host "=========================================="
Write-Host "  Target:   $Target"
Write-Host "  Env file: $EnvFile"
Write-Host "=========================================="

switch ($Target) {
    'backend' {
        Invoke-BackendBuild -Map $ProjectEnv
        Invoke-BackendDeploy -Map $ProjectEnv
    }
    'frontend' {
        Invoke-FrontendDeploy -Map $ProjectEnv
    }
    'all' {
        Invoke-BackendBuild -Map $ProjectEnv
        Invoke-BackendDeploy -Map $ProjectEnv
        Invoke-FrontendDeploy -Map $ProjectEnv
    }
}

Write-Host ""
Write-Host "Deployment complete."


