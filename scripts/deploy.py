import argparse
import os
import posixpath
import shutil
import socket
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.request
import zipfile
from pathlib import Path

import paramiko


ROOT = Path(__file__).resolve().parents[1]
FRONTEND = ROOT / "frontend"
DIST = FRONTEND / "dist"
PACKAGE_DIR = ROOT / "target" / "deploy"
PACKAGE_PATH = PACKAGE_DIR / "gs-store-system-release.tar.gz"
APP_NAME = "gs-store-system"
REMOTE_BASE = "/opt/gs-store-system"
REMOTE_RELEASES = f"{REMOTE_BASE}/releases"
REMOTE_CURRENT = f"{REMOTE_BASE}/current"
REMOTE_ENV = f"{REMOTE_BASE}/.env"
SERVICE_NAME = "gs-store-system.service"
LINUX_BINARY_NAME = "gs-store-system"
ZIG_VERSION = "0.13.0"


class DeployError(RuntimeError):
    pass


def log(message: str) -> None:
    print(f"[deploy] {message}", flush=True)


def run(command, cwd=ROOT, env=None, shell=False) -> None:
    printable = command if isinstance(command, str) else " ".join(command)
    log(printable)
    subprocess.run(command, cwd=cwd, env=env, shell=shell, check=True)


def read_env(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip().strip('"').strip("'")
    return values


def require_env(env: dict[str, str], key: str) -> str:
    value = env.get(key)
    if not value:
        raise DeployError(f".env missing required key: {key}")
    return value


def command_exists(name: str) -> bool:
    return shutil.which(name) is not None


def cargo_bin() -> str:
    cargo = shutil.which("cargo")
    if not cargo:
        raise DeployError("cargo not found")
    return cargo


def build_frontend() -> None:
    npm = "npm.cmd" if os.name == "nt" else "npm"
    run([npm, "run", "build"], cwd=FRONTEND)
    if not (DIST / "index.html").exists():
        raise DeployError("frontend build did not create frontend/dist/index.html")


def build_backend_with_docker() -> Path | None:
    if not command_exists("docker"):
        return None
    try:
        subprocess.run(["docker", "info"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    except subprocess.CalledProcessError:
        return None

    log("building Linux backend with Docker")
    proxy_args = []
    for key in ("HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "http_proxy", "https_proxy", "all_proxy"):
        value = os.environ.get(key)
        if value:
            proxy_args.extend(["-e", f"{key}={value}"])

    docker_cmd = (
        "rustup target add x86_64-unknown-linux-gnu && "
        "cargo build --release --target x86_64-unknown-linux-gnu"
    )
    run(
        [
            "docker",
            "run",
            "--rm",
            *proxy_args,
            "-v",
            f"{ROOT.as_posix()}:/work",
            "-w",
            "/work",
            "rust:1-bookworm",
            "bash",
            "-lc",
            docker_cmd,
        ]
    )
    binary = ROOT / "target" / "x86_64-unknown-linux-gnu" / "release" / LINUX_BINARY_NAME
    if binary.exists():
        return binary
    return None


def find_zig() -> str | None:
    zig = shutil.which("zig")
    if zig:
        return zig
    bundled = ROOT / ".tools" / "zig" / ("zig.exe" if os.name == "nt" else "zig")
    if bundled.exists():
        return str(bundled)
    return None


def ensure_zig() -> str | None:
    zig = find_zig()
    if zig:
        return zig
    if os.name != "nt":
        return None

    tools_dir = ROOT / ".tools"
    zip_path = tools_dir / f"zig-windows-x86_64-{ZIG_VERSION}.zip"
    extract_dir = tools_dir / f"zig-windows-x86_64-{ZIG_VERSION}"
    final_dir = tools_dir / "zig"
    url = f"https://ziglang.org/download/{ZIG_VERSION}/zig-windows-x86_64-{ZIG_VERSION}.zip"

    tools_dir.mkdir(parents=True, exist_ok=True)
    if not zip_path.exists():
        log(f"downloading Zig {ZIG_VERSION}")
        urllib.request.urlretrieve(url, zip_path)

    if final_dir.exists():
        shutil.rmtree(final_dir)
    log("extracting Zig")
    with zipfile.ZipFile(zip_path) as archive:
        archive.extractall(tools_dir)
    extract_dir.rename(final_dir)
    return find_zig()


def build_backend_with_zig() -> Path | None:
    zig = ensure_zig()
    if not zig:
        return None

    log("building Linux backend with local Zig linker")
    run(["rustup", "target", "add", "x86_64-unknown-linux-gnu"])

    wrapper_dir = ROOT / "target" / "deploy-tools"
    wrapper_dir.mkdir(parents=True, exist_ok=True)
    if os.name == "nt":
        py_wrapper = wrapper_dir / "zig-linux-cc.py"
        py_wrapper.write_text(
            "import subprocess, sys\n"
            f"zig = {zig!r}\n"
            "args = []\n"
            "skip = False\n"
            "for item in sys.argv[1:]:\n"
            "    if skip:\n"
            "        skip = False\n"
            "        continue\n"
            "    if item == '--target=x86_64-unknown-linux-gnu':\n"
            "        continue\n"
            "    if item == '--target' or item == '-target':\n"
            "        skip = True\n"
            "        continue\n"
            "    args.append(item)\n"
            "raise SystemExit(subprocess.call([zig, 'cc', '-target', 'x86_64-linux-gnu', *args]))\n",
            encoding="utf-8",
        )
        wrapper = wrapper_dir / "zig-linux-linker.cmd"
        wrapper.write_text(f'@echo off\r\npython "{py_wrapper}" %*\r\n', encoding="utf-8")
        ar_wrapper = wrapper_dir / "zig-linux-ar.cmd"
        ar_wrapper.write_text(f'@echo off\r\n"{zig}" ar %*\r\n', encoding="utf-8")
    else:
        wrapper = wrapper_dir / "zig-linux-linker"
        wrapper.write_text(f'#!/usr/bin/env sh\n"{zig}" cc -target x86_64-linux-gnu "$@"\n', encoding="utf-8")
        wrapper.chmod(0o755)
        ar_wrapper = wrapper_dir / "zig-linux-ar"
        ar_wrapper.write_text(f'#!/usr/bin/env sh\n"{zig}" ar "$@"\n', encoding="utf-8")
        ar_wrapper.chmod(0o755)

    env = os.environ.copy()
    env["CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER"] = str(wrapper)
    env["CC_x86_64_unknown_linux_gnu"] = str(wrapper)
    env["CC_x86_64-unknown-linux-gnu"] = str(wrapper)
    env["TARGET_CC"] = str(wrapper)
    env["AR_x86_64_unknown_linux_gnu"] = str(ar_wrapper)
    run(
        [
            cargo_bin(),
            "build",
            "--release",
            "--target",
            "x86_64-unknown-linux-gnu",
        ],
        env=env,
    )
    binary = ROOT / "target" / "x86_64-unknown-linux-gnu" / "release" / f"{LINUX_BINARY_NAME}.exe"
    if not binary.exists():
        binary = ROOT / "target" / "x86_64-unknown-linux-gnu" / "release" / LINUX_BINARY_NAME
    return binary if binary.exists() else None


def build_backend() -> Path:
    binary = build_backend_with_docker()
    if binary:
        return binary

    if os.environ.get("GS_STORE_SYSTEM_ALLOW_ZIG_CROSS") == "1":
        binary = build_backend_with_zig()
        if binary:
            return binary

    raise DeployError(
        "No local Linux build method available. Start Docker Desktop and rerun this script. "
        "Zig cross-linking is disabled by default because this machine lacks the MSVC linker required by Rust build scripts. "
        "Set GS_STORE_SYSTEM_ALLOW_ZIG_CROSS=1 only after installing Visual Studio Build Tools."
    )


def make_package(binary: Path) -> Path:
    PACKAGE_DIR.mkdir(parents=True, exist_ok=True)
    if PACKAGE_PATH.exists():
        PACKAGE_PATH.unlink()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        app_dir = tmp_path / APP_NAME
        (app_dir / "frontend").mkdir(parents=True)
        shutil.copy2(binary, app_dir / LINUX_BINARY_NAME)
        shutil.copytree(DIST, app_dir / "frontend" / "dist")

        with tarfile.open(PACKAGE_PATH, "w:gz") as tar:
            tar.add(app_dir, arcname=APP_NAME)

    log(f"package created: {PACKAGE_PATH}")
    return PACKAGE_PATH


def connect(env: dict[str, str]) -> paramiko.SSHClient:
    host = require_env(env, "DEPLOY_SERVER")
    username = require_env(env, "DEPLOY_USER")
    password = require_env(env, "DEPLOY_SSH_PASSWORD")
    port = int(env.get("DEPLOY_SSH_PORT") or env.get("DEPLOY_PORT") or 22)

    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    log(f"connecting to {username}@{host}:{port}")
    client.connect(
        hostname=host,
        port=port,
        username=username,
        password=password,
        timeout=20,
        banner_timeout=20,
        auth_timeout=20,
        look_for_keys=False,
        allow_agent=False,
    )
    return client


def remote_run(client: paramiko.SSHClient, command: str, hide_output: bool = False) -> str:
    log(f"remote: {command}")
    stdin, stdout, stderr = client.exec_command(command, get_pty=True)
    out = stdout.read().decode("utf-8", errors="replace")
    err = stderr.read().decode("utf-8", errors="replace")
    code = stdout.channel.recv_exit_status()
    if not hide_output and out.strip():
        print(out.strip())
    if code != 0:
        if err.strip():
            print(err.strip(), file=sys.stderr)
        raise DeployError(f"remote command failed with exit code {code}: {command}")
    return out


def sftp_put(client: paramiko.SSHClient, local: Path, remote: str) -> None:
    log(f"upload: {local.name} -> {remote}")
    with client.open_sftp() as sftp:
        sftp.put(str(local), remote)


def install_remote(client: paramiko.SSHClient, package_path: Path, env_path: Path) -> None:
    timestamp = time.strftime("%Y%m%d%H%M%S")
    release_dir = f"{REMOTE_RELEASES}/{timestamp}"
    remote_package = f"/tmp/{package_path.name}"

    remote_run(client, f"mkdir -p {REMOTE_RELEASES}")
    sftp_put(client, package_path, remote_package)
    sftp_put(client, env_path, REMOTE_ENV)

    remote_run(
        client,
        " && ".join(
            [
                f"rm -rf {release_dir}",
                f"mkdir -p {release_dir}",
                f"tar -xzf {remote_package} -C {release_dir} --strip-components=1",
                f"chmod +x {release_dir}/{LINUX_BINARY_NAME}",
                f"ln -sfn {release_dir} {REMOTE_CURRENT}",
            ]
        ),
    )


def install_systemd(client: paramiko.SSHClient) -> None:
    service = f"""[Unit]
Description=gs-store-system Rust web service
After=network.target

[Service]
Type=simple
WorkingDirectory={REMOTE_CURRENT}
EnvironmentFile={REMOTE_ENV}
ExecStart={REMOTE_CURRENT}/{LINUX_BINARY_NAME}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"""
    remote_service = f"/etc/systemd/system/{SERVICE_NAME}"
    escaped = service.replace("\\", "\\\\").replace("$", "\\$").replace("`", "\\`")
    remote_run(client, f"cat > {remote_service} <<'EOF'\n{escaped}EOF\n")
    remote_run(client, f"systemctl daemon-reload && systemctl enable {SERVICE_NAME}")


def install_nginx(client: paramiko.SSHClient) -> None:
    config = (ROOT / "deploy" / "nginx" / "gs-store-system.conf").read_text(encoding="utf-8")
    target = "/etc/nginx/conf.d/gs-store-system.conf"
    remote_run(client, "grep -R \"server_name .*www.mcx59481.cn\" /etc/nginx/conf.d /etc/nginx/sites-enabled 2>/dev/null || true")
    remote_run(client, f"cat > {target} <<'EOF'\n{config}EOF\n")
    remote_run(client, "nginx -t")


def restart_and_verify(client: paramiko.SSHClient) -> None:
    remote_run(client, f"systemctl restart {SERVICE_NAME}")
    remote_run(client, f"sleep 2 && systemctl --no-pager --full status {SERVICE_NAME} | sed -n '1,18p'")
    remote_run(client, "curl -fsS http://127.0.0.1:9000/api/health")
    remote_run(client, "systemctl reload nginx")


def verify_public() -> None:
    import urllib.request

    for url in ("https://www.mcx59481.cn/", "https://mcx59481.cn/"):
        log(f"verify public: {url}")
        with urllib.request.urlopen(url, timeout=20) as response:
            if response.status >= 400:
                raise DeployError(f"public verify failed: {url} -> {response.status}")
            log(f"{url} -> {response.status}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Build locally and deploy gs-store-system to remote Linux server.")
    parser.add_argument("--skip-build", action="store_true", help="reuse existing package")
    parser.add_argument("--skip-nginx", action="store_true", help="do not write nginx config")
    parser.add_argument("--skip-public-verify", action="store_true", help="do not request public domain")
    args = parser.parse_args()

    env_path = ROOT / ".env"
    if not env_path.exists():
        raise DeployError(".env not found")
    env = read_env(env_path)

    if args.skip_build:
        package = PACKAGE_PATH
        if not package.exists():
            raise DeployError(f"package not found: {package}")
    else:
        build_frontend()
        binary = build_backend()
        package = make_package(binary)

    client = connect(env)
    try:
        install_remote(client, package, env_path)
        install_systemd(client)
        if not args.skip_nginx:
            install_nginx(client)
        restart_and_verify(client)
    finally:
        client.close()

    if not args.skip_public_verify:
        verify_public()

    log("deployment completed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (DeployError, subprocess.CalledProcessError, socket.error, paramiko.SSHException) as error:
        print(f"[deploy] ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)


