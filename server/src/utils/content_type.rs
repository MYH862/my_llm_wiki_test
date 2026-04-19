use std::collections::HashMap;
use once_cell::sync::Lazy;

static CONTENT_TYPE_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    
    map.insert("pdf", "application/pdf");
    map.insert("doc", "application/msword");
    map.insert("docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
    map.insert("xls", "application/vnd.ms-excel");
    map.insert("xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
    map.insert("ppt", "application/vnd.ms-powerpoint");
    map.insert("pptx", "application/vnd.openxmlformats-officedocument.presentationml.presentation");
    
    map.insert("md", "text/markdown");
    map.insert("txt", "text/plain");
    map.insert("json", "application/json");
    map.insert("xml", "application/xml");
    map.insert("csv", "text/csv");
    map.insert("log", "text/plain");
    
    map.insert("jpg", "image/jpeg");
    map.insert("jpeg", "image/jpeg");
    map.insert("png", "image/png");
    map.insert("gif", "image/gif");
    map.insert("svg", "image/svg+xml");
    
    map.insert("zip", "application/zip");
    map.insert("tar", "application/x-tar");
    map.insert("gz", "application/gzip");
    
    map
});

pub fn get_content_type_by_extension(extension: &str) -> &'static str {
    CONTENT_TYPE_MAP
        .get(&extension.to_lowercase().as_str())
        .copied()
        .unwrap_or("application/octet-stream")
}

pub fn get_content_type_by_path(path: &str) -> String {
    path.rsplit('.')
        .next()
        .map(|ext| get_content_type_by_extension(ext))
        .unwrap_or("application/octet-stream")
        .to_string()
}