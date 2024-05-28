use candle_core::Device;
use indexmap::IndexMap;
use mistralrs::{
    Constraint, DeviceMapMetadata, GGUFLoaderBuilder, GGUFSpecificConfig, MistralRs,
    MistralRsBuilder, NormalRequest, Request, RequestMessage, SamplingParams, SchedulerMethod,
    TokenSource,
};
use std::sync::Arc;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone)]
pub struct CompletionModel {
    pub mistralrs: Arc<MistralRs>,
}

impl CompletionModel {
    pub fn new() -> anyhow::Result<Self> {
        // Select a Mistral model
        let loader = GGUFLoaderBuilder::new(
            GGUFSpecificConfig { repeat_last_n: 64 },
            None,
            None,
            Some("mistralai/Mistral-7B-Instruct-v0.2".to_string()),
            "TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string(),
            "mistral-7b-instruct-v0.2.Q4_K_M.gguf".to_string(),
        )
        .build();

        let pipeline = loader.load_model_from_hf(
            None,
            TokenSource::CacheToken,
            None,
            &Self::device()?,
            false,
            DeviceMapMetadata::dummy(),
            None,
        )?;
        // Create the MistralRs, which is a runner
        Ok(Self {
            mistralrs: MistralRsBuilder::new(
                pipeline,
                SchedulerMethod::Fixed(5.try_into().unwrap()),
            )
            .build(),
        })
    }

    #[cfg(feature = "metal")]
    fn device() -> anyhow::Result<Device> {
        Ok(Device::new_metal(0)?)
    }

    #[cfg(not(feature = "metal"))]
    fn device() -> anyhow::Result<Device> {
        Ok(Device::cuda_if_available(0)?)
    }

    pub async fn complete(
        &self,
        request: &str,
    ) -> anyhow::Result<ReceiverStream<mistralrs::Response>> {
        let (tx, rx) = channel(10_000);

        let mut messages = Vec::new();
        let mut message_map = IndexMap::new();
        message_map.insert("role".to_string(), "user".to_string());
        message_map.insert("content".to_string(), request.to_string());
        messages.push(message_map);
        let request = Request::Normal(NormalRequest {
            messages: RequestMessage::Chat(messages),
            sampling_params: SamplingParams::default(),
            response: tx,
            return_logprobs: false,
            is_streaming: true,
            id: self.mistralrs.next_request_id(),
            constraint: Constraint::None,
            suffix: None,
            adapters: None,
        });

        self.mistralrs.get_sender().send(request).await?;

        Ok(ReceiverStream::new(rx))
    }
}
