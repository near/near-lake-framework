# NEAR Lake Parent Transaction Cache (Context)

Lake Parent Transaction Cache is a ready-to-use context for the Lake Framework in Rust. It provides a cache for keeping the relation between transactions and receipts in cache.

## Example Usage

```no_run
use near_lake_parent_transaction_cache::{ParentTransactionCache, ParentTransactionCacheBuilder};
# use near_lake_framework::LakeBuilder;
# use near_lake_framework::near_lake_primitives::{block::Block, actions::ActionMetaDataExt};

# fn main() {
let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default()
    .build()
    .expect("Failed to build the ParentTransactionCache context");

LakeBuilder::default()
    .mainnet()
    .start_block_height(80504433)
    .build()
    .expect("Failed to build the Lake Framework")
    .run_with_context(handle_block, &parent_transaction_cache_ctx)
    .expect("Failed to run the Lake Framework");
# }

async fn handle_block(
    block: Block,
    ctx: &ParentTransactionCache,
) -> anyhow::Result<()> {
    for action in block.actions() {
        println!(
            "Action receipt ID: {:?} | Parent TX hash: {:?}",
            action.receipt_id(),
            ctx.get_parent_transaction_hash(&action.receipt_id())
        );
    }
    Ok(())
}
```

## Getting Started

To use the Lake Parent Transaction Cache context in your Rust project, follow these steps:

1. Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
near-lake-parent-transaction-cache = "<version>"
```

2. Import the necessary modules in your code:

```ignore
use near_lake_parent_transaction_cache::ParentTransactionCache;
use near_lake_primitives::actions::ActionMetaDataExt;
```

3. Create an instance of the `ParentTransactionCache` context:

```no_run
# use near_lake_parent_transaction_cache::ParentTransactionCacheBuilder;
let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default();
```

4. Configure the Lake Framework and run it with the created context:

```ignore
near_lake_framework::LakeBuilder::default()
    .mainnet()
    .start_block_height(<desired_block_height>)
    .build()?
    .run_with_context(<your_indexing_function>, &parent_transaction_cache_ctx)?;
```

Replace `<desired_block_height>` with the starting block height you want to use. Replace `<you_indexing_function>` with the function you want to use to index the blocks.

## Advanced Usage

### Cache size

We use [SizedCache](https://docs.rs/cached/0.43.0/cached/stores/struct.SizedCache.html) under the hood. So we can configure the cache size by using the `cache_size` method:

```no_run
# use near_lake_parent_transaction_cache::ParentTransactionCacheBuilder;
let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default()
    .cache_size(100_000);
```

By default the cache size is 100,000.

### Watch for specific accounts

By default `ParentTransactionCache` context will cache the relation between Transaction and Receipt for every Transaction in the block. But you can configure it to watch for specific accounts only:

#### You can pass a Vec of AccountId

```no_run
# use near_lake_parent_transaction_cache::ParentTransactionCacheBuilder;
use near_lake_framework::near_primitives::types::AccountId;

let accounts_to_watch: Vec<AccountId> = vec![
    String::from("alice.near").try_into().unwrap(),
    String::from("bob.near").try_into().unwrap(),
];
let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default()
    .for_accounts(accounts_to_watch);
```

#### You can pass accounts to watch one by one using `for_account` method

```no_run
# use near_lake_parent_transaction_cache::ParentTransactionCacheBuilder;
use near_lake_framework::near_primitives::types::AccountId;

let parent_transaction_cache_ctx = ParentTransactionCacheBuilder::default()
    .for_account(String::from("alice.near").try_into().unwrap())
    .for_account(String::from("bob.near").try_into().unwrap());
```

