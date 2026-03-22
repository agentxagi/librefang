//! Mock LLM 驱动 — 用于测试的可配置假 LLM 提供程序。
//!
//! 支持：
//! - 返回固定回复（canned responses）
//! - 记录所有请求以便断言
//! - 模拟流式响应

use async_trait::async_trait;
use librefang_runtime::llm_driver::{
    CompletionRequest, CompletionResponse, LlmDriver, LlmError, StreamEvent,
};
use librefang_types::message::{ContentBlock, StopReason, TokenUsage};
use std::sync::{Arc, Mutex};

/// 记录的 LLM 调用信息。
#[derive(Debug, Clone)]
pub struct RecordedCall {
    /// 请求中的模型名称。
    pub model: String,
    /// 消息数量。
    pub message_count: usize,
    /// 工具定义数量。
    pub tool_count: usize,
    /// 系统提示（如有）。
    pub system: Option<String>,
}

/// Mock LLM 驱动 — 返回可配置的固定回复，并记录所有调用。
pub struct MockLlmDriver {
    /// 固定回复文本列表，按顺序返回。用完后循环使用最后一个。
    responses: Vec<String>,
    /// 已记录的调用。
    calls: Arc<Mutex<Vec<RecordedCall>>>,
    /// 当前回复索引。
    index: Arc<Mutex<usize>>,
    /// 自定义输入 token 数（默认 10）。
    input_tokens: u64,
    /// 自定义输出 token 数（默认 5）。
    output_tokens: u64,
    /// 自定义停止原因（默认 EndTurn）。
    stop_reason: StopReason,
}

impl MockLlmDriver {
    /// 创建一个返回固定回复的 mock driver。
    ///
    /// ```rust
    /// use librefang_testing::MockLlmDriver;
    ///
    /// let driver = MockLlmDriver::new(vec!["你好！".into()]);
    /// ```
    pub fn new(responses: Vec<String>) -> Self {
        assert!(!responses.is_empty(), "MockLlmDriver 需要至少一个固定回复");
        Self {
            responses,
            calls: Arc::new(Mutex::new(Vec::new())),
            index: Arc::new(Mutex::new(0)),
            input_tokens: 10,
            output_tokens: 5,
            stop_reason: StopReason::EndTurn,
        }
    }

    /// 创建始终返回同一回复的 mock driver。
    pub fn with_response(response: impl Into<String>) -> Self {
        Self::new(vec![response.into()])
    }

    /// 设置自定义 token 用量（覆盖默认的 input=10, output=5）。
    ///
    /// ```rust
    /// use librefang_testing::MockLlmDriver;
    ///
    /// let driver = MockLlmDriver::with_response("hi").with_tokens(100, 50);
    /// ```
    pub fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.input_tokens = input;
        self.output_tokens = output;
        self
    }

    /// 设置自定义停止原因（覆盖默认的 EndTurn）。
    ///
    /// ```rust
    /// use librefang_testing::MockLlmDriver;
    /// use librefang_types::message::StopReason;
    ///
    /// let driver = MockLlmDriver::with_response("hi").with_stop_reason(StopReason::MaxTokens);
    /// ```
    pub fn with_stop_reason(mut self, reason: StopReason) -> Self {
        self.stop_reason = reason;
        self
    }

    /// 返回已记录的所有调用。
    pub fn recorded_calls(&self) -> Vec<RecordedCall> {
        self.calls.lock().unwrap().clone()
    }

    /// 返回调用次数。
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// 获取下一个回复文本。
    fn next_response(&self) -> String {
        let mut idx = self.index.lock().unwrap();
        let response = if *idx < self.responses.len() {
            self.responses[*idx].clone()
        } else {
            // 用完后循环使用最后一个
            self.responses.last().unwrap().clone()
        };
        *idx += 1;
        response
    }
}

#[async_trait]
impl LlmDriver for MockLlmDriver {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        // 记录调用
        {
            let call = RecordedCall {
                model: request.model.clone(),
                message_count: request.messages.len(),
                tool_count: request.tools.len(),
                system: request.system.clone(),
            };
            self.calls.lock().unwrap().push(call);
        }

        let text = self.next_response();
        Ok(CompletionResponse {
            content: vec![ContentBlock::Text {
                text,
                provider_metadata: None,
            }],
            stop_reason: self.stop_reason,
            tool_calls: vec![],
            usage: TokenUsage {
                input_tokens: self.input_tokens,
                output_tokens: self.output_tokens,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            },
        })
    }

    async fn stream(
        &self,
        request: CompletionRequest,
        tx: tokio::sync::mpsc::Sender<StreamEvent>,
    ) -> Result<CompletionResponse, LlmError> {
        // 模拟流式：先发送 TextDelta，再发送 ContentComplete
        let response = self.complete(request).await?;
        let text = response.text();
        if !text.is_empty() {
            let _ = tx.send(StreamEvent::TextDelta { text }).await;
        }
        let _ = tx
            .send(StreamEvent::ContentComplete {
                stop_reason: response.stop_reason,
                usage: response.usage,
            })
            .await;
        Ok(response)
    }

    fn is_configured(&self) -> bool {
        true
    }
}

/// 始终返回错误的 mock driver，用于测试错误处理。
pub struct FailingLlmDriver {
    error_message: String,
}

impl FailingLlmDriver {
    /// 创建一个始终返回指定错误的 driver。
    pub fn new(error_message: impl Into<String>) -> Self {
        Self {
            error_message: error_message.into(),
        }
    }
}

#[async_trait]
impl LlmDriver for FailingLlmDriver {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Err(LlmError::Api {
            status: 500,
            message: self.error_message.clone(),
        })
    }

    fn is_configured(&self) -> bool {
        false
    }
}
