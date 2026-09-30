"""pytest plugin: bridle specs as pytest-bdd scenarios.

Replaces data-contracts' spec-to-feature.py + run-specs.py + test_specs_bdd.py.
At collection time `register(globals())` runs `bridle spec export --format json`
and binds one pytest-bdd scenario per EXECUTABLE scenario to the step
definitions visible to the calling test module. Nothing generated is committed:
the Gherkin pytest-bdd parses is written to a temp dir that is removed at exit.

Test names carry the scenario id (`test_s_b310_<title>`), the scenario's tags
become pytest markers, and a Scenario Outline's Examples table becomes
parametrization (pytest-bdd's own). Each requirement is a Gherkin `Rule:`.

Options (also see README.md):
    --bridle-spec-root DIR   specs directory (env BRIDLE_SPEC_ROOT, default design/specs)
    --bridle-spec CAP        only this capability (repeatable)
    --bridle-scenario ID     only this scenario id, e.g. s-b310 (repeatable)
    env BRIDLE_BIN           the bridle binary (default: `bridle` on PATH)

Needs pytest-bdd >= 8 (Rule support).
"""

from __future__ import annotations

import atexit
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path

import pytest
from pytest_bdd import scenario

DEFAULT_ROOT = "design/specs"

_config: pytest.Config | None = None


class BridleSpecError(Exception):
    """`bridle spec export` refused, or the selection matched nothing."""


def pytest_addoption(parser: pytest.Parser) -> None:
    group = parser.getgroup("bridle-specs")
    group.addoption(
        "--bridle-spec-root",
        default=None,
        help=f"bridle specs directory (env BRIDLE_SPEC_ROOT, default {DEFAULT_ROOT})",
    )
    group.addoption(
        "--bridle-spec",
        action="append",
        default=[],
        metavar="CAPABILITY",
        help="only scenarios of this capability (repeatable)",
    )
    group.addoption(
        "--bridle-scenario",
        action="append",
        default=[],
        metavar="ID",
        help="only this scenario id, e.g. s-b310 (repeatable)",
    )


def pytest_configure(config: pytest.Config) -> None:
    global _config
    _config = config


def export(root: str, cwd: Path | None = None) -> dict:
    """The parsed `bridle spec export --format json` document."""
    binary = os.environ.get("BRIDLE_BIN") or "bridle"
    try:
        proc = subprocess.run(
            [binary, "spec", "export", "--format", "json", "--root", root],
            capture_output=True,
            text=True,
            cwd=cwd,
            check=False,
        )
    except OSError as exc:
        raise BridleSpecError(
            f"cannot run {binary!r} (set BRIDLE_BIN): {exc}\n"
            "See 'Running bridle in CI' in workflow/packs/python/README.md for CI setup"
        ) from exc
    if proc.returncode != 0:
        raise BridleSpecError(
            f"`bridle spec export` refused (exit {proc.returncode}):\n"
            f"{proc.stderr.strip()}\n{proc.stdout.strip()}".rstrip()
        )
    return json.loads(proc.stdout)


def _slug(text: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", text.lower()).strip("_")


def _cell(text: str) -> str:
    return text.replace("\\", "\\\\").replace("|", "\\|")


def _select(doc: dict, capabilities: list[str], ids: list[str]) -> list[dict]:
    """Specs narrowed to the selection, each requirement holding only its
    selected executable scenarios. Selecting nothing that exists is an error:
    a typo'd id must not turn into a green empty run."""
    known_caps = {s["capability"] for s in doc["specs"]}
    known_ids = {
        sc["id"]
        for s in doc["specs"]
        for r in s["requirements"]
        for sc in r["scenarios"]
        if sc["executable"] and sc["id"]
    }
    for cap in capabilities:
        if cap not in known_caps:
            raise BridleSpecError(
                f"--bridle-spec {cap!r}: no such capability (have: {', '.join(sorted(known_caps))})"
            )
    for sid in ids:
        if sid not in known_ids:
            raise BridleSpecError(f"--bridle-scenario {sid!r}: no such executable scenario")
    out = []
    for spec in doc["specs"]:
        if capabilities and spec["capability"] not in capabilities:
            continue
        reqs = []
        for req in spec["requirements"]:
            scs = [
                sc
                for sc in req["scenarios"]
                if sc["executable"] and (not ids or sc["id"] in ids)
            ]
            # Keep an empty Rule only in an unfiltered run, so the gap shows.
            if scs or not ids:
                reqs.append({**req, "scenarios": scs})
        out.append({**spec, "requirements": reqs})
    return out


def _scenario_name(sc: dict) -> str:
    return f"{sc['id']}: {sc['title']}" if sc["id"] else sc["title"]


def _feature_text(spec: dict) -> str:
    lines = [f"Feature: {spec['capability']}", ""]
    for req in spec["requirements"]:
        lines += [f"  Rule: {req['title']}", ""]
        for sc in req["scenarios"]:
            tags = list(sc["tags"])
            if sc["id"]:
                tags.append(sc["id"])
            if tags:
                lines.append("    " + " ".join(f"@{t}" for t in tags))
            outline = sc["examples"] is not None
            lines.append(f"    {'Scenario Outline' if outline else 'Scenario'}: {_scenario_name(sc)}")
            lines += [f"      {st['keyword']} {st['text']}" for st in sc["steps"]]
            if outline:
                ex = sc["examples"]
                lines += ["", "      Examples:"]
                for row in [ex["header"], *ex["rows"]]:
                    lines.append("        | " + " | ".join(_cell(c) for c in row) + " |")
            lines.append("")
    return "\n".join(lines)


def register(module_globals: dict) -> int:
    """Bind the selected executable scenarios into a test module's namespace.
    Returns how many scenarios (not parametrized cases) were registered."""
    if _config is None:
        raise BridleSpecError("the bridle_specs plugin is not loaded (pytest_plugins)")
    root = (
        _config.getoption("--bridle-spec-root")
        or os.environ.get("BRIDLE_SPEC_ROOT")
        or DEFAULT_ROOT
    )
    doc = export(root, cwd=_config.rootpath)
    specs = _select(
        doc, _config.getoption("--bridle-spec"), _config.getoption("--bridle-scenario")
    )

    tmp = Path(tempfile.mkdtemp(prefix="bridle-specs-"))
    atexit.register(shutil.rmtree, tmp, ignore_errors=True)
    count = 0
    for spec in specs:
        scenarios = [sc for r in spec["requirements"] for sc in r["scenarios"]]
        if not scenarios:
            continue
        path = tmp / f"{spec['capability']}.feature"
        path.write_text(_feature_text(spec), encoding="utf-8")
        for sc in scenarios:
            for tag in sc["tags"]:
                _config.addinivalue_line("markers", f"{tag}: bridle spec tag")
            def _test() -> None:
                pass

            base = "test_" + _slug(_scenario_name(sc))
            name, n = base, 0
            while name in module_globals:
                n += 1
                name = f"{base}_{n}"
            _test.__name__ = name
            module_globals[name] = scenario(str(path), _scenario_name(sc))(_test)
            count += 1
    return count
