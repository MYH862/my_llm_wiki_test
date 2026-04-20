#!/usr/bin/env python3
"""
Tauri Store 配置导出/导入工具

用法:
  python migrate_config.py export <store_path> [options]
  python migrate_config.py import <export_file> [options]

导出选项:
  --output         输出文件路径 (默认: config_export.json)

导入选项:
  --api-url        API 服务地址 (默认: http://localhost:3000/api)
  --token          JWT 认证令牌 (必需)
  --project-id     目标项目 ID (必需)
"""

import os
import sys
import json
import argparse
from pathlib import Path
from typing import Optional

try:
    import requests
except ImportError:
    print("错误: 需要安装 requests 包")
    print("运行: pip install requests")
    sys.exit(1)


class ConfigMigrator:
    def __init__(self, api_base_url: str = "http://localhost:3000/api", token: Optional[str] = None):
        self.api_base_url = api_base_url
        self.token = token
        self.headers = {"Content-Type": "application/json"}
        if token:
            self.headers["Authorization"] = f"Bearer {token}"

    def export_config(self, store_path: str, output_file: str):
        store_file = Path(store_path)
        if not store_file.exists():
            print(f"错误: Store 文件不存在: {store_path}")
            sys.exit(1)

        print(f"导出配置: {store_path}")

        with open(store_file, "r", encoding="utf-8") as f:
            config_data = json.load(f)

        export_data = {
            "llm_configs": config_data.get("llmConfigs", []),
            "search_api_config": config_data.get("searchApiConfig", {}),
            "embedding_config": config_data.get("embeddingConfig", {}),
            "language": config_data.get("language", "en"),
            "recent_projects": config_data.get("recentProjects", []),
        }

        with open(output_file, "w", encoding="utf-8") as f:
            json.dump(export_data, f, ensure_ascii=False, indent=2)

        print(f"✓ 配置已导出到: {output_file}")
        print(f"  - LLM 配置: {len(export_data['llm_configs'])} 个")
        print(f"  - 搜索 API 配置: {'是' if export_data['search_api_config'] else '否'}")
        print(f"  - Embedding 配置: {'是' if export_data['embedding_config'] else '否'}")
        print(f"  - 语言: {export_data['language']}")
        print(f"  - 最近项目: {len(export_data['recent_projects'])} 个")

    def import_config(self, export_file: str, project_id: str):
        with open(export_file, "r", encoding="utf-8") as f:
            config_data = json.load(f)

        print(f"\n导入配置到项目: {project_id}")
        print(f"来源文件: {export_file}\n")

        imported_count = 0
        failed_count = 0

        llm_configs = config_data.get("llm_configs", [])
        for llm_config in llm_configs:
            try:
                response = requests.post(
                    f"{self.api_base_url}/llm/configs",
                    headers=self.headers,
                    json={
                        "project_id": project_id,
                        "name": llm_config.get("name", ""),
                        "provider": llm_config.get("provider", ""),
                        "api_key": llm_config.get("apiKey", ""),
                        "model": llm_config.get("model", ""),
                        "base_url": llm_config.get("baseUrl", ""),
                    },
                )
                if response.status_code in [200, 201]:
                    imported_count += 1
                    print(f"✓ LLM 配置: {llm_config.get('name', 'unknown')}")
                else:
                    failed_count += 1
                    print(f"✗ LLM 配置失败: {llm_config.get('name', 'unknown')} - {response.text}")
            except Exception as e:
                failed_count += 1
                print(f"✗ LLM 配置失败: {llm_config.get('name', 'unknown')} - {e}")

        search_config = config_data.get("search_api_config", {})
        if search_config:
            try:
                response = requests.post(
                    f"{self.api_base_url}/projects/{project_id}/settings",
                    headers=self.headers,
                    json={
                        "key": "search_api_config",
                        "value": json.dumps(search_config),
                    },
                )
                if response.status_code in [200, 201]:
                    imported_count += 1
                    print(f"✓ 搜索 API 配置")
                else:
                    failed_count += 1
                    print(f"✗ 搜索 API 配置失败: {response.text}")
            except Exception as e:
                failed_count += 1
                print(f"✗ 搜索 API 配置失败: {e}")

        embedding_config = config_data.get("embedding_config", {})
        if embedding_config:
            try:
                response = requests.post(
                    f"{self.api_base_url}/projects/{project_id}/settings",
                    headers=self.headers,
                    json={
                        "key": "embedding_config",
                        "value": json.dumps(embedding_config),
                    },
                )
                if response.status_code in [200, 201]:
                    imported_count += 1
                    print(f"✓ Embedding 配置")
                else:
                    failed_count += 1
                    print(f"✗ Embedding 配置失败: {response.text}")
            except Exception as e:
                failed_count += 1
                print(f"✗ Embedding 配置失败: {e}")

        print(f"\n导入完成!")
        print(f"成功: {imported_count} 项")
        print(f"失败: {failed_count} 项")


def main():
    parser = argparse.ArgumentParser(description="Tauri Store 配置导出/导入")
    subparsers = parser.add_subparsers(dest="command", help="命令")

    export_parser = subparsers.add_parser("export", help="导出配置")
    export_parser.add_argument("store_path", help="Tauri Store 文件路径")
    export_parser.add_argument("--output", default="config_export.json", help="输出文件路径")

    import_parser = subparsers.add_parser("import", help="导入配置")
    import_parser.add_argument("export_file", help="导出的配置文件")
    import_parser.add_argument("--api-url", default="http://localhost:3000/api", help="API 服务地址")
    import_parser.add_argument("--token", required=True, help="JWT 认证令牌")
    import_parser.add_argument("--project-id", required=True, help="目标项目 ID")

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        sys.exit(1)

    migrator = ConfigMigrator(
        api_base_url=getattr(args, "api_url", "http://localhost:3000/api"),
        token=getattr(args, "token", None),
    )

    if args.command == "export":
        migrator.export_config(args.store_path, args.output)
    elif args.command == "import":
        migrator.import_config(args.export_file, args.project_id)


if __name__ == "__main__":
    main()
