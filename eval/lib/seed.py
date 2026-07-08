"""Seed fixture + case extras into a running TestServer via HTTP POST.

Ported from crates/eval-runner/src/seed.rs. Routes verified against
crates/heramind-api/src/server/router.rs.
"""
from __future__ import annotations

import requests


def _post(server, path: str, body):
    return server.post(path, body)


def _seed_devices(server, items: list):
    for d in items or []:
        r = _post(server, "/devices", d)
        if not r.ok:
            raise RuntimeError(
                f"seed device {d.get('device_id') or d.get('id')} -> "
                f"{r.status_code}: {r.text}"
            )


def _seed_metrics(server, items: list):
    # WriteMetricRequest expects field "metric".
    for m in items or []:
        device_id = m.get("device_id")
        if not device_id:
            raise RuntimeError(f"metric missing device_id: {m}")
        body = {
            "metric": m.get("metric"),
            "value": m.get("value"),
        }
        r = _post(server, f"/devices/{device_id}/metrics", body)
        if not r.ok:
            raise RuntimeError(
                f"seed metric -> {r.status_code}: {r.text}"
            )


def _seed_simple(server, items: list, path: str, kind: str):
    for x in items or []:
        r = _post(server, path, x)
        if not r.ok:
            raise RuntimeError(
                f"seed {kind} -> {r.status_code}: {r.text}"
            )


def seed_fixture(server, fixture: dict):
    _seed_devices(server, fixture.get("devices"))
    _seed_metrics(server, fixture.get("metrics"))
    _seed_simple(server, fixture.get("rules"), "/rules", "rule")
    _seed_simple(server, fixture.get("agents"), "/agents", "agent")
    _seed_simple(server, fixture.get("transforms"), "/automations", "transform")
    _seed_simple(server, fixture.get("dashboards"), "/dashboards", "dashboard")
    _seed_simple(server, fixture.get("channels"), "/messages/channels", "channel")
    # extensions omitted — Tier 1 doesn't ship .nep binaries


def seed_extras(server, extras: dict):
    seed_fixture(server, extras)
