#!/usr/bin/env python3
"""
本地文件批量上传到 MinIO 脚本

用法:
  python migrate_files.py <project_path> <project_id> [options]

选项:
  --minio-url      MinIO 服务地址 (默认: http://localhost:9000)
  --access-key     MinIO 访问密钥 (默认: minioadmin)
  --secret-key     MinIO 密钥 (默认: minioadmin)
  --bucket         MinIO 存储桶名称 (默认: wiki-files)
  --token          JWT 认证令牌 (可选)
"""

import os
import sys
import argparse
import mimetypes
from pathlib import Path
from typing import Optional

try:
    import requests
    from minio import Minio
except ImportError:
    print("错误: 需要安装依赖包")
    print("运行: pip install requests minio")
    sys.exit(1)


class FileMigrator:
    def __init__(
        self,
        minio_url: str,
        access_key: str,
        secret_key: str,
        bucket: str,
        api_base_url: str,
        token: Optional[str] = None,
    ):
        self.minio_client = Minio(
            minio_url.replace("http://", "").replace("https://", ""),
            access_key=access_key,
            secret_key=secret_key,
            secure=minio_url.startswith("https"),
        )
        self.bucket = bucket
        self.api_base_url = api_base_url
        self.token = token
        self.uploaded_count = 0
        self.failed_count = 0

    def ensure_bucket(self):
        if not self.minio_client.bucket_exists(self.bucket):
            self.minio_client.make_bucket(self.bucket)
            print(f"✓ 创建存储桶: {self.bucket}")

    def upload_file(self, project_path: str, file_path: Path) -> bool:
        try:
            relative_path = file_path.relative_to(project_path)
            object_name = f"{relative_path.as_posix()}"

            content_type, _ = mimetypes.guess_type(str(file_path))
            if content_type is None:
                content_type = "application/octet-stream"

            self.minio_client.fput_object(
                self.bucket,
                object_name,
                str(file_path),
                content_type=content_type,
            )
            self.uploaded_count += 1
            print(f"✓ 上传: {relative_path}")
            return True
        except Exception as e:
            self.failed_count += 1
            print(f"✗ 失败: {file_path} - {e}")
            return False

    def migrate_directory(self, project_path: str, exclude_dirs: Optional[list] = None):
        if exclude_dirs is None:
            exclude_dirs = [".git", "node_modules", ".cache", "dist", "build"]

        project = Path(project_path)
        if not project.exists():
            print(f"错误: 项目路径不存在: {project_path}")
            sys.exit(1)

        print(f"\n开始迁移项目: {project_path}")
        print(f"存储桶: {self.bucket}")
        print(f"排除目录: {', '.join(exclude_dirs)}\n")

        self.ensure_bucket()

        for file_path in project.rglob("*"):
            if file_path.is_file():
                relative = file_path.relative_to(project)
                if any(str(relative).startswith(d) for d in exclude_dirs):
                    continue

                self.upload_file(project_path, file_path)

        print(f"\n迁移完成!")
        print(f"成功: {self.uploaded_count} 个文件")
        print(f"失败: {self.failed_count} 个文件")


def main():
    parser = argparse.ArgumentParser(description="本地文件批量上传到 MinIO")
    parser.add_argument("project_path", help="项目本地路径")
    parser.add_argument("project_id", help="项目 ID (UUID)")
    parser.add_argument("--minio-url", default="http://localhost:9000", help="MinIO 服务地址")
    parser.add_argument("--access-key", default="minioadmin", help="MinIO 访问密钥")
    parser.add_argument("--secret-key", default="minioadmin", help="MinIO 密钥")
    parser.add_argument("--bucket", default="wiki-files", help="MinIO 存储桶名称")
    parser.add_argument("--api-url", default="http://localhost:3000/api", help="API 地址")
    parser.add_argument("--token", help="JWT 认证令牌")
    parser.add_argument("--exclude", nargs="*", default=[".git", "node_modules", ".cache"], help="排除的目录")

    args = parser.parse_args()

    migrator = FileMigrator(
        minio_url=args.minio_url,
        access_key=args.access_key,
        secret_key=args.secret_key,
        bucket=f"{args.bucket}-{args.project_id}",
        api_base_url=args.api_url,
        token=args.token,
    )

    migrator.migrate_directory(args.project_path, args.exclude)


if __name__ == "__main__":
    main()
