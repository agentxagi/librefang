//! # librefang-testing — 测试基础设施
//!
//! 提供用于单元测试 API 路由的 mock 基础设施，无需启动完整的 daemon。
//!
//! ## 主要组件
//!
//! - [`MockKernelBuilder`] — 构建最小化的 `LibreFangKernel`（内存 SQLite，临时目录）
//! - [`MockLlmDriver`] — 可配置的 LLM 驱动 mock，支持录制调用和固定回复
//! - [`TestAppState`] — 构建适用于 axum 测试的 `AppState`
//! - 辅助函数 — `test_request`、`assert_json_ok`、`assert_json_error`

pub mod helpers;
pub mod mock_driver;
pub mod mock_kernel;
pub mod test_app;

pub use helpers::{assert_json_error, assert_json_ok, test_request};
pub use mock_driver::{FailingLlmDriver, MockLlmDriver};
pub use mock_kernel::MockKernelBuilder;
pub use test_app::TestAppState;

#[cfg(test)]
mod tests;
