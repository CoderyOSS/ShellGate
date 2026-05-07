use crate::pipeline::BonsaiConfig;
use crate::types::GateError;

#[cfg(feature = "bonsai")]
use candle_core::{Device, Tensor};
#[cfg(feature = "bonsai")]
use std::sync::Mutex;

#[cfg(feature = "bonsai")]
pub struct BonsaiModel {
    device: Device,
    model_path: String,
    tokenizer: Option<tokenizers::Tokenizer>,
    config: BonsaiConfig,
}

#[cfg(not(feature = "bonsai"))]
pub struct BonsaiModel;

impl BonsaiModel {
    #[cfg(feature = "bonsai")]
    pub fn load(config: &BonsaiConfig) -> Result<Self, GateError> {
        if !std::path::Path::new(&config.model_path).exists() {
            tracing::warn!(path = %config.model_path, "bonsai model file not found, LLM stage will be skipped");
            return Ok(Self {
                device: Device::Cpu,
                model_path: config.model_path.clone(),
                tokenizer: None,
                config: config.clone(),
            });
        }

        let device = Device::Cpu;

        let tokenizer = tokenizers::Tokenizer::from_file(&config.model_path)
            .or_else(|_| {
                let tokenizer_path = std::path::Path::new(&config.model_path)
                    .parent()
                    .map(|p| p.join("tokenizer.json"))
                    .unwrap_or_else(|| std::path::PathBuf::from("tokenizer.json"));
                tokenizers::Tokenizer::from_file(&tokenizer_path).map_err(|e| {
                    format!("failed to load tokenizer from {:?}: {}", tokenizer_path, e)
                })
            })
            .ok();

        if tokenizer.is_none() {
            tracing::warn!("no tokenizer found, bonsai inference will not be available");
        }

        tracing::info!(
            path = %config.model_path,
            size = %config.model_size,
            "bonsai model loaded"
        );

        Ok(Self {
            device,
            model_path: config.model_path.clone(),
            tokenizer,
            config: config.clone(),
        })
    }

    #[cfg(not(feature = "bonsai"))]
    pub fn load(_config: &BonsaiConfig) -> Result<Self, GateError> {
        tracing::warn!("bonsai feature disabled, LLM stage will be skipped");
        Ok(Self)
    }

    pub fn is_available(&self) -> bool {
        #[cfg(feature = "bonsai")]
        {
            self.tokenizer.is_some()
        }
        #[cfg(not(feature = "bonsai"))]
        {
            false
        }
    }

    #[cfg(feature = "bonsai")]
    pub fn infer(&self, prompt: &str) -> Result<String, GateError> {
        let tokenizer = match &self.tokenizer {
            Some(t) => t,
            None => return Err("no tokenizer available".into()),
        };

        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| format!("tokenization error: {}", e))?;

        let token_ids = tokens.get_ids();
        tracing::debug!(tokens = token_ids.len(), "bonsai inference started");

        let generated = self.run_inference(token_ids)?;

        let decoded = tokenizer
            .decode(generated.as_slice(), true)
            .map_err(|e| format!("decode error: {}", e))?;

        Ok(decoded)
    }

    #[cfg(feature = "bonsai")]
    fn run_inference(&self, token_ids: &[u32]) -> Result<Vec<u32>, GateError> {
        use candle_transformers::models::quantized_llama::ModelWeights;

        let model_weights = ModelWeights::from_gguf_file(&self.model_path, &self.device)
            .map_err(|e| format!("failed to load GGUF model: {}", e))?;

        let mut output_tokens = Vec::new();
        let mut input_tensor = Tensor::new(token_ids, &self.device)?;

        for _ in 0..self.config.max_tokens {
            let logits = model_weights
                .forward(&input_tensor, 0)
                .map_err(|e| format!("forward pass error: {}", e))?;

            let next_token = logits
                .argmax(candle_core::D::Minus1)?
                .to_vec0::<u32>()?;

            output_tokens.push(next_token);

            if next_token == tokenizers::constants::eos_token() {
                break;
            }

            input_tensor = Tensor::new(&[next_token], &self.device)?;
        }

        Ok(output_tokens)
    }

    #[cfg(not(feature = "bonsai"))]
    pub fn infer(&self, _prompt: &str) -> Result<String, GateError> {
        Err("bonsai feature not compiled".into())
    }
}
