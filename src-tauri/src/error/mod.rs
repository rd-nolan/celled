use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("找不到文件")]
    FileNotFound,
    #[error("不支持的 Excel 文件")]
    UnsupportedExcel,
    #[error("找不到工作表")]
    SheetNotFound,
    #[error("工作表为空")]
    EmptyWorksheet,
    #[error("未能识别表头")]
    HeaderNotFound,
    #[error("表头行无效")]
    InvalidHeaderRow,
    #[error("映射无效：{0}")]
    InvalidMapping(String),
    #[error("{0}")]
    MappingConflict(String),
    #[cfg(feature = "onnx")]
    #[error("模型加载失败：{0}")]
    ModelLoadFailed(String),
    #[cfg(feature = "onnx")]
    #[error("模型推理失败：{0}")]
    ModelInferenceFailed(String),
    #[error("数据库错误：{0}")]
    DatabaseError(String),
    #[error("Excel 写入失败：{0}")]
    ExcelWriteError(String),
    #[error("尚未确认模板")]
    TemplateNotConfirmed,
    #[error("找不到导入会话")]
    SessionNotFound,
    #[error("仍有文件未确认映射，无法开始转换")]
    NotAllConfirmed,
    #[error("无法读取 Excel：{0}")]
    ExcelRead(String),
    #[error("{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<calamine::Error> for AppError {
    fn from(value: calamine::Error) -> Self {
        match value {
            calamine::Error::Io(_) => AppError::FileNotFound,
            other => AppError::ExcelRead(other.to_string()),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        AppError::DatabaseError(value.to_string())
    }
}

impl From<uuid::Error> for AppError {
    fn from(value: uuid::Error) -> Self {
        AppError::Internal(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        if value.kind() == std::io::ErrorKind::NotFound {
            AppError::FileNotFound
        } else {
            AppError::Internal(value.to_string())
        }
    }
}
