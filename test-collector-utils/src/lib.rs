use std::future::Future;
use std::pin::Pin;

type AsyncFn = Box<dyn Fn() -> Pin<Box<dyn Future<Output=()>>>>;

pub struct IntegrationTestMeta{
    pub name: String,
    pub sync_fn: Option<fn()>,
    pub async_fn: Option<AsyncFn>,
}

pub trait IntegrationTestRunner {
    fn block_on<F: Future>(&self, fut: F) -> F::Output;
}

impl IntegrationTestMeta {
    pub fn for_sync_fn(name: String, function: fn()) -> IntegrationTestMeta {
        IntegrationTestMeta {
            name,
            sync_fn: Some(function),
            async_fn: None
        }
    }

    pub fn for_async_fn(name: String, function: AsyncFn) -> IntegrationTestMeta {
        IntegrationTestMeta {
            name,
            sync_fn: None,
            async_fn: Some(function),
        }
    }
}

inventory::collect!(IntegrationTestMeta);