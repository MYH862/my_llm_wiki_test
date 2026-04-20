#!/usr/bin/env python3
"""
迁移验证脚本

用法:
  python verify_migration.py <project_id> [options]

选项:
  --api-url        API 服务地址 (默认: http://localhost:3000/api)
  --token          JWT 认证令牌 (必需)
  --minio-url      MinIO 服务地址 (默认: http://localhost:9000)
  --access-key     MinIO 访问密钥 (默认: minioadmin)
  --secret-key     MinIO 密钥 (默认: minioadmin)
  --qdrant-url     Qdrant 服务地址 (默认: http://localhost:6333)
"""

import sys
import argparse
from typing import Optional

try:
    import requests
    from minio import Minio
    from qdrant_client import QdrantClient
except ImportError:
    print("错误: 需要安装依赖包")
    print("运行: pip install requests minio qdrant-client")
    sys.exit(1)


class MigrationVerifier:
    def __init__(
        self,
        project_id: str,
        api_url: str,
        token: str,
        minio_url: str,
        minio_access_key: str,
        minio_secret_key: str,
        qdrant_url: str,
    ):
        self.project_id = project_id
        self.api_url = api_url
        self.token = token
        self.headers = {"Authorization": f"Bearer {token}"}

        self.minio_client = Minio(
            minio_url.replace("http://", "").replace("https://", ""),
            access_key=minio_access_key,
            secret_key=minio_secret_key,
            secure=minio_url.startswith("https"),
        )

        self.qdrant_client = QdrantClient(url=qdrant_url)

    def verify_files(self) -> bool:
        print("\n验证文件迁移...")
        try:
            response = requests.get(
                f"{self.api_url}/files/{self.project_id}/list",
                headers=self.headers,
            )

            if response.status_code == 200:
                data = response.json()
                entries = data.get("entries", [])
                print(f"  ✓ 文件列表 API 正常")
                print(f"  ✓ 找到 {len(entries)} 个文件")
                return True
            else:
                print(f"  ✗ 文件列表 API 失败: {response.text}")
                return False
        except Exception as e:
            print(f"  ✗ 文件验证失败: {e}")
            return False

    def verify_vectors(self) -> bool:
        print("\n验证向量迁移...")
        try:
            collections = [c.name for c in self.qdrant_client.get_collections().collections]
            collection_name = f"wiki-vectors-{self.project_id}"

            if collection_name in collections:
                info = self.qdrant_client.get_collection(collection_name)
                print(f"  ✓ Qdrant 集合存在: {collection_name}")
                print(f"  ✓ 向量数量: {info.points_count}")
                return True
            else:
                print(f"  ✗ Qdrant 集合不存在: {collection_name}")
                print(f"  可用集合: {', '.join(collections)}")
                return False
        except Exception as e:
            print(f"  ✗ 向量验证失败: {e}")
            return False

    def verify_minio(self) -> bool:
        print("\n验证 MinIO 存储...")
        try:
            bucket_name = f"wiki-files-{self.project_id}"
            if self.minio_client.bucket_exists(bucket_name):
                objects = list(self.minio_client.list_objects(bucket_name))
                print(f"  ✓ MinIO 存储桶存在: {bucket_name}")
                print(f"  ✓ 对象数量: {len(objects)}")
                return True
            else:
                print(f"  ✗ MinIO 存储桶不存在: {bucket_name}")
                return False
        except Exception as e:
            print(f"  ✗ MinIO 验证失败: {e}")
            return False

    def verify_all(self):
        print(f"=" * 60)
        print(f"迁移验证报告")
        print(f"项目 ID: {self.project_id}")
        print(f"=" * 60)

        results = {
            "files": self.verify_files(),
            "vectors": self.verify_vectors(),
            "minio": self.verify_minio(),
        }

        print(f"\n{'=' * 60}")
        print(f"验证结果:")
        print(f"  文件迁移: {'✓ 通过' if results['files'] else '✗ 失败'}")
        print(f"  向量迁移: {'✓ 通过' if results['vectors'] else '✗ 失败'}")
        print(f"  MinIO 存储: {'✓ 通过' if results['minio'] else '✗ 失败'}")
        print(f"{'=' * 60}")

        if all(results.values()):
            print(f"\n✓ 所有验证通过！迁移成功！")
            return True
        else:
            print(f"\n✗ 部分验证失败，请检查日志")
            return False


def main():
    parser = argparse.ArgumentParser(description="迁移验证")
    parser.add_argument("project_id", help="项目 ID")
    parser.add_argument("--api-url", default="http://localhost:3000/api", help="API 服务地址")
    parser.add_argument("--token", required=True, help="JWT 认证令牌")
    parser.add_argument("--minio-url", default="http://localhost:9000", help="MinIO 服务地址")
    parser.add_argument("--access-key", default="minioadmin", help="MinIO 访问密钥")
    parser.add_argument("--secret-key", default="minioadmin", help="MinIO 密钥")
    parser.add_argument("--qdrant-url", default="http://localhost:6333", help="Qdrant 服务地址")

    args = parser.parse_args()

    verifier = MigrationVerifier(
        project_id=args.project_id,
        api_url=args.api_url,
        token=args.token,
        minio_url=args.minio_url,
        minio_access_key=args.access_key,
        minio_secret_key=args.secret_key,
        qdrant_url=args.qdrant_url,
    )

    success = verifier.verify_all()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
