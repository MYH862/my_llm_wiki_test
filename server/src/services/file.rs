use minio_rsc::{provider::StaticProvider, Minio};
use bytes::Bytes;
use tracing::info;

use crate::middleware::error::AppError;

pub struct MinIOService {
    client: Minio,
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
        let provider = StaticProvider::new(access_key, secret_key, None);
        
        let client = Minio::builder()
            .host(endpoint)
            .provider(provider)
            .secure(use_ssl)
            .build()
            .map_err(|e| AppError::BadRequest(format!("Invalid MinIO configuration: {}", e)))?;

        Ok(Self {
            client,
            bucket_prefix: bucket_prefix.to_string(),
        })
    }

    fn get_bucket_name(&self, project_id: &str) -> String {
        format!("{}-{}", self.bucket_prefix, project_id)
    }

    pub async fn ensure_bucket(&self, project_id: &str) -> Result<(), AppError> {
        let bucket_name = self.get_bucket_name(project_id);

        let exists = self
            .client
            .bucket_exists(&bucket_name)
            .await
            .map_err(|e| AppError::Internal)?;

        if !exists {
            self.client
                .make_bucket(&bucket_name, false)
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
        let bucket_name = self.get_bucket_name(project_id);
        self.ensure_bucket(project_id).await?;

        let data = Bytes::from(content.to_vec());
        
        let key_args = minio_rsc::client::KeyArgs::new(path)
            .content_type(Some(content_type.to_string()));

        self.client
            .put_object(&bucket_name, key_args, data)
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(())
    }

    pub async fn download_file(&self, project_id: &str, path: &str) -> Result<Vec<u8>, AppError> {
        let bucket_name = self.get_bucket_name(project_id);

        let response = self
            .client
            .get_object(&bucket_name, path)
            .await
            .map_err(|e| AppError::Internal)?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(bytes.to_vec())
    }

    pub async fn delete_file(&self, project_id: &str, path: &str) -> Result<(), AppError> {
        let bucket_name = self.get_bucket_name(project_id);

        self.client
            .remove_object(&bucket_name, path)
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(())
    }

    pub async fn list_files(&self, project_id: &str, prefix: &str) -> Result<Vec<String>, AppError> {
        let bucket_name = self.get_bucket_name(project_id);

        let args = minio_rsc::client::ListObjectsArgs::default()
            .prefix(prefix.to_string())
            .recursive(true);

        let mut objects = self
            .client
            .list_objects(&bucket_name, args)
            .await
            .map_err(|e| AppError::Internal)?;

        let mut files = Vec::new();
        while let Some(result) = objects.next().await {
            match result {
                Ok(item) => {
                    files.push(item.key);
                }
                Err(_) => continue,
            }
        }

        Ok(files)
    }

    pub async fn copy_file(
        &self,
        project_id: &str,
        from_path: &str,
        to_path: &str,
    ) -> Result<(), AppError> {
        let bucket_name = self.get_bucket_name(project_id);

        let src = minio_rsc::client::CopySource::new(&bucket_name, from_path);
        let dst = minio_rsc::client::KeyArgs::new(to_path);

        self.client
            .copy_object(&bucket_name, dst, src)
            .await
            .map_err(|e| AppError::Internal)?;

        Ok(())
    }
}
