use crate::middleware::error::AppError;
use std::io::Cursor;
use std::sync::Arc;

pub trait DocumentProcessor: Send + Sync {
    fn supports(&self, extension: &str) -> bool;
    
    fn content_type(&self) -> &'static str;
    
    fn extract_to_markdown(&self, content: &[u8]) -> Result<String, AppError>;
}

pub struct PdfProcessor;

impl DocumentProcessor for PdfProcessor {
    fn supports(&self, extension: &str) -> bool {
        extension.eq_ignore_ascii_case("pdf")
    }
    
    fn content_type(&self) -> &'static str {
        "application/pdf"
    }
    
    fn extract_to_markdown(&self, content: &[u8]) -> Result<String, AppError> {
        let doc = lopdf::Document::load_from(Cursor::new(content))
            .map_err(|e| AppError::BadRequest(format!("Failed to parse PDF: {}", e)))?;

        let mut text = String::new();
        for page in doc.get_pages() {
            let page_number = page.0;
            if let Ok(page_text) = doc.extract_text(&[page_number]) {
                text.push_str(&page_text);
                text.push_str("\n\n");
            }
        }

        Ok(text)
    }
}

pub struct ExcelProcessor;

impl DocumentProcessor for ExcelProcessor {
    fn supports(&self, extension: &str) -> bool {
        extension.eq_ignore_ascii_case("xlsx") || extension.eq_ignore_ascii_case("xls")
    }
    
    fn content_type(&self) -> &'static str {
        "application/vnd.ms-excel"
    }
    
    fn extract_to_markdown(&self, content: &[u8]) -> Result<String, AppError> {
        use calamine::Reader;
        
        let mut workbook = calamine::Xlsx::<Cursor<&[u8]>>::new(Cursor::new(content))
            .map_err(|e| AppError::BadRequest(format!("Failed to parse Excel: {}", e)))?;

        let mut text = String::new();
        for sheet_name in workbook.sheet_names() {
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                text.push_str(&format!("## Sheet: {}\n\n", sheet_name));
                for row in range.rows() {
                    let row_text: Vec<String> = row
                        .iter()
                        .map(|cell| cell.to_string())
                        .collect();
                    text.push_str(&row_text.join("\t"));
                    text.push_str("\n");
                }
                text.push_str("\n");
            }
        }

        Ok(text)
    }
}

pub struct TextProcessor {
    content_type: &'static str,
    supported_extensions: Vec<&'static str>,
}

impl TextProcessor {
    pub fn new(content_type: &'static str, supported_extensions: Vec<&'static str>) -> Self {
        Self {
            content_type,
            supported_extensions,
        }
    }
}

impl DocumentProcessor for TextProcessor {
    fn supports(&self, extension: &str) -> bool {
        self.supported_extensions
            .iter()
            .any(|ext| extension.eq_ignore_ascii_case(ext))
    }
    
    fn content_type(&self) -> &'static str {
        self.content_type
    }
    
    fn extract_to_markdown(&self, content: &[u8]) -> Result<String, AppError> {
        String::from_utf8(content.to_vec())
            .map_err(|e| AppError::BadRequest(format!("Failed to decode text: {}", e)))
    }
}

use once_cell::sync::Lazy;

pub static PROCESSOR_REGISTRY: Lazy<Vec<Arc<dyn DocumentProcessor>>> = Lazy::new(|| {
    vec![
        Arc::new(PdfProcessor),
        Arc::new(ExcelProcessor),
        Arc::new(TextProcessor::new("text/markdown", vec!["md"])),
        Arc::new(TextProcessor::new("text/plain", vec!["txt", "log"])),
        Arc::new(TextProcessor::new("application/json", vec!["json"])),
        Arc::new(TextProcessor::new("application/xml", vec!["xml"])),
        Arc::new(TextProcessor::new("text/csv", vec!["csv"])),
    ]
});

pub fn get_processor_by_extension(extension: &str) -> Option<Arc<dyn DocumentProcessor>> {
    PROCESSOR_REGISTRY
        .iter()
        .find(|processor| processor.supports(extension))
        .cloned()
}

pub fn get_processor_by_path(path: &str) -> Option<Arc<dyn DocumentProcessor>> {
    path.rsplit('.')
        .next()
        .and_then(|ext| get_processor_by_extension(ext))
}