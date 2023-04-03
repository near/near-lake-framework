use near_crypto::{PublicKey, Signature};

use super::receipts::ExecutionStatus;
use crate::near_indexer_primitives::{types::AccountId, CryptoHash, IndexerTransactionWithOutcome};

#[derive(Debug, Clone)]
pub struct Transaction {
    transaction_hash: CryptoHash,
    signer_id: AccountId,
    signer_public_key: PublicKey,
    signature: Signature,
    receiver_id: AccountId,
    status: ExecutionStatus,
    execution_outcome_id: CryptoHash,
    actions: Vec<super::receipts::Action>,
}

impl Transaction {
    pub fn transaction_hash(&self) -> CryptoHash {
        self.transaction_hash
    }

    pub fn signer_id(&self) -> &AccountId {
        &self.signer_id
    }

    pub fn signer_public_key(&self) -> &PublicKey {
        &self.signer_public_key
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn receiver_id(&self) -> &AccountId {
        &self.receiver_id
    }

    pub fn status(&self) -> &ExecutionStatus {
        &self.status
    }

    pub fn execution_outcome_id(&self) -> CryptoHash {
        self.execution_outcome_id
    }

    pub fn actions_included(&self) -> impl Iterator<Item = &super::receipts::Action> {
        self.actions.iter()
    }
}

impl TryFrom<&IndexerTransactionWithOutcome> for Transaction {
    type Error = &'static str;

    fn try_from(tx_with_outcome: &IndexerTransactionWithOutcome) -> Result<Self, Self::Error> {
        Ok(Self {
            transaction_hash: tx_with_outcome.transaction.hash,
            signer_id: tx_with_outcome.transaction.signer_id.clone(),
            signer_public_key: tx_with_outcome.transaction.public_key.clone(),
            signature: tx_with_outcome.transaction.signature.clone(),
            receiver_id: tx_with_outcome.transaction.receiver_id.clone(),
            execution_outcome_id: tx_with_outcome.outcome.execution_outcome.id,
            status: (&tx_with_outcome.outcome.execution_outcome.outcome.status).into(),
            actions: super::receipts::Action::try_vec_from_transaction_outcome(tx_with_outcome)?,
        })
    }
}
