//! This example show how to use a context ParentTransactionCache with the Lake Framework.
//! It is going to follow the NEAR Social contract and cache the parent Transaction for the Receipts.
//! Thus we would be able to capture the Transaction where the change to the contract state has started.
//! **WARNING**: ParentTransactionCache captures all the transactions in the block.
//! That's why we filter it by only one account we're watching here.
use near_lake_framework::near_lake_primitives;
use near_lake_primitives::CryptoHash;
// We need to import this trait to use the `as_function_call` method.
use near_lake_parent_transaction_cache::{ParentTransactionCache, ParentTransactionCacheBuilder};
use near_lake_primitives::actions::ActionMetaDataExt;

const CONTRACT_ID: &str = "social.near";

fn main() -> anyhow::Result<()> {
    println!("Starting...");
    // Building the ParentTransactionCache context.
    // The way of instantiation of the context depends on the implementation developers choose.
    // ParentTransactionCache follows the Builder pattern.
    // This will create the context with the default size of the cache (100_000)
    // and a filter for the account we're watching.
    // It will omit caching all the transactions that are not related to the account.
    let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default()
        .for_account(String::from(CONTRACT_ID).try_into()?)
        .build()?;
    // Lake Framework start boilerplate
    near_lake_framework::LakeBuilder::default()
        .mainnet()
        .start_block_height(88444526)
        .build()?
        // developer-defined async function that handles each block
        .run_with_context(print_function_call_tx_hash, &parent_transaction_cache_ctx)?;
    Ok(())
}

async fn print_function_call_tx_hash(
    block: near_lake_primitives::block::Block,
    ctx: &ParentTransactionCache,
) -> anyhow::Result<()> {
    // Cache has been updated before this function is called.
    let block_height = block.block_height();
    let actions: Vec<(
        &near_lake_primitives::actions::FunctionCall,
        Option<CryptoHash>,
    )> = block
        .actions()
        .filter(|action| action.receiver_id().as_str() == CONTRACT_ID)
        .filter_map(|action| action.as_function_call())
        .map(|action| {
            (
                action,
                ctx.get_parent_transaction_hash(&action.receipt_id()),
            )
        })
        .collect();

    if !actions.is_empty() {
        // Here's the usage of the context.
        println!("Block #{:?}\n{:#?}", block_height, actions);
    }

    Ok(())
}
