use std::collections::HashMap;

use super::actions::{self, ActionMetaDataExt};
use super::events::{self, EventsTrait};
use super::receipts::{self};
use super::state_changes;
use super::transactions;
use crate::near_indexer_primitives::{types::AccountId, views, CryptoHash, StreamerMessage};

/// A structure that represents an entire block in the NEAR blockchain.
/// It is a high-level structure that is built on top of the low-level [StreamerMessage] structure.
///
/// The access to all the data is provided through the getters. Thus we can refactor the structure yet keep the API stable and backward compatible.
///
/// With a high-level update we are trying to replace the usage of the low-level [StreamerMessage] with this one.
///
/// #### Important notes on the Block
/// - All the entities located on different shards were merged into one single list without differentiation.
///   The statement from **NEAR** is that **sharding is going to be dynamic and seamless for the users**, that’s why we’ve decided indexer
///   developers don’t want to care about shards either.
/// - Original [near_indexer_primitives::StreamerMessage] represents the blockchain data in *a most fair manner**. Although, it used to be
///   a pain in the neck for indexer developers, we’ve decided to act as a painkiller here.
/// - [Block] is not the fairest name for this structure either. **NEAR Protocol** is a sharded blockchain, so its block is actually an
///   ephemeral structure that represents a collection of *real blocks* called Chunks in **NEAR Protocol**. We’ve been simplifying things here though,
///   so here is a result of the simplification.
#[derive(Debug)]
pub struct Block {
    streamer_message: StreamerMessage,
    executed_receipts: Vec<receipts::Receipt>,
    postponed_receipts: Vec<receipts::Receipt>,
    transactions: Vec<transactions::Transaction>,
    actions: Vec<actions::Action>,
    state_changes: Vec<state_changes::StateChange>,
}

impl Block {
    /// Return a reference to the original StreamerMessage of the block. This is the low-level structure.
    ///
    /// While introducing the high-level structures, methods, and helpers, we do want to keep the low-level “door” open
    /// for advanced developers or edge cases which we haven’t accidentally covered, or just don’t have the capacity to cover.
    ///
    /// That’s why every instance of the Block will hold the original StreamerMessage for developers.
    /// Think of it as backward compatibility if you prefer.
    pub fn streamer_message(&self) -> &StreamerMessage {
        &self.streamer_message
    }

    /// Returns the block hash. It is a shortcut to get the data from the block header.
    pub fn block_hash(&self) -> CryptoHash {
        self.header().hash()
    }

    /// Returns the previous block hash. It is a shortcut to get the data from the block header.
    pub fn prev_block_hash(&self) -> CryptoHash {
        self.header().prev_hash()
    }

    /// Returns the block height. It is a shortcut to get the data from the block header.
    pub fn block_height(&self) -> u64 {
        self.header().height()
    }

    /// Returns a [BlockHeader] structure of the block
    ///
    ///See [BlockHeader] structure sections for details.
    pub fn header(&self) -> BlockHeader {
        (&self.streamer_message).into()
    }

    /// Returns an iterator over the [Receipt](crate::receipts::Receipt)s executed in this [Block].
    ///
    /// This field is a representation of `StreamerMessage.shard[N].receipt_execution_outcomes`
    ///
    /// A reminder that `receipt_execution_outcomes` has a type [near_indexer_primitives::IndexerExecutionOutcomeWithReceipt] which is an
    /// ephemeral structure from `near-indexer-primitives` that hold a [near_primitives::views::ExecutionOutcomeView]
    /// along with the corresponding [near_primitives::views::ReceiptView].
    pub fn receipts(&self) -> impl Iterator<Item = &receipts::Receipt> {
        self.executed_receipts.iter()
    }

    /// Returns an iterator of [Receipt](crate::receipts::Receipt) included yet not executed in the [Block].
    ///
    /// [Receipts](crate::receipts::Receipt) included on the chain but not executed yet are called "postponed",
    /// they are represented by the same structure [Receipt](crate::receipts::Receipt).
    pub fn postponed_receipts(&self) -> impl Iterator<Item = &receipts::Receipt> {
        self.postponed_receipts.iter()
    }

    /// Returns an iterator of the [Transactions](crate::transactions::Transaction) included in the [Block].
    ///
    /// **Heads up!** Some indexer developers care about [Transaction](crate::transactions::Transaction)s for the knowledge where
    /// the action chain has begun. Other indexer developers care about it because of the habits
    /// from other blockchains like Ethereum where a transaction is a main asset. In case of NEAR
    /// [Receipts](crate::receipts::Receipt) are more important.
    pub fn transactions(&self) -> impl Iterator<Item = &transactions::Transaction> {
        self.transactions.iter()
    }

    /// Returns an iterator of the [Actions](crate::actions::Action) executed in the [Block]
    pub fn actions(&self) -> impl Iterator<Item = &actions::Action> {
        self.actions.iter()
    }

    /// Returns a Vec of [Events](crate::events::Event) emitted in the [Block]
    pub fn events(&self) -> HashMap<super::ReceiptId, Vec<events::Event>> {
        self.executed_receipts
            .iter()
            .map(|receipt| (receipt.receipt_id(), receipt.events()))
            .collect()
    }

    /// Returns an iterator of the [StateChanges](crate::state_changes::StateChange) happened in the [Block]
    pub fn state_changes(&self) -> impl Iterator<Item = &state_changes::StateChange> {
        self.state_changes.iter()
    }

    /// Helper to get all the [Actions](crate::actions::Action) by the single [Receipt](crate::receipts::Receipt)
    ///
    /// **Heads up!** This methods searches for the actions in the current [Block] only.
    pub fn actions_by_receipt_id<'a>(
        &'a self,
        receipt_id: &'a super::ReceiptId,
    ) -> impl Iterator<Item = &actions::Action> + 'a {
        self.actions()
            .filter(move |action| &action.receipt_id() == receipt_id)
    }

    /// Helper to get all the [Events](crate::events::Event) emitted by the specific [Receipt](crate::receipts::Receipt)
    pub fn events_by_receipt_id(&self, receipt_id: &super::ReceiptId) -> Vec<events::Event> {
        if let Some(events) = self.events().get(receipt_id) {
            events.to_vec()
        } else {
            vec![]
        }
    }

    /// Helper to get all the [Events](crate::events::Event) emitted by the specific contract ([AccountId](crate::near_indexer_primitives::types::AccountId))
    pub fn events_by_contract_id(
        &self,
        account_id: &crate::near_indexer_primitives::types::AccountId,
    ) -> Vec<events::Event> {
        let account_id_clone = account_id.clone(); // Clone the account_id
        self.events()
            .values()
            .flatten()
            .filter(|event| event.is_emitted_by_contract(&account_id_clone))
            .map(Clone::clone)
            .collect()
    }

    /// Helper to get a specific [Receipt](crate::receipts::Receipt) by the [ReceiptId](crate::types::ReceiptId)
    pub fn receipt_by_id(&self, receipt_id: &super::ReceiptId) -> Option<&receipts::Receipt> {
        self.receipts()
            .find(|receipt| &receipt.receipt_id() == receipt_id)
    }
}

impl From<StreamerMessage> for Block {
    fn from(streamer_message: StreamerMessage) -> Self {
        let executed_receipts: Vec<receipts::Receipt> = streamer_message
            .shards
            .iter()
            .flat_map(|shard| shard.receipt_execution_outcomes.iter())
            .map(Into::into)
            .collect();
        let postponed_receipts = streamer_message
            .shards
            .iter()
            .filter_map(|shard| shard.chunk.as_ref().map(|chunk| chunk.receipts.iter()))
            .flatten()
            // exclude receipts that are already executed
            .filter(|receipt| {
                !executed_receipts
                    .iter()
                    .any(|executed_receipt| executed_receipt.receipt_id() == receipt.receipt_id)
            })
            .map(Into::into)
            .collect();

        let transactions: Vec<transactions::Transaction> = streamer_message
            .shards
            .iter()
            .filter_map(|shard| shard.chunk.as_ref().map(|chunk| chunk.transactions.iter()))
            .flatten()
            .map(TryInto::try_into)
            .filter_map(|transactions| transactions.ok())
            .collect();

        let actions: Vec<actions::Action> = streamer_message
            .shards
            .iter()
            .flat_map(|shard| shard.receipt_execution_outcomes.iter())
            .filter_map(|receipt_execution_outcome| {
                actions::Action::try_vec_from_receipt_view(&receipt_execution_outcome.receipt).ok()
            })
            .flatten()
            .collect();

        let state_changes: Vec<state_changes::StateChange> = streamer_message
            .shards
            .iter()
            .flat_map(|shard| shard.state_changes.iter())
            .map(Into::into)
            .collect();

        Self {
            executed_receipts,
            postponed_receipts,
            transactions,
            actions,
            state_changes,
            streamer_message,
        }
    }
}

/// Replacement for [`BlockHeaderView`](near_primitives::views::BlockHeaderView) from `near-primitives`. Shrank and simplified.
/// We were trying to leave only the fields indexer developers might be interested in.
///
/// Friendly reminder, the original [`BlockHeaderView`](near_primitives::views::BlockHeaderView) is still accessible
/// via [`.streamer_message()`](Block::streamer_message()) method.
#[derive(Debug, Clone)]
pub struct BlockHeader {
    height: u64,
    hash: CryptoHash,
    prev_hash: CryptoHash,
    author: AccountId,
    timestamp_nanosec: u64,
    epoch_id: CryptoHash,
    next_epoch_id: CryptoHash,
    gas_price: u128,
    total_supply: u128,
    latest_protocol_version: u32,
    random_value: CryptoHash,
    chunks_included: u64,
    // TODO: replace with the corresponding Lake Primitives type eventually
    validator_proposals: Vec<views::validator_stake_view::ValidatorStakeView>,
}

impl BlockHeader {
    /// The height of the [Block]
    pub fn height(&self) -> u64 {
        self.height
    }

    /// The hash of the [Block]
    pub fn hash(&self) -> CryptoHash {
        self.hash
    }

    /// The hash of the previous [Block]
    pub fn prev_hash(&self) -> CryptoHash {
        self.prev_hash
    }

    /// The [AccountId](crate::near_indexer_primitives::types::AccountId) of the author of the [Block]
    pub fn author(&self) -> AccountId {
        self.author.clone()
    }

    /// The timestamp of the [Block] in nanoseconds
    pub fn timestamp_nanosec(&self) -> u64 {
        self.timestamp_nanosec
    }

    /// The [CryptoHash] of the epoch the [Block] belongs to
    pub fn epoch_id(&self) -> CryptoHash {
        self.epoch_id
    }

    /// The [CryptoHash] of the next epoch
    pub fn next_epoch_id(&self) -> CryptoHash {
        self.next_epoch_id
    }

    /// The gas price of the [Block]
    pub fn gas_price(&self) -> u128 {
        self.gas_price
    }

    /// The total supply of the [Block]
    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    /// The latest protocol version of the [Block]
    pub fn latest_protocol_version(&self) -> u32 {
        self.latest_protocol_version
    }

    /// The random value of the [Block]
    pub fn random_value(&self) -> CryptoHash {
        self.random_value
    }

    /// The number of chunks included in the [Block]
    pub fn chunks_included(&self) -> u64 {
        self.chunks_included
    }

    /// The validator proposals of the [Block]
    ///
    /// **Heads up!** This methods returns types defined in the `near-primitives` crate as is.
    /// It is a subject of change in the future (once we define the corresponding Lake Primitives types)
    pub fn validator_proposals(&self) -> Vec<views::validator_stake_view::ValidatorStakeView> {
        self.validator_proposals.clone()
    }
}

impl From<&StreamerMessage> for BlockHeader {
    fn from(streamer_message: &StreamerMessage) -> Self {
        Self {
            height: streamer_message.block.header.height,
            hash: streamer_message.block.header.hash,
            prev_hash: streamer_message.block.header.prev_hash,
            author: streamer_message.block.author.clone(),
            timestamp_nanosec: streamer_message.block.header.timestamp_nanosec,
            epoch_id: streamer_message.block.header.epoch_id,
            next_epoch_id: streamer_message.block.header.next_epoch_id,
            gas_price: streamer_message.block.header.gas_price,
            total_supply: streamer_message.block.header.total_supply,
            latest_protocol_version: streamer_message.block.header.latest_protocol_version,
            random_value: streamer_message.block.header.random_value,
            chunks_included: streamer_message.block.header.chunks_included,
            validator_proposals: streamer_message.block.header.validator_proposals.clone(),
        }
    }
}
