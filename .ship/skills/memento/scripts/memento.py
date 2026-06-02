#!/usr/bin/env python3
"""memento — terse decision-record store (stdlib only).

Subcommands: record, query, get, supersede, audit.
JSON to stdout; errors to stderr with nonzero exit. SQLite (WAL).
"""

import argparse
import json
import os
import sqlite3
import sys
import uuid
from datetime import datetime, timezone

# Tunable (skill var): the allowed record kinds.
KINDS = {"correction", "operational", "mistake", "rejected",
         "tooling", "dependency", "product"}

CONVICTIONS = ["low", "medium", "high"]
WEIGHT = {"low": 1, "medium": 2, "high": 3}

TITLE_CAP = 120
BODY_CAP = 600
HARD_LIMIT = 50

DDL = """
CREATE TABLE IF NOT EXISTS memento (
  id           TEXT PRIMARY KEY,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  kind         TEXT NOT NULL,
  status       TEXT NOT NULL DEFAULT 'active',
  domain       TEXT,
  layer        TEXT,
  title        TEXT NOT NULL,
  body         TEXT,
  conviction   TEXT NOT NULL DEFAULT 'medium',
  tags         TEXT NOT NULL DEFAULT '[]',
  confirmations INTEGER NOT NULL DEFAULT 1,
  supersedes   TEXT,
  superseded_by TEXT
);
CREATE INDEX IF NOT EXISTS idx_memento_dedup ON memento (kind, status);
CREATE INDEX IF NOT EXISTS idx_memento_status_conviction
  ON memento (status, conviction, confirmations, updated_at);
"""


def die(msg):
    print(msg, file=sys.stderr)
    sys.exit(1)


def now_iso():
    return datetime.now(timezone.utc).isoformat()


def db_path():
    return os.environ.get("MEMENTO_DB") or os.path.expanduser("~/.ship/memento.db")


def connect():
    path = db_path()
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    conn = sqlite3.connect(path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL")
    conn.executescript(DDL)
    return conn


def parse_tags(raw):
    if not raw:
        return []
    out = []
    for part in raw.split(","):
        t = part.strip()
        if t and t not in out:
            out.append(t)
    return out


def validate_kind(kind):
    if kind not in KINDS:
        die("unknown kind: %s; allowed: %s" % (kind, ", ".join(sorted(KINDS))))


def validate_content(title, body):
    title = title.strip()
    if not title:
        die("title is empty after strip")
    if len(title) > TITLE_CAP:
        die("title exceeds %d chars (cap)" % TITLE_CAP)
    if body is not None and len(body) > BODY_CAP:
        die("body exceeds %d chars (cap)" % BODY_CAP)
    return title


def row_to_dict(row):
    d = dict(row)
    d["tags"] = json.loads(d.get("tags") or "[]")
    return d


def cmd_record(conn, a):
    validate_kind(a.kind)
    title = validate_content(a.title, a.body)
    tags = parse_tags(a.tags)

    match = conn.execute(
        "SELECT id, conviction, confirmations FROM memento "
        "WHERE kind=? AND lower(title)=lower(?) AND status='active' "
        "ORDER BY updated_at DESC LIMIT 1",
        (a.kind, title),
    ).fetchone()

    if match:
        idx = min(CONVICTIONS.index(match["conviction"]) + 1, len(CONVICTIONS) - 1)
        conf = match["confirmations"] + 1
        conn.execute(
            "UPDATE memento SET confirmations=?, conviction=?, updated_at=? WHERE id=?",
            (conf, CONVICTIONS[idx], now_iso(), match["id"]),
        )
        conn.commit()
        print(json.dumps({"reinforced": True, "id": match["id"], "confirmations": conf}))
        return

    mid = uuid.uuid4().hex
    ts = now_iso()
    conn.execute(
        "INSERT INTO memento (id, created_at, updated_at, kind, status, domain, layer, "
        "title, body, conviction, tags, confirmations) "
        "VALUES (?,?,?,?,'active',?,?,?,?,?,?,1)",
        (mid, ts, ts, a.kind, a.domain, a.layer, title, a.body,
         a.conviction, json.dumps(tags)),
    )
    conn.commit()
    print(json.dumps({"reinforced": False, "id": mid}))


def cmd_query(conn, a):
    if a.kind:
        validate_kind(a.kind)
    if a.limit < 1:
        die("--limit must be a positive integer")
    limit = min(a.limit, HARD_LIMIT)

    sql = "SELECT * FROM memento WHERE status=?"
    params = [a.status]
    if a.kind:
        sql += " AND kind=?"
        params.append(a.kind)
    if a.domain:
        sql += " AND domain=?"
        params.append(a.domain)
    rows = [row_to_dict(r) for r in conn.execute(sql, params).fetchall()]

    if a.min_conviction:
        floor = WEIGHT[a.min_conviction]
        rows = [r for r in rows if WEIGHT[r["conviction"]] >= floor]
    if a.tag:
        rows = [r for r in rows if a.tag in r["tags"]]

    rows.sort(key=lambda r: (WEIGHT[r["conviction"]], r["confirmations"], r["updated_at"]),
              reverse=True)
    rows = rows[:limit]

    out = ["%s  [%s/%sx%d]  %s" % (r["id"][:8], r["kind"], r["conviction"],
                                   r["confirmations"], r["title"]) for r in rows]
    print(json.dumps(out))


def cmd_get(conn, a):
    row = conn.execute("SELECT * FROM memento WHERE id=?", (a.id,)).fetchone()
    if not row:
        die("memento not found: %s" % a.id)
    print(json.dumps(row_to_dict(row), indent=2))


def cmd_supersede(conn, a):
    validate_kind(a.kind)
    title = validate_content(a.title, a.body)
    tags = parse_tags(a.tags)

    old = conn.execute("SELECT id, status FROM memento WHERE id=?",
                       (a.old_id,)).fetchone()
    if not old:
        die("memento not found: %s" % a.old_id)
    if old["status"] == "superseded":
        die("memento already superseded: %s" % a.old_id)

    mid = uuid.uuid4().hex
    ts = now_iso()
    conn.execute(
        "INSERT INTO memento (id, created_at, updated_at, kind, status, domain, layer, "
        "title, body, conviction, tags, confirmations, supersedes) "
        "VALUES (?,?,?,?,'active',?,?,?,?,?,?,1,?)",
        (mid, ts, ts, a.kind, a.domain, a.layer, title, a.body,
         a.conviction, json.dumps(tags), a.old_id),
    )
    conn.execute("UPDATE memento SET status='superseded', superseded_by=? WHERE id=?",
                 (mid, a.old_id))
    conn.commit()
    print(json.dumps({"superseded": a.old_id, "id": mid}))


def label_for(conviction, confirmations):
    if confirmations >= 3 and conviction == "high":
        return "STRONG candidate"
    if conviction == "high" or confirmations >= 3:
        return "candidate"
    return "weak"


def cmd_audit(conn, a):
    if a.limit < 1:
        die("--limit must be a positive integer")
    limit = min(a.limit, HARD_LIMIT)
    rows = [row_to_dict(r) for r in
            conn.execute("SELECT * FROM memento WHERE status='active'").fetchall()]
    rows.sort(key=lambda r: (r["confirmations"], WEIGHT[r["conviction"]], r["updated_at"]),
              reverse=True)
    out = [{"id": r["id"], "kind": r["kind"], "conviction": r["conviction"],
            "confirmations": r["confirmations"], "title": r["title"],
            "label": label_for(r["conviction"], r["confirmations"])}
           for r in rows[:limit]]
    print(json.dumps(out))


def add_record_opts(p):
    p.add_argument("--kind", required=True)
    p.add_argument("--title", required=True)
    p.add_argument("--body")
    p.add_argument("--domain")
    p.add_argument("--layer")
    p.add_argument("--conviction", choices=CONVICTIONS, default="medium")
    p.add_argument("--tags")


def main():
    parser = argparse.ArgumentParser(prog="memento")
    sub = parser.add_subparsers(dest="cmd", required=True)

    pr = sub.add_parser("record")
    add_record_opts(pr)

    pq = sub.add_parser("query")
    pq.add_argument("--kind")
    pq.add_argument("--domain")
    pq.add_argument("--status", choices=["active", "superseded"], default="active")
    pq.add_argument("--tag")
    pq.add_argument("--min-conviction", dest="min_conviction", choices=CONVICTIONS)
    pq.add_argument("--limit", type=int, default=20)

    pg = sub.add_parser("get")
    pg.add_argument("id")

    ps = sub.add_parser("supersede")
    ps.add_argument("old_id")
    add_record_opts(ps)

    pa = sub.add_parser("audit")
    pa.add_argument("--limit", type=int, default=30)

    args = parser.parse_args()
    conn = None
    try:
        conn = connect()
        {"record": cmd_record, "query": cmd_query, "get": cmd_get,
         "supersede": cmd_supersede, "audit": cmd_audit}[args.cmd](conn, args)
    except (sqlite3.Error, OSError) as e:
        die("infrastructure error: %s" % e)
    finally:
        if conn is not None:
            conn.close()


if __name__ == "__main__":
    main()
