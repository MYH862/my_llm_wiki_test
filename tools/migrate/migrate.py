#!/usr/bin/env python3
"""
数据迁移主脚本 - 一键完成所有迁移步骤

用法:
  python migrate.py <project_path> <project_id> [options]
"""

import sys
import argparse
import subprocess
from pathlib import Path


def run_script(script_name: str, args: list) -> bool:
    script_path = Path(__file__).parent / script_name
    cmd = [sys.executable, str(script_path)] + args

    print(f"\n{'=' * 60}")
    print(f"执行: {script_name}")
    print(f"{'=' * 60}\n")

    result = subprocess.run(cmd, cwd=Path(__file__).parent)
    return result.returncode == 0


def main():
    parser = argparse.ArgumentParser(description="数据迁移主脚本")
    parser.add_argument("project_path", help="项目本地路径")
    parser.add_argument("project_id", help="项目 ID (UUID)")
    parser.add_argument("--minio-url", default="http://localhost:9000")
    parser.add_argument("--access-key", default="minioadmin")
    parser.add_argument("--secret-key", default="minioadmin")
    parser.add_argument("--qdrant-url", default="http://localhost:6333")
    parser.add_argument("--api-url", default="http://localhost:3000/api")
    parser.add_argument("--token", help="JWT 认证令牌 (用于验证)")
    parser.add_argument("--skip-files", action="store_true", help="跳过文件迁移")
    parser.add_argument("--skip-vectors", action="store_true", help="跳过向量迁移")
    parser.add_argument("--skip-config", action="store_true", help="跳过配置迁移")
    parser.add_argument("--skip-verify", action="store_true", help="跳过验证")

    args = parser.parse_args()

    print("=" * 60)
    print("LLM Wiki 数据迁移工具")
    print("=" * 60)
    print(f"项目路径: {args.project_path}")
    print(f"项目 ID: {args.project_id}")
    print("=" * 60)

    steps = []

    if not args.skip_files:
        steps.append(("migrate_files.py", [args.project_path, args.project_id,
                                            "--minio-url", args.minio_url,
                                            "--access-key", args.access_key,
                                            "--secret-key", args.secret_key]))

    if not args.skip_vectors:
        export_file = "vectors_export.json"
        steps.append(("export_lancedb.py", [args.project_path, "--output", export_file]))
        steps.append(("import_qdrant.py", [export_file, args.project_id,
                                            "--qdrant-url", args.qdrant_url]))

    if not args.skip_config:
        store_path = Path(args.project_path) / ".tauri" / "store.json"
        if store_path.exists():
            config_export = "config_export.json"
            steps.append(("migrate_config.py", ["export", str(store_path), "--output", config_export]))
            if args.token:
                steps.append(("migrate_config.py", ["import", config_export,
                                                     "--api-url", args.api_url,
                                                     "--token", args.token,
                                                     "--project-id", args.project_id]))
        else:
            print(f"\n警告: 未找到 Tauri Store 文件: {store_path}")
            print("跳过配置迁移\n")

    if not args.skip_verify and args.token:
        steps.append(("verify_migration.py", [args.project_id,
                                               "--api-url", args.api_url,
                                               "--token", args.token,
                                               "--minio-url", args.minio_url,
                                               "--access-key", args.access_key,
                                               "--secret-key", args.secret_key,
                                               "--qdrant-url", args.qdrant_url]))

    success_count = 0
    failed_count = 0

    for script_name, script_args in steps:
        if run_script(script_name, script_args):
            success_count += 1
        else:
            failed_count += 1
            print(f"\n✗ {script_name} 执行失败")
            response = input("是否继续? (y/n): ")
            if response.lower() != 'y':
                print("迁移已中止")
                sys.exit(1)

    print(f"\n{'=' * 60}")
    print(f"迁移完成!")
    print(f"成功: {success_count} 个步骤")
    print(f"失败: {failed_count} 个步骤")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
