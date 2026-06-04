#!/usr/bin/env python3
"""
BiblePro / Zehut license key generator (development tool).

Generates a signed ES256 JWT that can be pasted into the activation screen.
The private key here corresponds to the public key embedded in the desktop app.

Usage:
    python3 tools/gen_license.py --device DEV_ID --plan standard --org "Grace Church" --days 365

Requires:
    pip install PyJWT cryptography
"""

import argparse
import time
import jwt  # PyJWT


# ── Private key (KEEP THIS SECRET — backend only) ────────────────────────────
PRIVATE_KEY_PEM = """-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIAWHV4BCrxdjravqyzTb1k5pukdbLW5LM+n1kd0nGQaRoAoGCCqGSM49
AwEHoUQDQgAENXBr3alqXj4H+y2RQaPHTwmSDVNI44B6pA0nJPo/ZRUPE1z80poO
RvTdjHoVEVyZtMpbkrCFZNZMk2insfLwAw==
-----END EC PRIVATE KEY-----"""

ISSUER = "biblepro-zehut"


def generate(device_id: str, plan: str, org: str, org_id: str, max_devices: int, days: int) -> str:
    now = int(time.time())
    payload = {
        "sub": org_id,
        "org": org,
        "plan": plan,
        "max_devices": max_devices,
        "device_id": device_id,
        "iat": now,
        "exp": now + days * 86400,
        "iss": ISSUER,
    }
    token = jwt.encode(payload, PRIVATE_KEY_PEM, algorithm="ES256")
    return token


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate a BiblePro license JWT")
    parser.add_argument("--device",      required=True,  help="Machine fingerprint (run app to see it in License tab)")
    parser.add_argument("--plan",        default="standard", choices=["free", "standard", "premium"])
    parser.add_argument("--org",         default="Test Church")
    parser.add_argument("--org-id",      default="org_test_001")
    parser.add_argument("--max-devices", default=3, type=int)
    parser.add_argument("--days",        default=365, type=int, help="Validity in days")
    args = parser.parse_args()

    token = generate(
        device_id=args.device,
        plan=args.plan,
        org=args.org,
        org_id=args.org_id,
        max_devices=args.max_devices,
        days=args.days,
    )
    print("\n── License Key ─────────────────────────────────────────────────────────────")
    print(token)
    print("────────────────────────────────────────────────────────────────────────────")
    print(f"\nPlan:    {args.plan}")
    print(f"Org:     {args.org}")
    print(f"Device:  {args.device}")
    print(f"Expires: {args.days} days from now")
    print("\nPaste this key into BiblePro → License tab → Activate.\n")
