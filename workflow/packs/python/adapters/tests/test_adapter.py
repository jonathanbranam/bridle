"""Adapter tests: run pytest in a scratch project against the fixture spec.

Skipped (not failed) when pytest-bdd or the bridle binary is unavailable, so
`just check` passes on machines without them.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

pytest.importorskip("pytest_bdd")

HERE = Path(__file__).parent
ADAPTERS = HERE.parent


def _bridle() -> str:
    found = os.environ.get("BRIDLE_BIN") or shutil.which("bridle")
    if not found:
        pytest.skip("bridle binary not available")
    # The binary must know `spec export` (older installs don't).
    probe = subprocess.run([found, "spec", "export", "--help"], capture_output=True)
    if probe.returncode != 0:
        pytest.skip("bridle binary lacks `spec export`")
    return found


STEPS = '''
from pytest_bdd import given, parsers, then, when

@given("a calculator", target_fixture="calc")
def _calc():
    return {}

@when(parsers.parse("it adds {a:d} and {b:d}"))
def _add(calc, a, b):
    calc["r"] = a + b

@when(parsers.parse("it multiplies {a:d} and {b:d}"))
def _mul(calc, a, b):
    calc["r"] = a * b

@then(parsers.parse("the result is {n:d}"))
def _res(calc, n):
    assert calc["r"] == n
'''

CONFTEST = 'pytest_plugins = ["bridle_specs"]\n'
TEST = "import bridle_specs\nbridle_specs.register(globals())\n"


@pytest.fixture
def project(tmp_path: Path) -> Path:
    _bridle()
    (tmp_path / "specs").mkdir()
    shutil.copy(HERE / "fixture-specs" / "calc.md", tmp_path / "specs" / "calc.md")
    (tmp_path / "conftest.py").write_text(CONFTEST + STEPS)
    (tmp_path / "test_specs.py").write_text(TEST)
    return tmp_path


def run(project: Path, *args: str, root: str = "specs") -> subprocess.CompletedProcess:
    env = {**os.environ, "PYTHONPATH": str(ADAPTERS), "BRIDLE_SPEC_ROOT": root}
    return subprocess.run(
        [sys.executable, "-m", "pytest", "-p", "no:cacheprovider", "-v", *args],
        cwd=project,
        env=env,
        capture_output=True,
        text=True,
    )


def test_registers_executable_scenarios_with_ids(project: Path) -> None:
    r = run(project)
    assert r.returncode == 0, r.stdout + r.stderr
    assert "test_s_aa01_add_two_numbers PASSED" in r.stdout
    assert "test_s_aa03_multiply_pairs[2-3-6] PASSED" in r.stdout
    assert "test_s_aa03_multiply_pairs[4-5-20] PASSED" in r.stdout
    assert "aa02" not in r.stdout  # non-executable: not collected


def test_tags_are_markers(project: Path) -> None:
    r = run(project, "-m", "slow")
    assert "2 passed" in r.stdout and "1 deselected" in r.stdout, r.stdout


def test_select_by_scenario_and_capability(project: Path) -> None:
    r = run(project, "--bridle-scenario", "s-aa01")
    assert "1 passed" in r.stdout, r.stdout
    r = run(project, "--bridle-spec", "calc")
    assert "3 passed" in r.stdout, r.stdout
    r = run(project, "--bridle-scenario", "s-zzzz")
    assert r.returncode != 0 and "no such executable scenario" in r.stdout + r.stderr


def test_export_refusal_fails_collection(project: Path) -> None:
    spec = project / "specs" / "calc.md"
    spec.write_text(spec.read_text().replace("**executable** @fast", "**exec** @fast"))
    r = run(project)
    assert r.returncode != 0
    assert "refused" in r.stdout + r.stderr


def test_missing_bridle_error_mentions_readme(tmp_path: Path) -> None:
    """Error when bridle is missing mentions the 'Running bridle in CI' README section."""
    pytest.importorskip("pytest_bdd")

    (tmp_path / "specs").mkdir()
    (tmp_path / "specs" / "test.md").write_text(
        "# Test\n\n## Requirement\n\nScenario: test\n  Given x\n"
    )
    (tmp_path / "conftest.py").write_text(CONFTEST + STEPS)
    (tmp_path / "test_specs.py").write_text(TEST)

    env = {**os.environ, "PYTHONPATH": str(ADAPTERS), "BRIDLE_BIN": "/nonexistent/bridle"}
    r = subprocess.run(
        [sys.executable, "-m", "pytest", "-p", "no:cacheprovider"],
        cwd=tmp_path,
        env=env,
        capture_output=True,
        text=True,
    )
    assert r.returncode != 0
    output = r.stdout + r.stderr
    assert "cannot run" in output
    assert "Running bridle in CI" in output
    assert "workflow/packs/python/README.md" in output
