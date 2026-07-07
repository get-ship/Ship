"""Spec tests for memento.py (stdlib unittest only).

These tests define the CLI contract and exercise real behavior against a
temp DB. MEMENTO_DB is set to a tempfile in setUp and removed in tearDown;
the real ~/.ship/memento.db is never touched.

Run: python3 -m unittest test_memento -v   (or: python3 test_memento.py)
Tests are expected to FAIL until memento.py is implemented.
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
MEMENTO = os.path.join(HERE, "memento.py")


class MementoTestBase(unittest.TestCase):
    def setUp(self):
        fd, self.db_path = tempfile.mkstemp(suffix=".db", prefix="memento_test_")
        os.close(fd)
        # Start from a clean (non-existent) file so bootstrap creates schema.
        os.remove(self.db_path)
        self.env = dict(os.environ)
        self.env["MEMENTO_DB"] = self.db_path

    def tearDown(self):
        for p in (self.db_path, self.db_path + "-wal", self.db_path + "-shm"):
            try:
                os.remove(p)
            except FileNotFoundError:
                pass

    def run_cli(self, *args):
        """Invoke memento.py as a subprocess. Returns CompletedProcess."""
        return subprocess.run(
            [sys.executable, MEMENTO, *args],
            env=self.env,
            capture_output=True,
            text=True,
        )

    def ok(self, *args):
        """Run, assert exit 0, return parsed JSON stdout."""
        cp = self.run_cli(*args)
        self.assertEqual(
            cp.returncode, 0,
            msg=f"expected exit 0 for {args!r}; stderr={cp.stderr!r} stdout={cp.stdout!r}",
        )
        return json.loads(cp.stdout)

    def record(self, **kw):
        """Helper: record a new/reinforced memento, return parsed JSON."""
        args = ["record", "--kind", kw.pop("kind", "operational"),
                "--title", kw.pop("title", "a title")]
        for flag in ("body", "domain", "layer", "conviction", "tags"):
            if flag in kw:
                args += [f"--{flag}", kw.pop(flag)]
        assert not kw, f"unexpected kwargs: {kw}"
        return self.ok(*args)


class TestRecord(MementoTestBase):
    def test_record_new_returns_not_reinforced_with_hex_id(self):
        out = self.record(kind="operational", title="Use WAL mode")
        self.assertFalse(out["reinforced"])
        self.assertIn("id", out)
        self.assertEqual(len(out["id"]), 32)
        int(out["id"], 16)  # valid hex
        self.assertNotIn("reinforced", {"confirmations"})  # no confirmations on new

    def test_record_reinforcement_bumps_confirmations_and_raises_conviction(self):
        first = self.record(kind="operational", title="Cache the connection",
                            conviction="low")
        self.assertFalse(first["reinforced"])

        # Same kind + case-insensitive title => reinforce, regardless of conviction arg.
        second = self.run_cli("record", "--kind", "operational",
                              "--title", "cache the CONNECTION", "--conviction", "low")
        self.assertEqual(second.returncode, 0, msg=second.stderr)
        sout = json.loads(second.stdout)
        self.assertTrue(sout["reinforced"])
        self.assertEqual(sout["id"], first["id"])
        self.assertEqual(sout["confirmations"], 2)

        # Conviction raised one notch low->medium; verify via get.
        rec = self.ok("get", first["id"])
        self.assertEqual(rec["conviction"], "medium")
        self.assertEqual(rec["confirmations"], 2)

        # Third reinforcement medium->high; fourth caps at high.
        self.ok("record", "--kind", "operational", "--title", "Cache the connection")
        self.ok("record", "--kind", "operational", "--title", "Cache the connection")
        rec = self.ok("get", first["id"])
        self.assertEqual(rec["conviction"], "high")
        self.assertEqual(rec["confirmations"], 4)

    def test_reinforcement_does_not_overwrite_body(self):
        first = self.record(kind="operational", title="keep body", body="ORIGINAL")
        self.run_cli("record", "--kind", "operational", "--title", "keep body",
                     "--body", "NEW BODY SHOULD BE IGNORED")
        rec = self.ok("get", first["id"])
        self.assertEqual(rec["body"], "ORIGINAL")

    def test_different_kind_same_title_is_new_row(self):
        a = self.record(kind="operational", title="shared title")
        b = self.record(kind="tooling", title="shared title")
        self.assertNotEqual(a["id"], b["id"])
        self.assertFalse(b["reinforced"])

    def test_unknown_kind_rejected(self):
        cp = self.run_cli("record", "--kind", "bogus", "--title", "x")
        self.assertEqual(cp.returncode, 1)
        # stderr should list allowed kinds.
        for k in ("correction", "operational", "mistake", "rejected",
                  "tooling", "dependency", "product"):
            self.assertIn(k, cp.stderr)

    def test_title_over_cap_rejected(self):
        cp = self.run_cli("record", "--kind", "operational", "--title", "x" * 121)
        self.assertEqual(cp.returncode, 1)
        self.assertIn("title", cp.stderr.lower())

    def test_title_at_cap_accepted(self):
        out = self.record(kind="operational", title="x" * 120)
        self.assertFalse(out["reinforced"])

    def test_body_over_cap_rejected(self):
        cp = self.run_cli("record", "--kind", "operational", "--title", "ok",
                          "--body", "b" * 601)
        self.assertEqual(cp.returncode, 1)
        self.assertIn("body", cp.stderr.lower())

    def test_empty_title_after_strip_rejected(self):
        cp = self.run_cli("record", "--kind", "operational", "--title", "   ")
        self.assertEqual(cp.returncode, 1)

    def test_invalid_conviction_argparse_exit_2(self):
        cp = self.run_cli("record", "--kind", "operational", "--title", "ok",
                          "--conviction", "supreme")
        self.assertEqual(cp.returncode, 2)

    def test_tags_parsed_dedup_and_strip(self):
        rec_id = self.record(kind="operational", title="tagged",
                            tags=" a , b ,a, ,c ")["id"]
        rec = self.ok("get", rec_id)
        self.assertEqual(rec["tags"], ["a", "b", "c"])

    def test_tags_absent_is_empty_array(self):
        rec_id = self.record(kind="operational", title="untagged")["id"]
        rec = self.ok("get", rec_id)
        self.assertEqual(rec["tags"], [])


class TestQuery(MementoTestBase):
    def test_empty_store_returns_empty_array(self):
        out = self.ok("query")
        self.assertEqual(out, [])

    def test_query_ordering_by_conviction_then_confirmations(self):
        # low conviction, but many confirmations
        self.record(kind="operational", title="low one", conviction="low")
        self.record(kind="operational", title="low one")  # confirmations=2, conv->medium
        # Reset: make a clearly-high entry and a clearly-low entry.
        high_id = self.record(kind="tooling", title="high entry", conviction="high")["id"]
        low_id = self.record(kind="tooling", title="zlow entry", conviction="low")["id"]

        out = self.ok("query", "--kind", "tooling")
        self.assertEqual(len(out), 2)
        # high conviction must come before low conviction.
        self.assertIn(high_id[:8], out[0])
        self.assertIn(low_id[:8], out[1])

    def test_query_oneliner_format(self):
        rec_id = self.record(kind="product", title="My Title", conviction="high")["id"]
        out = self.ok("query", "--kind", "product")
        self.assertEqual(len(out), 1)
        expected = f"{rec_id[:8]}  [product/highx1]  My Title"
        self.assertEqual(out[0], expected)

    def test_query_no_body_in_output(self):
        self.record(kind="operational", title="has body", body="SECRET BODY")
        out = self.ok("query", "--kind", "operational")
        self.assertNotIn("SECRET BODY", out[0])

    def test_query_status_filter(self):
        active_id = self.record(kind="mistake", title="active mistake")["id"]
        old_id = self.record(kind="mistake", title="old mistake")["id"]
        # supersede old -> becomes superseded
        self.ok("supersede", old_id, "--kind", "mistake", "--title", "new mistake")

        active = self.ok("query", "--kind", "mistake", "--status", "active")
        joined = "\n".join(active)
        self.assertIn(active_id[:8], joined)
        self.assertNotIn(old_id[:8], joined)

        superseded = self.ok("query", "--kind", "mistake", "--status", "superseded")
        self.assertEqual(len(superseded), 1)
        self.assertIn(old_id[:8], superseded[0])

    def test_query_limit_hard_cap_at_50(self):
        for i in range(55):
            self.record(kind="operational", title=f"entry number {i}")
        out = self.ok("query", "--limit", "100")
        self.assertEqual(len(out), 50)

    def test_query_limit_under_one_rejected(self):
        cp = self.run_cli("query", "--limit", "0")
        self.assertEqual(cp.returncode, 1)

    def test_query_min_conviction_inclusive_filter(self):
        self.record(kind="operational", title="low row", conviction="low")
        self.record(kind="operational", title="med row", conviction="medium")
        self.record(kind="operational", title="high row", conviction="high")
        out = self.ok("query", "--kind", "operational", "--min-conviction", "medium")
        joined = "\n".join(out)
        self.assertIn("med row", joined)
        self.assertIn("high row", joined)
        self.assertNotIn("low row", joined)

    def test_query_tag_exact_element_match(self):
        self.record(kind="operational", title="alpha tagged", tags="db,wal")
        self.record(kind="operational", title="beta tagged", tags="cache")
        out = self.ok("query", "--kind", "operational", "--tag", "wal")
        self.assertEqual(len(out), 1)
        self.assertIn("alpha tagged", out[0])
        # substring should NOT match
        out2 = self.ok("query", "--kind", "operational", "--tag", "wa")
        self.assertEqual(out2, [])

    def test_query_unknown_kind_rejected(self):
        cp = self.run_cli("query", "--kind", "nope")
        self.assertEqual(cp.returncode, 1)

    def test_query_no_matches_returns_empty(self):
        self.record(kind="operational", title="present")
        out = self.ok("query", "--kind", "product")
        self.assertEqual(out, [])


class TestGet(MementoTestBase):
    def test_get_returns_full_body(self):
        rec_id = self.record(kind="operational", title="full record",
                            body="the full body text", domain="L0",
                            layer="substrate", conviction="high",
                            tags="x,y")["id"]
        rec = self.ok("get", rec_id)
        self.assertEqual(rec["id"], rec_id)
        self.assertEqual(rec["title"], "full record")
        self.assertEqual(rec["body"], "the full body text")
        self.assertEqual(rec["domain"], "L0")
        self.assertEqual(rec["layer"], "substrate")
        self.assertEqual(rec["conviction"], "high")
        self.assertEqual(rec["tags"], ["x", "y"])
        self.assertEqual(rec["status"], "active")
        self.assertEqual(rec["confirmations"], 1)

    def test_get_pretty_printed(self):
        rec_id = self.record(kind="operational", title="pretty")["id"]
        cp = self.run_cli("get", rec_id)
        self.assertEqual(cp.returncode, 0)
        # indent=2 pretty JSON contains newlines and indentation.
        self.assertIn("\n", cp.stdout)
        self.assertIn('  "id"', cp.stdout)

    def test_get_missing_id_errors(self):
        cp = self.run_cli("get", "deadbeef" * 4)
        self.assertEqual(cp.returncode, 1)
        self.assertIn("memento not found", cp.stderr)

    def test_get_on_empty_store_errors(self):
        cp = self.run_cli("get", "abc123")
        self.assertEqual(cp.returncode, 1)
        self.assertIn("memento not found", cp.stderr)


class TestSupersede(MementoTestBase):
    def test_supersede_links_old_new_and_flips_status(self):
        old = self.record(kind="correction", title="old decision",
                          conviction="high")["id"]
        out = self.ok("supersede", old, "--kind", "correction",
                      "--title", "new decision", "--body", "replacement")
        self.assertEqual(out["superseded"], old)
        new = out["id"]
        self.assertNotEqual(new, old)

        old_rec = self.ok("get", old)
        new_rec = self.ok("get", new)
        self.assertEqual(old_rec["status"], "superseded")
        self.assertEqual(old_rec["superseded_by"], new)
        self.assertEqual(new_rec["status"], "active")
        self.assertEqual(new_rec["supersedes"], old)
        self.assertEqual(new_rec["confirmations"], 1)

    def test_supersede_new_row_defaults_conviction_medium(self):
        old = self.record(kind="correction", title="orig")["id"]
        out = self.ok("supersede", old, "--kind", "correction", "--title", "next")
        new_rec = self.ok("get", out["id"])
        self.assertEqual(new_rec["conviction"], "medium")

    def test_supersede_does_not_dedup(self):
        # Active row with same kind+title exists; supersede still inserts new row.
        a = self.record(kind="operational", title="topic")["id"]
        out = self.ok("supersede", a, "--kind", "operational", "--title", "topic")
        self.assertNotEqual(out["id"], a)
        new_rec = self.ok("get", out["id"])
        self.assertEqual(new_rec["confirmations"], 1)
        self.assertFalse("reinforced" in out)

    def test_supersede_already_superseded_errors(self):
        old = self.record(kind="correction", title="first")["id"]
        out = self.ok("supersede", old, "--kind", "correction", "--title", "second")
        # Now try to supersede the already-superseded old row again.
        cp = self.run_cli("supersede", old, "--kind", "correction",
                          "--title", "third")
        self.assertEqual(cp.returncode, 1)
        self.assertIn(old, cp.stderr)

    def test_supersede_missing_old_id_errors(self):
        cp = self.run_cli("supersede", "f" * 32, "--kind", "correction",
                          "--title", "x")
        self.assertEqual(cp.returncode, 1)

    def test_supersede_validates_kind_and_caps(self):
        old = self.record(kind="correction", title="base")["id"]
        cp = self.run_cli("supersede", old, "--kind", "bogus", "--title", "x")
        self.assertEqual(cp.returncode, 1)
        # old row must remain active (no partial write).
        self.assertEqual(self.ok("get", old)["status"], "active")

        cp = self.run_cli("supersede", old, "--kind", "correction",
                          "--title", "y" * 121)
        self.assertEqual(cp.returncode, 1)
        self.assertEqual(self.ok("get", old)["status"], "active")


class TestAudit(MementoTestBase):
    def test_empty_store_returns_empty_array(self):
        out = self.ok("audit")
        self.assertEqual(out, [])

    def test_audit_ranks_and_flags_strong(self):
        # STRONG candidate: confirmations>=3 AND conviction=='high'.
        strong = self.record(kind="operational", title="strong topic",
                            conviction="high")["id"]
        # bump confirmations to 3 (stays high)
        self.record(kind="operational", title="strong topic")
        self.record(kind="operational", title="strong topic")

        weak = self.record(kind="tooling", title="weak topic",
                         conviction="low")["id"]

        out = self.ok("audit")
        self.assertTrue(all(isinstance(o, dict) for o in out))
        labels = {o["id"]: o["label"] for o in out}
        self.assertEqual(labels[strong], "STRONG candidate")
        self.assertNotEqual(labels[weak], "STRONG candidate")

        # Ranked by confirmations desc first => strong (3) before weak (1).
        ids = [o["id"] for o in out]
        self.assertLess(ids.index(strong), ids.index(weak))

        # Objects carry annotation fields.
        s = next(o for o in out if o["id"] == strong)
        self.assertEqual(s["kind"], "operational")
        self.assertEqual(s["conviction"], "high")
        self.assertEqual(s["confirmations"], 3)
        self.assertIn("title", s)

    def test_audit_only_active_rows(self):
        old = self.record(kind="correction", title="to be replaced")["id"]
        self.ok("supersede", old, "--kind", "correction", "--title", "replacement")
        out = self.ok("audit")
        ids = [o["id"] for o in out]
        self.assertNotIn(old, ids)

    def test_audit_limit_clamped_at_50(self):
        for i in range(55):
            self.record(kind="operational", title=f"audit entry {i}")
        out = self.ok("audit", "--limit", "100")
        self.assertEqual(len(out), 50)

    def test_audit_limit_under_one_rejected(self):
        cp = self.run_cli("audit", "--limit", "0")
        self.assertEqual(cp.returncode, 1)


class TestInfrastructureErrors(MementoTestBase):
    def test_unmakeable_db_parent_is_clean_error_not_traceback(self):
        # Point MEMENTO_DB under a path whose parent cannot be created: the
        # parent component is a regular file, so makedirs fails with a
        # non-directory OSError. The CLI must surface a clean one-line error.
        blocker = self.db_path  # a regular file (created in setUp)
        with open(blocker, "w") as f:
            f.write("not a directory")
        self.env["MEMENTO_DB"] = os.path.join(blocker, "sub", "x.db")

        cp = self.run_cli("record", "--kind", "operational", "--title", "x")
        self.assertEqual(cp.returncode, 1, msg=f"stderr={cp.stderr!r}")
        self.assertTrue(cp.stderr.strip(), msg="expected non-empty stderr")
        self.assertNotIn("Traceback", cp.stderr)


if __name__ == "__main__":
    unittest.main()
