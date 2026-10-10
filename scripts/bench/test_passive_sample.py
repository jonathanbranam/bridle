"""Unit tests for passive-sample.py: parsing, classification and the start gate. Never samples."""
import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("ps_mod", Path(__file__).with_name("passive-sample.py"))
ps = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ps)

PS = """\
  10     1   1:02:03.50  5000 /usr/local/bin/bridle serve
  20    10      0:01.25   300 /bin/sh -c ps
  30    10     12:00.00  9000 /Users/x/.local/bin/claude --print
  40     1      0:05.00   700 /opt/claude --foo
garbage line
"""


class Parse(unittest.TestCase):
    def test_cpu(self):
        self.assertEqual(ps.parse_cpu("0:01.25"), 1.25)
        self.assertEqual(ps.parse_cpu("1:02:03.50"), 3723.5)
        self.assertEqual(ps.parse_cpu("2-00:00:10"), 172810)

    def test_process_columns_separate_children_from_claude(self):
        c = ps.process_columns(ps.parse_ps(PS), 10)
        self.assertEqual((c["daemon_cpu_s"], c["daemon_rss_kb"]), (3723.5, 5000))
        self.assertEqual((c["children_n"], c["children_rss_kb"]), (1, 300))
        self.assertEqual((c["claude_n"], c["claude_rss_kb"]), (2, 9700))

    def test_daemon_down_leaves_daemon_columns_blank(self):
        c = ps.process_columns(ps.parse_ps(PS), 999)
        self.assertNotIn("daemon_cpu_s", c)
        self.assertEqual(c["claude_n"], 2)

    def test_memory(self):
        self.assertEqual(ps.parse_swap_used_mb("total = 3072.00M  used = 1350.50M  free = 1M"), 1350.5)
        vm = "Mach Virtual Memory Statistics: (page size of 4096 bytes)\nPages occupied by compressor:   256.\n"
        self.assertEqual(ps.parse_compressed_kb(vm), 1024)
        self.assertIsNone(ps.parse_compressed_kb("nope"))

    def test_agent_columns(self):
        c = ps.agent_columns({"agents_by_state": {"working": 2, "idle": 1, "weird": 3}})
        self.assertEqual((c["agents_working"], c["agents_stopped"], c["agents_other"]), (2, 0, 3))
        self.assertEqual(ps.agent_columns(None), {})


class Classify(unittest.TestCase):
    def test_idle_busy_unknown(self):
        self.assertEqual(ps.classify([{"agents_working": 0}, {"agents_working": 0}]), "idle")
        self.assertEqual(ps.classify([{"agents_working": 0}, {"agents_working": 1}]), "busy")
        self.assertEqual(ps.classify([{}]), "unknown")
        self.assertEqual(ps.load_context([{"agents_working": 1}, {"agents_working": 3}])["working_mean"], 2.0)


class StartGate(unittest.TestCase):
    def git(self, d, *a):
        subprocess.run(["git", "-C", str(d), "-c", "user.name=t", "-c", "user.email=t@t", *a],
                       check=True, capture_output=True)

    def repo(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        origin, clone = Path(tmp.name, "origin"), Path(tmp.name, "clone")
        origin.mkdir()
        self.git(origin, "init", "-q", "-b", "main")
        self.git(origin, "commit", "-q", "--allow-empty", "-m", "a")
        subprocess.run(["git", "clone", "-q", str(origin), str(clone)], check=True, capture_output=True)
        return origin, clone

    def test_gate(self):
        origin, clone = self.repo()
        self.assertIsNone(ps.preflight(clone))
        (clone / "f").write_text("x")
        self.assertIn("uncommitted", ps.preflight(clone))
        (clone / "f").unlink()
        self.git(origin, "commit", "-q", "--allow-empty", "-m", "b")
        self.git(clone, "fetch", "-q")
        self.assertIn("behind", ps.preflight(clone))
        self.git(clone, "checkout", "-q", "-b", "other")
        self.assertIn("not main", ps.preflight(clone))


if __name__ == "__main__":
    unittest.main()
