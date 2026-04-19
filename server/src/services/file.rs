use std::collections::HashMap;

use minio_rsc::client::{BucketArgs, CopySource, KeyArgs, ListObjectsArgs, Minio};
use minio_rsc::provider::StaticProvider;
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
        info!("Initializing MinIO service with endpoint: {}", endpoint);
        
        let provider = StaticProvider::new(access_key, secret_key, None);
        let client = Minio::builder()
            .endpoint(endpoint)
            .provider(provider)
            .secure(use_ssl)
            .build()
            .map_err(|e| AppError::BadRequest(format!("Failed to create MinIO client: {}", e)))?;
        
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
        info!("Ensuring bucket exists: {}", bucket_name);
        
        let exists = self.client.bucket_exists(BucketArgs::new(&bucket_name)).await
            .map_err(|e| AppError::BadRequest(format!("Failed to check bucket: {}", e)))?;
        
        if !exists {
            info!("Creating bucket: {}", bucket_name);
            self.client.make_bucket(BucketArgs::new(&bucket_name), false).await
                .map_err(|e| AppError::BadRequest(format!("Failed to create bucket: {}", e)))?;
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
        info!("Uploading file to {}/{}: {} bytes", bucket_name, path, content.len());
        self.ensure_bucket(project_id).await?;
        
        let mut metadata = HashMap::new();
        metadata.insert("content-type".to_string(), content_type.to_string());
        
        let key_args = KeyArgs::new(path)
            .content_type(Some(content_type.to_string()))
            .metadata(metadata);
        
        self.client.put_object(&bucket_name, key_args, content.to_vec().into()).await
            .map_err(|e| AppError::BadRequest(format!("Failed to upload file: {}", e)))?;
        
        Ok(())
    }

    pub async fn download_file(&self, project_id: &str, path: &str) -> Result<Vec<u8>, AppError> {
        let bucket_name = self.get_bucket_name(project_id);
        info!("Downloading file from {}/{}", bucket_name, path);
        
        let response = self.client.get_object(&bucket_name, KeyArgs::new(path)).await
            .map_err(|e| AppError::BadRequest(format!("Failed to download file: {}", e)))?;
        
        let bytes = response.bytes().await
            .map_err(|e| AppError::BadRequest(format!("Failed to read response: {}", e)))?;
        
        Ok(bytes.to_vec())
    }

    pub async fn delete_file(&self, project_id: &str, path: &str) -> Result<(), AppError> {
        let bucket_name = self.get_bucket_name(project_id);
        info!("Deleting file from {}/{}", bucket_name, path);
        
        self.client.remove_object(&bucket_name, KeyArgs::new(path)).await
            .map_err(|e| AppError::BadRequest(format!("Failed to delete file: {}", e)))?;
        
        Ok(())
    }

    pub async fn list_files(&self, project_id: &str, prefix: &str) -> Result<Vec<String>, AppError> {
        let bucket_name = self.get_bucket_name(project_id);
        info!("Listing files in {}/{}", bucket_name, prefix);
        
        let args = ListObjectsArgs::default().prefix(prefix.to_string());
        let result = self.client.list_objects(&bucket_name, args).await
            .map_err(|e| AppError::BadRequest(format!("Failed to list files: {}", e)))?;
        
        let files = result.contents.iter().map(|obj| obj.key.clone()).collect();
        
        Ok(files)
    }

    pub async fn copy_file(
        &self,
        project_id: &str,
        from_path: &str,
        to_path: &str,
    ) -> Result<(), AppError> {
        let bucket_name = self.get_bucket_name(project_id);
        info!("Copying file in {} from {} to {}", bucket_name, from_path, to_path);
        
        let source = CopySource::new(&bucket_name, from_path);
        self.client.copy_object(&bucket_name, KeyArgs::new(to_path), source).await
            .map_err(|e| AppError::BadRequest(format!("Failed to copy file: {}", e)))?;
        
        Ok(())
    }
}
