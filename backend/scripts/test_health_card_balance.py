#!/usr/bin/env python3
"""Smoke test for the mini-app health card balance API.

The script logs in first to obtain a bearer token, then queries the
health-card balance endpoint with that token and prints both responses.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import hmac
import json
import os
import socket
import struct
import time
import sys
import urllib.error
import urllib.request


def request_json(url: str, method: str = "GET", headers: dict[str, str] | None = None, body: dict | None = None):
    req_headers = {"Content-Type": "application/json"}
    if headers:
        req_headers.update(headers)

    data = None
    if body is not None:
        data = json.dumps(body, ensure_ascii=False).encode("utf-8")

    req = urllib.request.Request(url, data=data, method=method, headers=req_headers)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            raw = resp.read().decode("utf-8")
            return resp.status, json.loads(raw)
    except urllib.error.HTTPError as exc:
        raw = exc.read().decode("utf-8", errors="replace")
        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError:
            parsed = {"raw": raw}
        return exc.code, parsed


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def make_jwt(secret: str, openid: str, wechat_id: int) -> str:
    header = {"alg": "HS256", "typ": "JWT"}
    now = int(time.time())
    payload = {
        "sub": openid,
        "wechat_id": wechat_id,
        "openid": openid,
        "iat": now,
        "exp": now + 30 * 24 * 3600,
    }
    signing_input = ".".join(
        [
            b64url(json.dumps(header, separators=(",", ":")).encode("utf-8")),
            b64url(json.dumps(payload, separators=(",", ":")).encode("utf-8")),
        ]
    )
    sig = hmac.new(secret.encode("utf-8"), signing_input.encode("ascii"), hashlib.sha256).digest()
    return f"{signing_input}.{b64url(sig)}"


def redis_resp(*parts: str) -> bytes:
    chunks = [f"*{len(parts)}\r\n".encode("ascii")]
    for part in parts:
        encoded = part.encode("utf-8")
        chunks.append(f"${len(encoded)}\r\n".encode("ascii"))
        chunks.append(encoded)
        chunks.append(b"\r\n")
    return b"".join(chunks)


def redis_command(host: str, port: int, password: str, *parts: str) -> str:
    with socket.create_connection((host, port), timeout=10) as sock:
        if password:
            sock.sendall(redis_resp("AUTH", password))
            auth_reply = sock.recv(1024)
            if not auth_reply.startswith(b"+OK"):
                raise RuntimeError(f"redis auth failed: {auth_reply!r}")
        sock.sendall(redis_resp(*parts))
        reply = sock.recv(1024)
        return reply.decode("utf-8", errors="replace").strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="Health card balance API smoke test")
    parser.add_argument("--base-url", default="https://www.gsyl.cloud", help="API base URL")
    parser.add_argument("--openid", default="o-gZp1_l78Nnqcb9EOCdpToWoI6k", help="Test openid")
    parser.add_argument("--wechat-id", type=int, default=1, help="JWT claim value for wechat_id")
    parser.add_argument(
        "--jwt-secret",
        default="gs-store-system-secret-key-32-bytes-minimum",
        help="JWT signing secret",
    )
    parser.add_argument("--redis-host", default="47.103.220.84", help="Redis host")
    parser.add_argument("--redis-port", type=int, default=6379, help="Redis port")
    parser.add_argument("--redis-password", default="liu5tgb^TFC", help="Redis password")
    args = parser.parse_args()

    balance_url = args.base_url.rstrip("/") + "/api/mini/health-card/balance"
    token = make_jwt(args.jwt_secret, args.openid, args.wechat_id)
    redis_key = f"welfare:wechat:token:{token}"

    redis_reply = redis_command(
        args.redis_host,
        args.redis_port,
        args.redis_password,
        "SET",
        redis_key,
        str(args.wechat_id),
        "EX",
        str(30 * 24 * 3600),
    )
    print(f"REDIS_REPLY={redis_reply}")
    print(f"TOKEN={token}")

    balance_status, balance_resp = request_json(
        balance_url,
        method="GET",
        headers={"Authorization": f"Bearer {token}"},
    )
    print(f"BALANCE_STATUS={balance_status}")
    print(json.dumps(balance_resp, ensure_ascii=False, indent=2))

    return 0 if balance_status == 200 and balance_resp.get("code") == 200 else 2


if __name__ == "__main__":
    raise SystemExit(main())
