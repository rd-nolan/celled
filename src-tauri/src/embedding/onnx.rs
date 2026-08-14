use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};

use crate::embedding::{l2_normalize, EmbeddingProvider};
use crate::error::AppError;

const DEFAULT_VERSION: &str = "bge-small-zh-v1.5@v1";
const MAX_LEN: usize = 64;

/// ONNX Runtime embedding provider. Model is loaded once and reused.
pub struct OnnxEmbeddingProvider {
    session: Mutex<ort::session::Session>,
    tokenizer: Mutex<Tokenizer>,
    model_version: String,
    input_names: Vec<String>,
    output_name: String,
}

impl OnnxEmbeddingProvider {
    pub fn load(model_dir: &Path) -> Result<Self, AppError> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");
        if !model_path.exists() || !tokenizer_path.exists() {
            return Err(AppError::ModelLoadFailed(
                "model.onnx or tokenizer.json not found".into(),
            ));
        }

        let session = ort::session::Session::builder()
            .map_err(|e| AppError::ModelLoadFailed(e.to_string()))?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
            .map_err(|e| AppError::ModelLoadFailed(e.to_string()))?
            .with_intra_threads(2)
            .map_err(|e| AppError::ModelLoadFailed(e.to_string()))?
            .commit_from_file(&model_path)
            .map_err(|e| AppError::ModelLoadFailed(e.to_string()))?;

        let mut tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| AppError::ModelLoadFailed(e.to_string()))?;
        let _ = tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        }));
        let _ = tokenizer.with_truncation(Some(TruncationParams {
            max_length: MAX_LEN,
            ..Default::default()
        }));

        let input_names: Vec<String> = session
            .inputs()
            .iter()
            .map(|input| input.name().to_string())
            .collect();
        let output_name = session
            .outputs()
            .iter()
            .find(|output| {
                let name = output.name();
                name.contains("sentence") || name.contains("embedding")
            })
            .or_else(|| session.outputs().first())
            .map(|output| output.name().to_string())
            .ok_or_else(|| AppError::ModelLoadFailed("ONNX model has no outputs".into()))?;

        let version = std::fs::read_to_string(model_dir.join("config.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
            .and_then(|json| {
                json.get("model_version")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| DEFAULT_VERSION.to_string());

        Ok(Self {
            session: Mutex::new(session),
            tokenizer: Mutex::new(tokenizer),
            model_version: version,
            input_names,
            output_name,
        })
    }
}

impl EmbeddingProvider for OnnxEmbeddingProvider {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, AppError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let encodings = self
            .tokenizer
            .lock()
            .map_err(|_| AppError::ModelInferenceFailed("tokenizer lock poisoned".into()))?
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| AppError::ModelInferenceFailed(e.to_string()))?;

        let batch = encodings.len();
        let seq = encodings.first().map(|e| e.get_ids().len()).unwrap_or(0);
        let mut input_ids = Vec::with_capacity(batch * seq);
        let mut attention_mask = Vec::with_capacity(batch * seq);
        let mut token_type_ids = Vec::with_capacity(batch * seq);
        for encoding in &encodings {
            input_ids.extend(encoding.get_ids().iter().map(|v| *v as i64));
            attention_mask.extend(encoding.get_attention_mask().iter().map(|v| *v as i64));
            let types = encoding.get_type_ids();
            if types.is_empty() {
                token_type_ids.extend(std::iter::repeat(0i64).take(seq));
            } else {
                token_type_ids.extend(types.iter().map(|v| *v as i64));
            }
        }

        let mut session = self
            .session
            .lock()
            .map_err(|_| AppError::ModelInferenceFailed("session lock poisoned".into()))?;

        let mut values: HashMap<String, ort::value::Value> = HashMap::new();
        for name in &self.input_names {
            let data = if name.contains("attention") {
                attention_mask.clone()
            } else if name.contains("token_type") || name.contains("type_id") {
                token_type_ids.clone()
            } else {
                input_ids.clone()
            };
            let tensor = ort::value::Tensor::from_array(([batch, seq], data))
                .map_err(|e| AppError::ModelInferenceFailed(e.to_string()))?;
            values.insert(name.clone(), tensor.into());
        }

        let outputs = session
            .run(values)
            .map_err(|e| AppError::ModelInferenceFailed(e.to_string()))?;

        let (shape, data) = extract_f32_tensor(&outputs, &self.output_name)?;
        mean_pool_or_take(&shape, &data, &attention_mask, batch, seq)
    }

    fn model_version(&self) -> &str {
        &self.model_version
    }

    fn backend_name(&self) -> &str {
        "onnx"
    }
}

fn extract_f32_tensor(
    outputs: &ort::session::SessionOutputs<'_>,
    preferred: &str,
) -> Result<(Vec<i64>, Vec<f32>), AppError> {
    if let Some(output) = outputs.get(preferred) {
        return tensor_to_owned(output);
    }
    for (_, output) in outputs.iter() {
        return tensor_to_owned(&output);
    }
    Err(AppError::ModelInferenceFailed("missing embedding output".into()))
}

fn tensor_to_owned(value: &ort::value::Value) -> Result<(Vec<i64>, Vec<f32>), AppError> {
    let (shape, data) = value
        .try_extract_tensor::<f32>()
        .map_err(|e| AppError::ModelInferenceFailed(e.to_string()))?;
    Ok((shape.to_vec(), data.to_vec()))
}

fn mean_pool_or_take(
    shape: &[i64],
    data: &[f32],
    attention_mask: &[i64],
    batch: usize,
    seq: usize,
) -> Result<Vec<Vec<f32>>, AppError> {
    match shape.len() {
        2 => {
            let hidden = shape[1] as usize;
            let mut result = Vec::with_capacity(batch);
            for row in 0..batch {
                let start = row * hidden;
                let mut vec = data[start..start + hidden].to_vec();
                l2_normalize(&mut vec);
                result.push(vec);
            }
            Ok(result)
        }
        3 => {
            let hidden = shape[2] as usize;
            let mut result = Vec::with_capacity(batch);
            for b in 0..batch {
                let mut acc = vec![0.0f32; hidden];
                let mut count = 0.0f32;
                for t in 0..seq {
                    let mask = attention_mask[b * seq + t];
                    if mask == 0 {
                        continue;
                    }
                    count += 1.0;
                    let offset = (b * seq + t) * hidden;
                    for h in 0..hidden {
                        acc[h] += data[offset + h];
                    }
                }
                if count > 0.0 {
                    for v in acc.iter_mut() {
                        *v /= count;
                    }
                }
                l2_normalize(&mut acc);
                result.push(acc);
            }
            Ok(result)
        }
        _ => Err(AppError::ModelInferenceFailed(format!(
            "unexpected ONNX output rank {}",
            shape.len()
        ))),
    }
}
