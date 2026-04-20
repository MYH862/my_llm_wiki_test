#!/usr/bin/env python3
"""
LanceDB 向量数据导出脚本

用法:
  python export_lancedb.py <project_path> [options]

选项:
  --output         输出文件路径 (默认: vectors_export.json)
  --batch-size     批次大小 (默认: 100)
"""

import os
import sys
import json
import argparse
from pathlib import Path
from typing import Optional

try:
    import lancedb
except ImportError:
    print("错误: 需要安装 lancedb 包")
    print("运行: pip install lancedb")
    sys.exit(1)


class LanceDBExporter:
    def __init__(self, project_path: str, output_file: str, batch_size: int = 100):
        self.project_path = Path(project_path)
        self.output_file = Path(output_file)
        self.batch_size = batch_size
        self.exported_count = 0

    def find_lancedb_path(self) -> Optional[Path]:
        lancedb_dir = self.project_path / ".lancedb"
        if lancedb_dir.exists():
            return lancedb_dir

        for item in self.project_path.rglob(".lancedb"):
            if item.is_dir():
                return item

        return None

    def export_vectors(self) -> dict:
        lancedb_path = self.find_lancedb_path()
        if not lancedb_path:
            print(f"错误: 在 {self.project_path} 中未找到 LanceDB 数据")
            sys.exit(1)

        print(f"找到 LanceDB 数据: {lancedb_path}")
        print(f"开始导出向量数据...\n")

        db = lancedb.connect(str(lancedb_path))
        tables = db.table_names()

        if not tables:
            print("警告: 未找到任何向量表")
            return {"tables": [], "total_vectors": 0}

        export_data = {"tables": [], "total_vectors": 0}

        for table_name in tables:
            print(f"导出表: {table_name}")
            table = db.open_table(table_name)

            vectors = []
            batch = table.to_pandas()

            for _, row in batch.iterrows():
                vector_data = {
                    "id": str(row.get("id", "")),
                    "file_path": str(row.get("file_path", "")),
                    "content": str(row.get("content", "")),
                    "vector": row.get("vector", []).tolist() if hasattr(row.get("vector"), "tolist") else list(row.get("vector", [])),
                    "metadata": {
                        k: v for k, v in row.items()
                        if k not in ["id", "file_path", "content", "vector"]
                    }
                }
                vectors.append(vector_data)
                self.exported_count += 1

            export_data["tables"].append({
                "name": table_name,
                "vectors": vectors,
                "count": len(vectors),
            })

            print(f"  ✓ 导出 {len(vectors)} 个向量")

        export_data["total_vectors"] = self.exported_count

        with open(self.output_file, "w", encoding="utf-8") as f:
            json.dump(export_data, f, ensure_ascii=False, indent=2)

        print(f"\n导出完成!")
        print(f"总计: {self.exported_count} 个向量")
        print(f"输出文件: {self.output_file}")

        return export_data


def main():
    parser = argparse.ArgumentParser(description="LanceDB 向量数据导出")
    parser.add_argument("project_path", help="项目本地路径")
    parser.add_argument("--output", default="vectors_export.json", help="输出文件路径")
    parser.add_argument("--batch-size", type=int, default=100, help="批次大小")

    args = parser.parse_args()

    exporter = LanceDBExporter(
        project_path=args.project_path,
        output_file=args.output,
        batch_size=args.batch_size,
    )

    exporter.export_vectors()


if __name__ == "__main__":
    main()
