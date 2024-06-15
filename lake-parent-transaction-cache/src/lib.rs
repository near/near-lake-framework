#![doc = include_str!("../README.md")]
#[macro_use]
extern crate derive_builder;

use cached::{Cached, SizedCache};
use near_lake_framework::{
    near_indexer_primitives::{near_primitives::types::AccountId, CryptoHash},
    near_lake_primitives::{actions::ActionMetaDataExt, block::Block},
    LakeContextExt,
};

pub type ReceiptId = CryptoHash;
pub type TransactionHash = CryptoHash;
type Cache = SizedCache<ReceiptId, TransactionHash>;

#[derive(Debug, Builder)]
#[builder(pattern = "owned")]
pub struct ParentTransactionCache {
    #[builder(
        setter(custom = true, name = "cache_size"),
        default = "std::sync::RwLock::new(Cache::with_size(100_000))"
    )]
    cache: std::sync::RwLock<Cache>,
    #[builder(setter(custom = true, name = "for_accounts"))]
    account_ids: Vec<AccountId>,
}

impl ParentTransactionCacheBuilder {
    /// Sets the size of the cache. Default is 100_000.
    pub fn cache_size(mut self, value: usize) -> Self {
        self.cache = Some(std::sync::RwLock::new(Cache::with_size(value)));
        self
    }

    /// Stores the Vec of [AccountId](near_lake_framework::near_indexer_primitives::near_primitives::types::AccountId) to cache transactions for.
    /// If not set, the cache will be created for all the Transactions in the block.
    /// If set the cache will be created only for the transactions that have the
    /// sender or receiver in the list of accounts.
    /// **Warning**: This method overrides the previous value.
    pub fn for_accounts(mut self, accounts_id: Vec<AccountId>) -> Self {
        self.account_ids = Some(accounts_id);
        self
    }

    /// Adds an account to the watching list for the parent transaction cache.
    /// Similarly to the method [for_accounts](#method.for_accounts) this method will
    /// create the cache only for the transactions that have the sender or receiver
    /// in the list of accounts.
    /// **Warning**: This method appends to the previous value.
    pub fn for_account(mut self, account_id: AccountId) -> Self {
        if let Some(mut accounts_id) = self.account_ids.take() {
            accounts_id.push(account_id);
            self.account_ids = Some(accounts_id);
        } else {
            self.account_ids = Some(vec![account_id]);
        }
        self
    }
}

impl LakeContextExt for ParentTransactionCache {
    /// The process to scan the [near_lake_primitives::Block](near_lake_framework::near_lake_primitives::block::Block) and update the cache
    /// with the new transactions and first expected receipts.
    /// The cache is used to find the parent transaction hash for a given receipt id.
    fn execute_before_run(&self, block: &Block) {
        // Fill up the cache with new transactions and first expected receipts
        // We will try to skip the transactions related to the accounts we're not watching for.
        // Based on `accounts_id`
        for tx in block.transactions().filter(move |tx| {
            self.account_ids.is_empty()
                || self.account_ids.contains(tx.signer_id())
                || self.account_ids.contains(tx.receiver_id())
        }) {
            let tx_hash = tx.transaction_hash();
            tx.actions_included()
                .map(|action| action.metadata().receipt_id())
                .for_each(|receipt_id| {
                    let mut cache = self.cache.write().unwrap();
                    cache.cache_set(receipt_id, tx_hash);
                });
        }
        for receipt in block.receipts() {
            let receipt_id = receipt.receipt_id();
            let mut cache = self.cache.write().unwrap();
            let parent_tx_hash = cache.cache_remove(&receipt_id);

            if let Some(parent_tx_hash) = parent_tx_hash {
                cache.cache_set(receipt_id, parent_tx_hash);
            }
        }
    }

    /// We don't need to do anything after the run.
    fn execute_after_run(&self) {}
}

impl ParentTransactionCache {
    /// Returns the parent transaction hash for a given receipt id.
    /// If the receipt id is not found in the cache, it returns None.
    /// If the receipt id is found in the cache, it returns the parent transaction hash.
    pub fn get_parent_transaction_hash(&self, receipt_id: &ReceiptId) -> Option<TransactionHash> {
        // **Note**: [cached::SizedCache] updates metadata on every cache access. That's why
        // we need to use a write lock here.
        let mut cache = self.cache.write().unwrap();
        cache.cache_get(receipt_id).cloned()
    }
}
