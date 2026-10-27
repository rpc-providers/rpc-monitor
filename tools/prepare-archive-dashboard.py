#!/usr/bin/env python3
"""Prepare a Grafana dashboard import without modifying the active dashboard.

Accepts a dashboard JSON or GET /api/dashboards/uid/... response on stdin.
Writes a separate preview import to stdout. No credentials or writes to Grafana.
"""
import copy
import json
import sys


def prepare(source):
    dashboard = copy.deepcopy(source.get("dashboard", source))
    dashboard.update(id=None, uid="rpc-archive-preview", version=0)
    dashboard["title"] = "RPC Providers - Archive preview"
    dashboard["tags"] = list(dict.fromkeys(dashboard.get("tags", []) + ["archive-preview"]))
    dashboard["description"] = (
        "Archive/pruned migration preview. Requires rpc_archive_status from the new monitor. "
        "1 = Archive, 0 = Pruned, -1 = Unknown. Missing data does not mean Pruned."
    )

    def walk(panels):
        for panel in panels:
            panel["title"] = panel.get("title", "").replace("block 0", "latest block")
            panel["targets"] = [
                target for target in panel.get("targets", [])
                if 'error="blockzero"' not in target.get("expr", "")
            ]
            for target in panel["targets"]:
                target["expr"] = target.get("expr", "").replace("rpc_getblockzero", "rpc_getblock")
                if 'error=~"blockzero"' in target["expr"]:
                    # Old uptime queries used blockzero and a fixed sample count.
                    # Non-archive endpoints use latest-block availability instead.
                    period = "30d" if "[30d]" in target["expr"] else "10d"
                    target["expr"] = (
                        '(1 - avg_over_time(rpc_error{wss=~"$wss",zone=~"$zone",'
                        'network=~"$network",error="getblock"}[' + period + '])) * 100'
                    )
            for transform in panel.get("transformations", []):
                rename = transform.get("options", {}).get("renameByName", {})
                for key, value in rename.items():
                    if isinstance(value, str):
                        rename[key] = value.replace("Block 0 time", "Latest block time").replace("Block 0", "Latest block")
            if panel.get("type") == "table" and panel.get("id") == 4:
                if any(t.get("refId") == "N" for t in panel["targets"]):
                    raise ValueError("Archive query refId N already in use")
                panel["targets"].append({
                    "refId": "N", "editorMode": "code", "format": "table",
                    "instant": True, "range": False,
                    "expr": 'last_over_time(rpc_archive_status{wss=~"$wss",zone=~"$zone",network=~"$network"}[30m])',
                    "datasource": panel["datasource"],
                })
                for transform in panel.get("transformations", []):
                    if transform.get("id") == "organize":
                        opts = transform.setdefault("options", {})
                        opts.setdefault("renameByName", {})["Value #N"] = "Archive"
                        opts.setdefault("indexByName", {})["Value #N"] = 14
                panel.setdefault("fieldConfig", {}).setdefault("overrides", []).append({
                    "matcher": {"id": "byName", "options": "Archive"},
                    "properties": [
                        {"id": "unit", "value": "none"},
                        {"id": "noValue", "value": "Not measured"},
                        {"id": "mappings", "value": [{"type": "value", "options": {
                            "1": {"text": "Archive", "color": "green"},
                            "0": {"text": "Pruned", "color": "blue"},
                            "-1": {"text": "Unknown", "color": "gray"},
                        }}]},
                    ],
                })
            walk(panel.get("panels", []))

    walk(dashboard.get("panels", []))
    rendered = json.dumps(dashboard)
    if "rpc_archive_status" not in rendered or "rpc_getblockzero" in rendered or 'blockzero' in rendered:
        raise ValueError("Dashboard migration incomplete; inspect source panel layout")
    return dashboard


if __name__ == "__main__":
    json.dump(prepare(json.load(sys.stdin)), sys.stdout, indent=2)
    print()
