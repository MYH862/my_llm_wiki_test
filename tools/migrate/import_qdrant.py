#!/usr/bin/env python3
"""
Qdrant 向量导入脚本

用法:
  python import_qdrant.py <export_file> <project_id> [options]

选项:
  --qdrant-url     Qdrant 服务地址 (默认: http://localhost:6333)
  --api-key        Qdrant API 密钥 (可选)
  --collection     集合名称 (默认: wiki-vectors-{project_id})
  --vector-size    向量维度 (默认: 384)
  --batch-size     批次大小 (默认: 100)
"""

import sys
import json
import argparse
import uuid
from typing import Optional, List

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import (
        Distance,
        VectorParams,
        PointStruct,
    )
except ImportError:
    print("错误: 需要安装 qdrant-client 包")
    print("运行: pip install qdrant-client")
    sys.exit(1)


class QdrantImporter:
    def __init__(
        self,
        qdrant_url: str,
        api_key: Optional[str],
        collection: str,
        vector_size: int = 384,
        batch_size: int = 100,
    ):
        self.client = QdrantClient(url=qdrant_url, api_key=api_key)
        self.collection = collection
        self.vector_size = vector_size
        self.batch_size = batch_size
        self.imported_count = 0
        self.failed_count = 0

    def create_collection(self):
        collections = [c.name for c in self.client.get_collections().collections]

        if self.collection in collections:
            print(f"集合已存在: {self.collection}")
            return

        self.client.create_collection(
            collection_name=self.collection,
            vectors_config=VectorParams(
                size=self.vector_size,
                distance=Distance.COSINE,
            ),
        )
        print(f"✓ 创建集合: {self.collection}")

    def import_vectors(self, export_file: str):
        with open(export_file, "r", encoding="utf-8") as f:
            export_data = json.load(f)

        print(f"\n开始导入向量数据...")
        print(f"来源文件: {export_file}")
        print(f"目标集合: {self.collection}\n")

        self.create_collection()

        for table in export_data.get("tables", []):
            table_name = table["name"]
            vectors = table["vectors"]

            print(f"导入表: {table_name} ({len(vectors)} 个向量)")

            points = []
            for vector_data in vectors:
                try:
                    point_id = str(uuid.uuid4())
                    if vector_data.get("id"):
                        point_id = str(vector_data["id"])

                    point = PointStruct(
                        id=point_id,
                        vector=vector_data["vector"],
                        payload={
                            "file_path": vector_data.get("file_path", ""),
                            "content": vector_data.get("content", ""),
                            "table_name": table_name,
                            **vector_data.get("metadata", {}),
                        },
                    )
                    points.append(point)

                    if len(points) >= self.batch_size:
                        self.client.upsert(
                            collection_name=self.collection,
                            points=points,
                        )
                        self.imported_count += len(points)
                        points = []

                except Exception as e:
                    self.failed_count += 1
                    print(f"  ✗ 失败: {vector_data.get('file_path', 'unknown')} - {e}")

            if points:
                try:
                    self.client.upsert(
                        collection_name=self.collection,
                        points=points,
                    )
                    self.imported_count += len(points)
                except Exception as e:
                    self.failed_count += len(points)
                    print(f"  ✗ 批次导入失败: {e}")

            print(f"  ✓ 导入 {table['count']} 个向量")

        print(f"\n导入完成!")
        print(f"成功: {self.imported_count} 个向量")
        print(f"失败: {self.failed_count} 个向量")


def main():
    parser = argparse.ArgumentParser(description="Qdrant 向量导入")
    parser.add_argument("export_file", help="导出的向量数据文件")
    parser.add_argument("project_id", help="项目 ID")
    parser.add_argument("--qdrant-url", default="http://localhost:6333", help="Qdrant 服务地址")
    parser.add_argument("--api-key", help="Qdrant API 密钥")
    parser.add_argument("--collection", help="集合名称 (默认: wiki-vectors-{project_id})")
    parser.add_argument("--vector-size", type=int, default=384, help="向量维度")
    parser.add_argument("--batch-size", type=int, default=100, help="批次大小")

    args = parser.parse_args()

    collection = args.collection or f"wiki-vectors-{args.project_id}"

    importer = QdrantImporter(
        qdrant_url=args.qdrant_url,
        api_key=args.api_key,
        collection=collection,
        vector_size=args.vector_size,
        batch_size=args.batch_size,
    )

    importer.import_vectors(args.export_file)


if __name__ == "__main__":
    main()
