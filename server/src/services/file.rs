use minio::s3::args::{BucketExistsArgs, MakeBucketArgs, PutObjectArgs, GetObjectArgs, RemoveObjectArgs, ListObjectsArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::Ssl;
use std::path::Path;
use tokio::io::AsyncReadExt;
use tracing::info;

use crate::middleware::error::AppError;

pub struct MinIOService {
    client: Client,
    bucket_prefix: String,
}

impl MinIOService {
    pub fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        use_ssl: bool,
        bucket_prefix: &str,
    ) -> Result<Self, AppError> {
        let base_url = endpoint.parse::<BaseUrl>().map_err(|e| {
            AppError::BadRequest(format!("Invalid MinIO endpoint: {}", e))
        })?;

        let provider = StaticProvider::new(access_key, secret_key, None);

        let client = Client::new(
            base_url,
            Some(provider),
            None,
            None,
            Ssl::from(use_ssl),
        );

        Ok(Self {
            client,
            bucket_prefix: bucket_prefix.to_string(),
        })
    }

    pub async fn ensure_bucket(&self, project_id: &str) -> Result<(), AppError> {
        let bucket_name = format!("{}-{}", self.bucket_prefix, project_id);

        let exists = self
            .client
            .bucket_exists(&BucketExistsArgs::new(&bucket_name).map_err(|e| AppError::Internal)?)
            .await
            .map_err(|e| AppError::Internal)?;

        if !exists {
            self.client
                .make_bucket(
                    &MakeBucketArgs::new(&bucket_name).map_err(|e| AppError::Internal)?,
                )
                .await
                .map_err(|e| AppError::Internal)?;

            info!("Created MinIO bucket: {}", bucket_name);
        }

        Ok(())
    }

    pub async fn upload_file(
        &self,
        project_id: &str,
        path: &str,
        content: &[u8],
        content_type: &str,
    ) -> Result<(), AppError> {
        let bucket_name = format!("{}-{}", self.bucket_prefix, project_id);
        self.ensure_bucket(project_id).await?;

        self.client
            .put_object(
                &PutObjectArgs::new(
                    &bucket_name,
                    path,
                    &mut std::io::Cursor::new(content),
                    Some(content.len() as u64),
                    None,
                )
                .map_err(|e| AppError::Internal)?
                .content_type(content_type),
            )
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(())
    }

    pub async fn download_file(&self, project_id: &str, path: &str) -> Result<Vec<u8>, AppError> {
        let bucket_name = format!("{}-{}", self.bucket_prefix, project_id);

        let mut object = self
            .client
            .get_object(
                &GetObjectArgs::new(&bucket_name, path).map_err(|e| AppError::Internal)?,
            )
            .await
            .map_err(|e| AppError::Internal)?;

        let mut buffer = Vec::new();
        object
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(buffer)
    }

    pub async fn delete_file(&self, project_id: &str, path: &str) -> Result<(), AppError> {
        let bucket_name = format!("{}-{}", self.bucket_prefix, project_id);

        self.client
            .remove_object(
                &RemoveObjectArgs::new(&bucket_name, path).map_err(|e| AppError::Internal)?,
            )
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(())
    }

    pub async fn list_files(&self, project_id: &str, prefix: &str) -> Result<Vec<String>, AppError> {
        let bucket_name = format!("{}-{}", self.bucket_prefix, project_id);

        let mut objects = self
            .client
            .list_objects(
                &ListObjectsArgs::new(&bucket_name)
                    .prefix(prefix)
                    .recursive(true)
                    .map_err(|e| AppError::Internal)?,
            )
            .await
            .map_err(|e| AppError::Internal)?;

        let mut files = Vec::new();
        while let Some(result) = objects.next().await {
            match result {
                Ok(item) => {
                    files.push(item.name);
                }
                Err(_) => continue,
            }
        }

        Ok(files)
    }
}
