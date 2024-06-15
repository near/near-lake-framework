//! This example show how to use a context with Lake Framework.
//! It is going to follow the NEAR Social contract and the block height along
//! with a number of calls to the contract.
use near_lake_framework::{near_lake_primitives, LakeContext};
use std::io::Write;
// We need to import this trait to use the `as_function_call` method.
use near_lake_primitives::actions::ActionMetaDataExt;

const CONTRACT_ID: &str = "social.near";

// This is the context we're going to use.
// Lake::run_with_context requires the context to implement the LakeContext trait.
// That trait requires to implement two methods `execute_before_run` and `execute_after_run`.
// However, we don't actually need them in our cause of using the context.
// That's why we're using the derive macro to implement the trait for us.
// The macro will generate the default implementation of the methods. Those methods are empty.
// By doing so, we don't need to implement the trait manually and can use the context as is.
#[derive(Clone, LakeContext)]
struct FileContext {
    path: std::path::PathBuf,
}

impl FileContext {
    fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }

    // append to the file
    pub fn write(&self, value: &str) -> anyhow::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(value.as_bytes())?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    println!("Starting...");
    // Create the context
    let context = FileContext::new("./output.txt");
    // Lake Framework start boilerplate
    near_lake_framework::LakeBuilder::default()
        .mainnet()
        .start_block_height(88444526)
        .build()?
        // developer-defined async function that handles each block
        .run_with_context(print_function_calls_to_my_account, &context)?;
    Ok(())
}

async fn print_function_calls_to_my_account(
    block: near_lake_primitives::block::Block,
    ctx: &FileContext,
) -> anyhow::Result<()> {
    let block_height = block.block_height();
    let actions: Vec<&near_lake_primitives::actions::FunctionCall> = block
        .actions()
        .filter(|action| action.receiver_id().as_str() == CONTRACT_ID)
        .filter_map(|action| action.as_function_call())
        .collect();

    if !actions.is_empty() {
        // Here's the usage of the context.
        ctx.write(
            format!(
                "Block #{} - {} calls to {}\n",
                block_height,
                actions.len(),
                CONTRACT_ID
            )
            .as_str(),
        )?;
        println!("Block #{:?}\n{:#?}", block_height, actions);
    }

    Ok(())
}
