#!/usr/bin/env python3
"""update_readme.py — Автоматическое обновление README.md проекта SOFIA.

Task 1: Парсинг исходного кода (функции parse_project, parse_cargo, find_rs_files).
Будущие задачи добавят: CLI, LLM-интеграцию, сборку README, валидацию, git hook.
"""

import os
import re
import sys
import glob
import json
import shutil
import logging
import argparse
import subprocess
import urllib.request
import urllib.error
from datetime import datetime
from pathlib import Path
from typing import List, Dict, Optional, Tuple

PROJECT_ROOT = Path(__file__).parent.parent.resolve()
README_PATH = PROJECT_ROOT / "README.md"
BACKUP_DIR = PROJECT_ROOT / ".readme_backups"
LOG_FILE = PROJECT_ROOT / ".readme_sync.log"
SRC_DIR = PROJECT_ROOT / "src"

# Маркеры для секций README
MARKERS = {
    "structure": ("<!-- BEGIN structure -->", "<!-- END structure -->"),
    "architecture": ("<!-- BEGIN architecture -->", "<!-- END architecture -->"),
    "modules": ("<!-- BEGIN modules -->", "<!-- END modules -->"),
    "stats": ("<!-- BEGIN stats -->", "<!-- END stats -->"),
    "changes": ("<!-- BEGIN changes -->", "<!-- END changes -->"),
}

# 9Router
NINEROUTER_URL = os.environ.get("NINEROUTER_URL", "")
NINEROUTER_KEY = os.environ.get("NINEROUTER_KEY", "")
LLM_MODEL = os.environ.get("LLM_MODEL", "openai/gpt-4o")


def find_rs_files() -> List[Path]:
    """Найти все .rs файлы в src/ (рекурсивно)."""
    return sorted(SRC_DIR.rglob("*.rs"))


def parse_public_symbols(filepath: Path) -> Dict:
    """Извлечь публичные символы из .rs файла."""
    content = filepath.read_text(encoding="utf-8")
    rel_path = filepath.relative_to(PROJECT_ROOT)

    modules = re.findall(r'^pub mod (\w+);', content, re.MULTILINE)
    structs = re.findall(r'^pub struct (\w+)', content, re.MULTILINE)
    enums = re.findall(r'^pub enum (\w+)', content, re.MULTILINE)
    fns = re.findall(r'^pub fn (\w+)', content, re.MULTILINE)
    impls = re.findall(r'^impl(?:<[^>]+>)? (\w+)', content, re.MULTILINE)
    has_tests = "mod tests" in content

    return {
        "path": str(rel_path),
        "modules": modules,
        "structs": structs,
        "enums": enums,
        "functions": fns,
        "impls": impls,
        "has_tests": has_tests,
        "loc": len(content.splitlines()),
    }


def parse_project() -> Dict:
    """Парсинг всего проекта — возвращает структурированные данные."""
    files = find_rs_files()
    symbols = [parse_public_symbols(f) for f in files]

    all_modules = []
    all_structs = []
    all_enums = []
    all_fns = []
    total_loc = 0
    test_modules = 0

    for s in symbols:
        all_modules.extend(s["modules"])
        all_structs.extend(s["structs"])
        all_enums.extend(s["enums"])
        all_fns.extend(s["functions"])
        total_loc += s["loc"]
        if s["has_tests"]:
            test_modules += 1

    return {
        "files": symbols,
        "modules": sorted(set(all_modules)),
        "structs": sorted(set(all_structs)),
        "enums": sorted(set(all_enums)),
        "functions": sorted(set(all_fns)),
        "total_loc": total_loc,
        "total_files": len(files),
        "test_modules": test_modules,
    }


def parse_cargo() -> Dict:
    """Извлечь зависимости и метаданные из Cargo.toml."""
    cargo_path = PROJECT_ROOT / "Cargo.toml"
    if not cargo_path.exists():
        return {"name": "unknown", "deps": []}

    content = cargo_path.read_text(encoding="utf-8")
    name_match = re.search(r'^name\s*=\s*"(.+?)"', content, re.MULTILINE)
    name = name_match.group(1) if name_match else "unknown"

    # Парсим только секцию [dependencies]
    deps = []
    dep_match = re.search(
        r'^\[dependencies\](.*?)(?:^\[|\Z)', content, re.MULTILINE | re.DOTALL
    )
    if dep_match:
        dep_section = dep_match.group(1)
        deps = re.findall(r'^\s*(\w[\w.-]*)\s*=', dep_section, re.MULTILINE)

    return {"name": name, "deps": deps}


def llm_generate(prompt: str, system_prompt: str = None) -> Optional[str]:
    """Отправить запрос в 9Router chat completion. Вернуть текст ответа или None."""
    if not NINEROUTER_URL:
        return None

    url = f"{NINEROUTER_URL.rstrip('/')}/v1/chat/completions"

    messages = []
    if system_prompt:
        messages.append({"role": "system", "content": system_prompt})
    messages.append({"role": "user", "content": prompt})

    payload = json.dumps({
        "model": LLM_MODEL,
        "messages": messages,
        "max_tokens": 2000,
        "temperature": 0.3,
    }).encode("utf-8")

    headers = {"Content-Type": "application/json"}
    if NINEROUTER_KEY:
        headers["Authorization"] = f"Bearer {NINEROUTER_KEY}"

    req = urllib.request.Request(url, data=payload, headers=headers, method="POST")

    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            result = json.loads(resp.read().decode("utf-8"))
            return result["choices"][0]["message"]["content"]
    except (urllib.error.URLError, json.JSONDecodeError, KeyError, TimeoutError) as e:
        logging.warning("LLM request failed: %s", e)
        return None


SYSTEM_PROMPT = """Ты — технический писатель для проекта SOFIA, интерпретатора языка программирования на Rust.
Опиши осмысленно назначение каждого модуля. Используй русский язык.
Формат: кратко, 1-2 предложения на модуль. Без лишних деталей."""


def generate_module_descriptions(project_data: Dict) -> Dict[str, str]:
    """Сгенерировать описания модулей через LLM. Вернуть {имя_модуля: описание}."""
    module_list = "\n".join(f"- {m}" for m in project_data["modules"])

    prompt = f"""Перечисли назначение каждого модуля в проекте SOFIA:

{module_list}

Для каждого модуля дай описание в формате:
- module_name: краткое описание (1-2 предложения)
"""

    response = llm_generate(prompt, SYSTEM_PROMPT)
    if not response:
        return {}

    descriptions = {}
    for line in response.split("\n"):
        match = re.match(r'^\s*[-*]\s*(\w+):\s*(.+)', line)
        if match:
            descriptions[match.group(1)] = match.group(2).strip()

    return descriptions


def generate_changes_summary() -> Optional[str]:
    """Сгенерировать описание последних изменений через LLM по git diff."""
    try:
        diff = subprocess.run(
            ["git", "diff", "--stat", "HEAD~3..HEAD"],
            capture_output=True, text=True, cwd=PROJECT_ROOT, timeout=10,
        )
        log = subprocess.run(
            ["git", "log", "--oneline", "-5"],
            capture_output=True, text=True, cwd=PROJECT_ROOT, timeout=10,
        )
        if not diff.stdout and not log.stdout:
            return None
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return None

    prompt = f"""На основе следующих данных о коммитах напиши краткую сводку изменений (2-4 предложения) на русском языке:

Последние коммиты:
{log.stdout}

Изменения:
{diff.stdout}

Сводка:"""

    return llm_generate(prompt)
