#![doc = include_str!("../README.md")]
#[macro_use]
extern crate derive_builder;

use futures::{Future, StreamExt};

pub use near_lake_context_derive::LakeContext;
pub use near_lake_primitives::{
    self,
    near_indexer_primitives::{self, near_primitives},
};

pub use aws_credential_types::Credentials;
pub use types::{Lake, LakeBuilder, LakeContextExt, LakeError};

mod s3_fetchers;
mod streamer;
pub(crate) mod types;

pub(crate) const LAKE_FRAMEWORK: &str = "near_lake_framework";

impl types::Lake {
    /// Creates `mpsc::channel` and returns the `receiver` to read the stream of `StreamerMessage`
    ///```no_run
    ///  # use near_lake_framework::{LakeContext};
    ///
    /// #[derive(LakeContext)]
    ///  struct MyContext {
    ///      my_field: String,
    ///  }
    ///
    ///# fn main() -> anyhow::Result<()> {
    ///
    ///    let context = MyContext {
    ///       my_field: "my_value".to_string(),
    ///    };
    ///
    ///    near_lake_framework::LakeBuilder::default()
    ///        .testnet()
    ///        .start_block_height(112205773)
    ///        .build()?
    ///        .run_with_context(handle_block, &context)?;
    ///    Ok(())
    ///# }
    ///
    /// # async fn handle_block(_block: near_lake_primitives::block::Block, context: &MyContext) -> anyhow::Result<()> { Ok(()) }
    ///```
    pub fn run_with_context<'context, C: LakeContextExt, E, Fut>(
        self,
        f: impl Fn(near_lake_primitives::block::Block, &'context C) -> Fut,
        context: &'context C,
    ) -> Result<(), LakeError>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<Box<dyn std::error::Error>>,
    {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|err| LakeError::RuntimeStartError { error: err })?;

        runtime.block_on(async move { self.run_with_context_async(f, context).await })
    }

    pub async fn run_with_context_async<'context, C: LakeContextExt, E, Fut>(
        self,
        f: impl Fn(near_lake_primitives::block::Block, &'context C) -> Fut,
        context: &'context C,
    ) -> Result<(), LakeError>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<Box<dyn std::error::Error>>,
    {
        // capture the concurrency value before it moves into the streamer
        let concurrency = self.concurrency;

        // instantiate the NEAR Lake Framework Stream
        let (sender, stream) = streamer::streamer(self);

        // read the stream events and pass them to a handler function with
        // concurrency 1
        let mut handlers = tokio_stream::wrappers::ReceiverStream::new(stream)
            .map(|streamer_message| async {
                let mut block: near_lake_primitives::block::Block = streamer_message.into();

                context.execute_before_run(&mut block);

                let user_indexer_function_execution_result = f(block, context).await;

                context.execute_after_run();

                user_indexer_function_execution_result
            })
            .buffer_unordered(concurrency);

        while let Some(_handle_message) = handlers.next().await {}
        drop(handlers); // close the channel so the sender will stop

        // propagate errors from the sender
        match sender.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(err.into()), // JoinError
        }
    }

    /// Creates `mpsc::channel` and returns the `receiver` to read the stream of `StreamerMessage`
    ///```no_run
    ///# fn main() -> anyhow::Result<()> {
    ///    near_lake_framework::LakeBuilder::default()
    ///        .testnet()
    ///        .start_block_height(112205773)
    ///        .build()?
    ///        .run(handle_block)?;
    ///    Ok(())
    ///# }
    ///
    /// # async fn handle_block(_block: near_lake_primitives::block::Block) -> anyhow::Result<()> { Ok(()) }
    ///```
    pub fn run<Fut, E>(
        self,
        f: impl Fn(near_lake_primitives::block::Block) -> Fut,
    ) -> Result<(), LakeError>
    where
        Fut: Future<Output = Result<(), E>>,
        E: Into<Box<dyn std::error::Error>>,
    {
        struct EmptyContext {}

        impl LakeContextExt for EmptyContext {
            fn execute_before_run(&self, _block: &mut near_lake_primitives::block::Block) {}

            fn execute_after_run(&self) {}
        }

        let context = EmptyContext {};

        self.run_with_context(|block, _context| f(block), &context)
    }
}
