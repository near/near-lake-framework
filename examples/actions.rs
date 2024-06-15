//! This example shows how to filter actions in a block.
//! It it a more real-life example than the simple example.
//! It is going to follow the NEAR Social contract and print all function calls to it.
use near_lake_framework::near_lake_primitives;
// We need to import this trait to use the `as_function_call` method.
use near_lake_primitives::actions::ActionMetaDataExt;

const CONTRACT_ID: &str = "social.near";

fn main() -> anyhow::Result<()> {
    eprintln!("Starting...");
    // Lake Framework start boilerplate
    near_lake_framework::LakeBuilder::default()
        .mainnet()
        .start_block_height(88444526)
        .build()?
        // developer-defined async function that handles each block
        .run(print_function_calls_to_my_account)?;
    Ok(())
}

async fn print_function_calls_to_my_account(
    block: near_lake_primitives::block::Block,
) -> anyhow::Result<()> {
    let block_height = block.block_height();
    let actions: Vec<&near_lake_primitives::actions::FunctionCall> = block
        .actions()
        .filter(|action| action.receiver_id().as_str() == CONTRACT_ID)
        .filter_map(|action| action.as_function_call())
        .collect();

    if !actions.is_empty() {
        println!("Block #{:?}\n{:#?}", block_height, actions);
    }

    Ok(())
}
