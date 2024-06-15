# NEAR Lake Context Derive

Lake Context Derive is a Rust crate that provides a derive macro for easy and convenient implementation of the `near_lake_framework::LakeContextExt` trait. This trait has two functions: `execute_before_run` and `execute_after_run` that are executed before and after the user-provided indexer function respectively.

## Usage

The Lake Context Derive macro can be utilized by annotating the context struct with `#[derive(LakeContext)]`. This trait implementation will then facilitate the combination of different contexts. For instance, to use a `ParentTransactionCache` with some additional data, one would define a context like:

```ignore
use near_lake_parent_transaction_cache::ParentTransactionCache;

#[derive(LakeContext)]
struct MyContext {
  db_connection_string: String,
  parent_tx_cache: ParentTransactionCache,
}
```

### Instantiation

You can create an instance of your context as follows:

```ignore
use near_lake_parent_transaction_cache::{ParentTransactionCacheBuilder};

let my_context = MyContext {
  db_connection_string: String::from("postgres://user:pass@host/db"),
  parent_tx_cache: ParentTransactionCacheBuilder::default().build().unwrap(),
};
```

### User Indexer Function

This will simplify your indexer function signature. It now needs only the context as an additional parameter:

```ignore
async fn handle_block(
    block: Block,
    ctx: &MyContext,
) -> anyhow::Result<()> {
  // body
}
```

The Lake Context Derive will look for all fields in the struct that implement `LakeContextExt`, and will append their trait methods to the top-level calls. For `execute_before_run`, it is done in ascending order, and for `execute_after_run` in descending order.

## Purpose

The purpose of the Lake Context Derive crate is to alleviate some of the common pain points in context development and usage in Rust. By encapsulating and standardizing the handling of these function calls, we aim to create a more accessible and user-friendly approach to context implementation.

## Collaboration

We hope that this tool will be useful for the Rust community and look forward to seeing how it can be used in a range of different projects. We encourage community contributions, whether that's through sharing your own unique context implementations or by providing feedback and suggestions for how we can continue to improve the Lake Context Derive.
