# README Autosync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Создать систему автосинхронизации README.md, которая после каждого коммита обновляет документацию проекта на основе анализа исходного кода, с LLM-генерацией осмысленных описаний и механизмом rollback.

**Architecture:** Python-скрипт (`scripts/update_readme.py`) парсит .rs файлы проекта, при наличии 9Router gateway отправляет данные в LLM для генерации описаний, собирает README по маркерам `<!-- BEGIN -->`/`<!-- END -->`, валидирует результат и сохраняет. Git hook post-commit запускает скрипт автоматически. Makefile предоставляет цели для ручного запуска и rollback.

**Tech Stack:** Python 3.8+, 9Router API (OpenAI-compatible), git hooks, Makefile

---

### Task 1: Python-скрипт — парсинг исходного кода

**Files:**
- Create: `scripts/update_readme.py`

Этот task реализует функции парсинга Rust-кода: извлечение модулей, публичных структур, enum, функций, impl-блоков из .rs файлов.

- [ ] **Step 1: Создать структуру скрипта и функцию поиска .rs файлов**

Добавить в начало `scripts/update_readme.py`:

```python
#!/usr/bin/env python3
"""update_readme.py — Автоматическое обновление README.md проекта SOFIA.

Режимы:
  --trigger=post-commit  : запуск из git hook
  --trigger=manual       : ручной запуск
  --rollback             : восстановить предыдущую версию README
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
```

- [ ] **Step 2: Реализовать парсинг публичных символов из .rs файлов**

Добавить после `find_rs_files`:

```python
def parse_public_symbols(filepath: Path) -> Dict:
    """Извлечь публичные символы из .rs файла."""
    content = filepath.read_text(encoding="utf-8")
    rel_path = filepath.relative_to(PROJECT_ROOT)

    modules = re.findall(r'^pub mod (\w+);', content, re.MULTILINE)
    structs = re.findall(r'^pub struct (\w+)', content, re.MULTILINE)
    enums = re.findall(r'^pub enum (\w+)', content, re.MULTILINE)
    fns = re.findall(r'^pub fn (\w+)', content, re.MULTILINE)
    impls = re.findall(r'^pub? impl(?:<[^>]+>)? (\w+)', content, re.MULTILINE)
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
```

- [ ] **Step 3: Реализовать чтение Cargo.toml**

```python
def parse_cargo() -> Dict:
    """Извлечь зависимости и метаданные из Cargo.toml."""
    cargo_path = PROJECT_ROOT / "Cargo.toml"
    if not cargo_path.exists():
        return {"name": "unknown", "deps": []}

    content = cargo_path.read_text(encoding="utf-8")
    name_match = re.search(r'^name\s*=\s*"(.+)"', content, re.MULTILINE)
    name = name_match.group(1) if name_match else "unknown"

    deps = re.findall(r'^([a-zA-Z0-9_-]+)\s*=', content, re.MULTILINE)
    # Исключаем секции [package], [lib], [[bin]]
    deps = [d for d in deps if d not in ("package", "lib")]

    return {"name": name, "deps": deps}
```

- [ ] **Step 4: Проверить парсинг на существующих файлах**

```bash
cd /home/adelurazov/Projects/ATLAS/sofia
python3 -c "
import sys; sys.path.insert(0, '.')
from scripts.update_readme import parse_project, parse_cargo

data = parse_project()
print(f'Files: {data[\"total_files\"]}')
print(f'Modules: {data[\"modules\"]}')
print(f'Structs: {data[\"structs\"]}')
print(f'Enums: {data[\"enums\"]}')
print(f'Functions: {data[\"functions\"]}')
assert data['total_files'] > 0, 'No .rs files found'
assert len(data['modules']) > 0, 'No modules found'
print('Parse OK')
"
```

- [ ] **Step 5: Commit**

```bash
git add scripts/update_readme.py
git commit -m "feat: добавлен парсинг публичных символов из .rs файлов"
```

---

### Task 2: LLM-интеграция через 9Router

**Files:**
- Modify: `scripts/update_readme.py`

- [ ] **Step 1: Реализовать функцию запроса к 9Router**

Добавить после `parse_cargo`:

```python
import urllib.request
import urllib.error


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
```

- [ ] **Step 2: Реализовать генерацию описаний модулей через LLM**

```python
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
```

- [ ] **Step 3: Реализовать генерацию сводки изменений**

```python
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
```

- [ ] **Step 4: Commit**

```bash
git add scripts/update_readme.py
git commit -m "feat: добавлена LLM-генерация описаний модулей и сводки изменений"
```

---

### Task 3: Сборка README по маркерам

**Files:**
- Modify: `scripts/update_readme.py`

- [ ] **Step 1: Реализовать функции работы с маркерами**

```python
def read_current_readme() -> str:
    """Прочитать текущий README.md."""
    if README_PATH.exists():
        return README_PATH.read_text(encoding="utf-8")
    return ""


def replace_section(content: str, section_name: str, new_section_content: str) -> str:
    """Заменить содержимое между маркерами BEGIN/END в секции."""
    begin, end = MARKERS[section_name]

    pattern = re.escape(begin) + r".*?" + re.escape(end)
    replacement = f"{begin}\n{new_section_content.strip()}\n{end}"

    if re.search(pattern, content, re.DOTALL):
        return re.sub(pattern, replacement, content, count=1, flags=re.DOTALL)
    else:
        logging.warning("Section '%s' markers not found in README", section_name)
        return content


def build_structure_section(project_data: Dict) -> str:
    """Собрать секцию 'Структура проекта'."""
    lines = ["```", "src/"]
    for f in project_data["files"]:
        indent = "    " * str(f["path"]).count("/")
        lines.append(f"{indent}{os.path.basename(f['path'])}")
    lines.append("```")
    return "\n".join(lines)


def build_modules_section(project_data: Dict, descriptions: Dict[str, str]) -> str:
    """Собрать секцию 'Модули'."""
    lines = ["| Модуль | Публичные типы | Описание |", "|---|---|---|"]
    for f in project_data["files"]:
        mod_name = Path(f["path"]).stem
        types = ", ".join(f["structs"] + f["enums"])
        desc = descriptions.get(mod_name, "")
        lines.append(f"| `{mod_name}` | {types} | {desc} |")
    return "\n".join(lines)


def build_stats_section(project_data: Dict) -> str:
    """Собрать секцию 'Статистика'."""
    return (
        f"- **Файлов:** {project_data['total_files']}\n"
        f"- **Модулей:** {len(project_data['modules'])}\n"
        f"- **Публичных структур:** {len(project_data['structs'])}\n"
        f"- **Публичных enum:** {len(project_data['enums'])}\n"
        f"- **Публичных функций:** {len(project_data['functions'])}\n"
        f"- **Строк кода:** {project_data['total_loc']}\n"
        f"- **Модулей с тестами:** {project_data['test_modules']}"
    )


def build_architecture_section(project_data: Dict, descriptions: Dict[str, str]) -> str:
    """Собрать секцию 'Архитектура'."""
    has_compiler = "compiler" in project_data["modules"]
    has_vm = "vm" in project_data["modules"]
    has_evaluator = "evaluator" in project_data["modules"]

    if has_compiler and has_vm:
        mermaid = """```mermaid
graph LR
    A[Source Code] --> B(Lexer)
    B --> C(Parser)
    C --> D{AST}
    D --> E(Compiler)
    E --> F(Bytecode)
    F --> G(VM)
    G --> H[Result]
    D -.-> I(Evaluator - fallback)
    I --> H
```"""
    else:
        mermaid = """```mermaid
graph LR
    A[Source Code] --> B(Lexer)
    B --> C(Parser)
    C --> D(AST)
    D --> E(Evaluator)
    E --> F[Result]
```"""

    desc_lines = []
    for mod_name in ["lexer", "parser", "ast", "evaluator", "compiler", "vm"]:
        if mod_name in project_data["modules"] and mod_name in descriptions:
            desc_lines.append(f"- **{mod_name.title()}:** {descriptions[mod_name]}")

    return mermaid + "\n\n" + "\n".join(desc_lines)


def build_changes_section(changes: Optional[str], project_data: Dict) -> str:
    """Собрать секцию 'Последние изменения'."""
    lines = []
    if changes:
        lines.append(changes)
    lines.append("")
    lines.append(f"*Автообновлено: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}*")
    lines.append(f"*Всего модулей: {len(project_data['modules'])}, строк кода: {project_data['total_loc']}*")
    return "\n".join(lines)
```

- [ ] **Step 2: Commit**

```bash
git add scripts/update_readme.py
git commit -m "feat: реализована сборка README по маркерам"
```

---

### Task 4: Валидация, backup, rollback, логирование

**Files:**
- Modify: `scripts/update_readme.py`

- [ ] **Step 1: Реализовать backup**

```python
def backup_readme() -> Optional[Path]:
    """Создать backup текущего README. Вернуть путь к backup или None."""
    if not README_PATH.exists():
        return None

    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    backup_path = BACKUP_DIR / f"README.md.{timestamp}"

    shutil.copy2(README_PATH, backup_path)

    # Ротация: оставить только последние 5
    backups = sorted(BACKUP_DIR.glob("README.md.*"))
    for old in backups[:-5]:
        old.unlink()

    return backup_path
```

- [ ] **Step 2: Реализовать валидацию**

```python
def validate_readme(content: str) -> Tuple[bool, List[str]]:
    """Проверить корректность сгенерированного README. Вернуть (OK, [ошибки])."""
    errors = []

    if len(content) < 500:
        errors.append(f"Content too short: {len(content)} chars (min 500)")

    for name, (begin, end) in MARKERS.items():
        if begin not in content:
            errors.append(f"Missing BEGIN marker for '{name}'")
        if end not in content:
            errors.append(f"Missing END marker for '{name}'")
        if begin in content and end in content:
            # Проверяем, что BEGIN перед END
            if content.index(begin) > content.index(end):
                errors.append(f"BEGIN marker after END for '{name}'")

    return len(errors) == 0, errors
```

- [ ] **Step 3: Реализовать rollback**

```python
def rollback_readme() -> bool:
    """Восстановить README из последнего backup. Вернуть True при успехе."""
    backups = sorted(BACKUP_DIR.glob("README.md.*"))
    if not backups:
        logging.error("No backups found for rollback")
        return False

    latest = backups[-1]
    shutil.copy2(latest, README_PATH)
    logging.warning("Rollback to backup: %s", latest.name)
    return True
```

- [ ] **Step 4: Настроить логирование**

```python
def setup_logging():
    """Настроить логирование в файл и stdout."""
    logging.basicConfig(
        level=logging.INFO,
        format="[%(asctime)s] %(levelname)-5s | %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
        handlers=[
            logging.FileHandler(LOG_FILE),
            logging.StreamHandler(),
        ],
    )
```

- [ ] **Step 5: Commit**

```bash
git add scripts/update_readme.py
git commit -m "feat: реализованы валидация, backup, rollback и логирование"
```

---

### Task 5: Точка входа и CLI

**Files:**
- Modify: `scripts/update_readme.py`

- [ ] **Step 1: Реализовать main() — полный пайплайн**

В конец файла:

```python
def main():
    setup_logging()

    parser = argparse.ArgumentParser(description="Sync README.md with project code")
    parser.add_argument("--trigger", choices=["post-commit", "manual"], default="manual")
    parser.add_argument("--rollback", action="store_true", help="Rollback to last backup")
    args = parser.parse_args()

    # Rollback mode
    if args.rollback:
        if rollback_readme():
            logging.info("Rollback completed")
        else:
            logging.error("Rollback failed")
            sys.exit(1)
        return

    commit_hash = ""
    if args.trigger == "post-commit":
        try:
            commit_hash = subprocess.run(
                ["git", "rev-parse", "--short", "HEAD"],
                capture_output=True, text=True, cwd=PROJECT_ROOT, timeout=5,
            ).stdout.strip()
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass

    logging.info("Sync started | trigger=%s commit=%s", args.trigger, commit_hash)

    # 1. Backup
    backup_path = backup_readme()
    if backup_path:
        logging.info("Backup saved: %s", backup_path.name)

    # 2. Parse project
    project_data = parse_project()
    cargo_data = parse_cargo()
    logging.info(
        "Parsed %d .rs files, %d public symbols",
        project_data["total_files"],
        len(project_data["structs"]) + len(project_data["enums"]) + len(project_data["functions"]),
    )

    # 3. LLM generation
    descriptions = generate_module_descriptions(project_data)
    changes = generate_changes_summary()
    if descriptions:
        logging.info("LLM: generated descriptions for %d modules", len(descriptions))
    if changes:
        logging.info("LLM: generated changes summary (%d chars)", len(changes))

    # 4. Build sections
    readme = read_current_readme()
    old_readme = readme

    readme = replace_section(readme, "structure", build_structure_section(project_data))
    readme = replace_section(readme, "architecture", build_architecture_section(project_data, descriptions))
    readme = replace_section(readme, "modules", build_modules_section(project_data, descriptions))
    readme = replace_section(readme, "stats", build_stats_section(project_data))
    readme = replace_section(readme, "changes", build_changes_section(changes, project_data))

    # 5. Validate
    is_valid, errors = validate_readme(readme)
    if not is_valid:
        for err in errors:
            logging.error("Validation failed: %s", err)
        logging.error("README not updated — validation failed")
        sys.exit(1)

    logging.info("Validation passed (size=%d bytes, sections=%d)", len(readme), len(MARKERS))

    # 6. Write
    README_PATH.write_text(readme, encoding="utf-8")
    logging.info("README updated successfully")
    logging.info("Sync completed")


if __name__ == "__main__":
    main()
```

- [ ] **Step 2: Commit**

```bash
git add scripts/update_readme.py
git commit -m "feat: реализована точка входа CLI с полным пайплайном"
```

---

### Task 6: Git hook + Makefile + .gitignore

**Files:**
- Create: `Makefile`
- Create: `.git/hooks/post-commit`
- Modify: `.gitignore`

- [ ] **Step 1: Создать Makefile**

```makefile
.PHONY: update-readme readme-rollback readme-status

update-readme:
	python3 scripts/update_readme.py --trigger=manual

readme-rollback:
	python3 scripts/update_readme.py --rollback

readme-status:
	@cat .readme_sync.log 2>/dev/null || echo "No sync logs found"
```

- [ ] **Step 2: Создать git hook post-commit**

```bash
#!/bin/bash
# Автоматическое обновление README после коммита

PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$PROJECT_ROOT" || exit 0

# Запускаем только если были изменения в .rs, Cargo.toml, Cargo.lock, docs/
CHANGED_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null || echo "")

if echo "$CHANGED_FILES" | grep -qE '\.rs$|Cargo\.toml|Cargo\.lock|docs/'; then
    python3 scripts/update_readme.py --trigger=post-commit &
fi
```

Сделать hook исполняемым:

```bash
chmod +x .git/hooks/post-commit
```

- [ ] **Step 3: Обновить .gitignore**

Добавить в `.gitignore`:

```
# README autosync
.readme_backups/
.readme_sync.log
```

- [ ] **Step 4: Commit**

```bash
git add Makefile .gitignore
git add -f .git/hooks/post-commit  # hooks are in .gitignore by default
git commit -m "feat: добавлены Makefile, git hook post-commit, .gitignore для autosync"
```

**Note:** Git hooks не трекаются в репозитории (лежат в `.git/`). Нужно либо добавить скрипт установки hook'а, либо документировать ручную установку. Предлагаю скрипт установки:

- [ ] **Step 5: Создать скрипт установки hook**

```bash
#!/bin/bash
# scripts/install-hooks.sh — установить git hooks для проекта

HOOKS_DIR="$(cd "$(dirname "$0")/.." && pwd)/.git/hooks"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

cp "$SCRIPT_DIR/../.git/hooks/post-commit" "$HOOKS_DIR/post-commit" 2>/dev/null || true

cat > "$HOOKS_DIR/post-commit" << 'HOOK'
#!/bin/bash
PROJECT_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$PROJECT_ROOT" || exit 0
CHANGED_FILES=$(git diff --name-only HEAD~1..HEAD 2>/dev/null || echo "")
if echo "$CHANGED_FILES" | grep -qE '\.rs$|Cargo\.toml|Cargo\.lock|docs/'; then
    python3 scripts/update_readme.py --trigger=post-commit &
fi
HOOK
chmod +x "$HOOKS_DIR/post-commit"
echo "Hook installed: $HOOKS_DIR/post-commit"
```

И запустить:

```bash
bash scripts/install-hooks.sh
```

```bash
git add scripts/install-hooks.sh
git commit -m "chore: добавлен скрипт установки git hooks"
```

---

### Task 7: Обновление README.md — добавление маркеров

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Добавить маркеры в README.md**

Обновить README.md, добавив секции с маркерами. Существующий контент сохраняется, маркеры вставляются в соответствующие места:

```markdown
## 📁 Структура проекта
<!-- BEGIN structure -->
*Автоматически генерируется...*
<!-- END structure -->

## 🏗 Архитектура
<!-- BEGIN architecture -->
*Автоматически генерируется...*
<!-- END architecture -->

## 📦 Модули
<!-- BEGIN modules -->
*Автоматически генерируется...*
<!-- END modules -->

## 📊 Статистика
<!-- BEGIN stats -->
*Автоматически генерируется...*
<!-- END stats -->

## 🔄 Последние изменения
<!-- BEGIN changes -->
*Автоматически генерируется...*
<!-- END changes -->
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "feat: добавлены маркеры для автообновляемых секций README"
```

---

### Task 8: Интеграционное тестирование

- [ ] **Step 1: Запустить скрипт вручную и проверить результат**

```bash
cd /home/adelurazov/Projects/ATLAS/sofia
python3 scripts/update_readme.py --trigger=manual
```

Ожидаемый результат:
- README.md обновлён — секции между маркерами заменены
- `.readme_sync.log` содержит записи INFO
- `.readme_backups/` содержит backup

- [ ] **Step 2: Проверить валидацию на пустом/битом контенте**

```bash
# Испортить маркер и проверить что скрипт отказывается писать
```

- [ ] **Step 3: Проверить rollback**

```bash
make readme-rollback
# README должен вернуться к предыдущей версии
```

- [ ] **Step 4: Проверить git hook**

```bash
# Внести изменение в .rs файл, закоммитить, проверить что README обновился
echo "" >> src/lib.rs
git add src/lib.rs
git commit -m "test: triger README update"
# Проверить .readme_sync.log
```

- [ ] **Step 5: Финальный commit**

```bash
git add -A
git commit -m "chore: финальная настройка README autosync"
```
