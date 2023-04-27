use std::collections::HashMap;

use super::actions::{self, ActionMetaDataExt};
use super::events::{self, EventsTrait};
use super::receipts::{self};
use super::state_changes;
use super::transactions;
use crate::near_indexer_primitives::{types::AccountId, views, CryptoHash, StreamerMessage};

#[derive(Debug)]
pub struct Block {
    streamer_message: StreamerMessage,
    executed_receipts: Vec<receipts::Receipt>,
    postponed_receipts: Vec<receipts::Receipt>,
    transactions: Vec<transactions::Transaction>,
    actions: Vec<actions::Action>,
    events: HashMap<super::ReceiptId, Vec<events::Event>>,
    state_changes: Vec<state_changes::StateChange>,
}

impl Block {
    pub fn streamer_message(&self) -> &StreamerMessage {
        &self.streamer_message
    }

    pub fn block_hash(&self) -> CryptoHash {
        self.header().hash()
    }

    pub fn prev_block_hash(&self) -> CryptoHash {
        self.header().prev_hash()
    }

    pub fn block_height(&self) -> u64 {
        self.header().height()
    }

    pub fn header(&self) -> BlockHeader {
        (&self.streamer_message).into()
    }

    pub fn receipts(&mut self) -> impl Iterator<Item = &receipts::Receipt> {
        if self.executed_receipts.is_empty() {
            self.executed_receipts = self
                .streamer_message
                .shards
                .iter()
                .flat_map(|shard| shard.receipt_execution_outcomes.iter())
                .map(Into::into)
                .collect();
        }
        self.executed_receipts.iter()
    }

    pub fn postponed_receipts(&mut self) -> impl Iterator<Item = &receipts::Receipt> {
        if self.postponed_receipts.is_empty() {
            let executed_receipts_ids: Vec<_> = self
                .receipts()
                .map(|receipt| receipt.receipt_id())
                .collect();
            self.postponed_receipts = self
                .streamer_message
                .shards
                .iter()
                .filter_map(|shard| shard.chunk.as_ref().map(|chunk| chunk.receipts.iter()))
                .flatten()
                // exclude receipts that are already executed
                .filter(|receipt| !executed_receipts_ids.contains(&receipt.receipt_id))
                .map(Into::into)
                .collect();
        }
        self.postponed_receipts.iter()
    }

    pub fn transactions(&mut self) -> impl Iterator<Item = &transactions::Transaction> {
        if self.transactions.is_empty() {
            self.transactions = self
                .streamer_message
                .shards
                .iter()
                .filter_map(|shard| shard.chunk.as_ref().map(|chunk| chunk.transactions.iter()))
                .flatten()
                .map(TryInto::try_into)
                .filter_map(|transactions| transactions.ok())
                .collect();
        }
        self.transactions.iter()
    }

    fn actions_from_streamer_message(&self) -> Vec<actions::Action> {
        self.streamer_message()
            .shards
            .iter()
            .flat_map(|shard| shard.receipt_execution_outcomes.iter())
            .filter_map(|receipt_execution_outcome| {
                actions::Action::try_vec_from_receipt_view(&receipt_execution_outcome.receipt).ok()
            })
            .flatten()
            .collect()
    }

    pub fn actions(&mut self) -> impl Iterator<Item = &actions::Action> {
        if self.actions.is_empty() {
            self.build_actions_cache();
        }
        self.actions.iter()
    }

    pub fn events(&mut self) -> impl Iterator<Item = &events::Event> {
        if self.events.is_empty() {
            self.build_events_hashmap();
        }
        self.events.values().flatten()
    }

    pub fn state_changes(&mut self) -> impl Iterator<Item = &state_changes::StateChange> {
        if self.state_changes.is_empty() {
            self.state_changes = self
                .streamer_message
                .shards
                .iter()
                .flat_map(|shard| shard.state_changes.iter())
                .map(Into::into)
                .collect();
        }
        self.state_changes.iter()
    }

    pub fn actions_by_receipt_id<'a>(
        &'a mut self,
        receipt_id: &'a super::ReceiptId,
    ) -> impl Iterator<Item = &actions::Action> + 'a {
        self.actions()
            .filter(move |action| &action.receipt_id() == receipt_id)
    }

    pub fn events_by_receipt_id(&mut self, receipt_id: &super::ReceiptId) -> Vec<events::Event> {
        if self.events.is_empty() {
            self.build_events_hashmap();
        }
        if let Some(events) = self.events.get(receipt_id) {
            events.to_vec()
        } else {
            vec![]
        }
    }

    pub fn events_by_contract_id<'a>(
        &'a mut self,
        account_id: &'a crate::near_indexer_primitives::types::AccountId,
    ) -> impl Iterator<Item = &events::Event> + 'a {
        self.events()
            .filter(move |event| event.is_emitted_by_contract(&account_id.clone()))
    }

    pub fn receipt_by_id(&mut self, receipt_id: &super::ReceiptId) -> Option<&receipts::Receipt> {
        self.receipts()
            .find(|receipt| &receipt.receipt_id() == receipt_id)
    }
}

impl Block {
    fn build_actions_cache(&mut self) {
        self.actions = self.actions_from_streamer_message().to_vec();
    }

    fn build_events_hashmap(&mut self) {
        self.events = self
            .receipts()
            .map(|receipt| (receipt.receipt_id(), receipt.events()))
            .collect();
    }
}

impl From<StreamerMessage> for Block {
    fn from(streamer_message: StreamerMessage) -> Self {
        Self {
            streamer_message,
            executed_receipts: vec![],
            postponed_receipts: vec![],
            transactions: vec![],
            actions: vec![],
            events: HashMap::new(),
            state_changes: vec![],
        }
    }
}

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
    validator_proposals: Vec<views::validator_stake_view::ValidatorStakeView>,
}

impl BlockHeader {
    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn hash(&self) -> CryptoHash {
        self.hash
    }

    pub fn prev_hash(&self) -> CryptoHash {
        self.prev_hash
    }

    pub fn author(&self) -> AccountId {
        self.author.clone()
    }

    pub fn timestamp_nanosec(&self) -> u64 {
        self.timestamp_nanosec
    }

    pub fn epoch_id(&self) -> CryptoHash {
        self.epoch_id
    }

    pub fn next_epoch_id(&self) -> CryptoHash {
        self.next_epoch_id
    }

    pub fn gas_price(&self) -> u128 {
        self.gas_price
    }

    pub fn total_supply(&self) -> u128 {
        self.total_supply
    }

    pub fn latest_protocol_version(&self) -> u32 {
        self.latest_protocol_version
    }

    pub fn random_value(&self) -> CryptoHash {
        self.random_value
    }

    pub fn chunks_included(&self) -> u64 {
        self.chunks_included
    }

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