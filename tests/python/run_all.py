"""自动发现并顺序运行本目录全部 test_*.py，按 ``  ok  `` / ``  FAIL `` 行汇总。

- 自动发现消除了旧版手工 SUITES 清单漏跑 test_b_base.py 的缺陷；
- 子进程以本文件所在目录为 cwd，可从任意位置调用；
- 任一套件失败（含无 FAIL 行的非零退出——崩溃/超时）时退出码 1。
"""
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
suites = sorted(p.name for p in HERE.glob("test_*.py"))

if not suites:
    print("no test_*.py suites found")
    sys.exit(1)

# 仅统计断言行（"  ok   xxx" / "  FAIL xxx"），不计 SUMMARY 复述行
# （"  FAILED: xxx" 不匹配 FAIL\s，避免重复计数）。
OK_LINE = re.compile(r"^\s*ok\s")
FAIL_LINE = re.compile(r"^\s*FAIL\s")

total_pass = total_fail = 0
failed_suites = []
for name in suites:
    print(f"\n######## {name} ########", flush=True)
    r = subprocess.run(
        [sys.executable, str(HERE / name)],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        cwd=str(HERE),
        # 子进程管道输出统一 UTF-8（Windows 默认按 locale 编码管道，会与本
        # 处 utf-8 解码不一致导致中文乱码；对控制台输出无影响）。
        env={**os.environ, "PYTHONIOENCODING": "utf-8"},
    )
    out = r.stdout or ""
    suite_fail_lines = 0
    for line in out.splitlines():
        if OK_LINE.match(line):
            total_pass += 1
        elif FAIL_LINE.match(line):
            total_fail += 1
            suite_fail_lines += 1
    print(out[-1500:] if len(out) > 1500 else out, end="", flush=True)
    if r.stderr.strip():
        print(f"  [stderr] {r.stderr.strip()[-600:]}", flush=True)
    if r.returncode != 0 and suite_fail_lines == 0:
        # 崩溃/跳转异常退出但没有任何 FAIL 断言行：按 1 次失败计，避免被漏报。
        total_fail += 1
        print(f"  SUITE-FAIL {name}: exit={r.returncode} (no FAIL lines — crash?)", flush=True)
    if r.returncode != 0:
        failed_suites.append(name)

print(f"\n==== TOTALS: {total_pass} ok, {total_fail} FAIL ({len(suites)} suites) ====")
for name in failed_suites:
    print(f"  suite failed: {name}")
sys.exit(1 if total_fail else 0)
