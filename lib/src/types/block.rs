use super::{Transaction, TransactionOutput};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::util::Saveable;
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    pub timestamp: DateTime<Utc>,
    pub nonce: u64,
    pub prev_block_hash: Hash,
    pub merkle_root: MerkleRoot,
    pub target: U256,
}

impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            header,
            transactions,
        }
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }

    pub fn calcualte_block_fees(
        &self,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<u64> {
        let mut fees: u64 = 0;
        for tx in &self.transactions {
            fees += tx.calculate_fee(utxos)?;
        }
        Ok(fees)
    }

    pub fn verify_coinbase_transaction(
        &self,
        predicted_block_height: u64,
        miner_fees: u64,
    ) -> Result<()> {
        let coinbase_transaction = &self.transactions[0];
        if coinbase_transaction.inputs.len() != 0 {
            return Err(BtcError::InvalidTransaction);
        }

        if coinbase_transaction.outputs.len() == 0 {
            return Err(BtcError::InvalidTransaction);
        }
        let block_reward = crate::INITIAL_REWARD * 10u64.pow(8)
            / 2u64.pow((predicted_block_height / crate::HALVING_INTERVAL) as u32);

        let total_coinbase_outputs: u64 = coinbase_transaction
            .outputs
            .iter()
            .map(|output| output.value)
            .sum();
        if total_coinbase_outputs != block_reward + miner_fees {
            return Err(BtcError::InvalidTransaction);
        }

        Ok(())
    }

    pub fn verify_transactions(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, (bool, TransactionOutput)>,
    ) -> Result<()> {
        let mut spent_utxos = HashSet::new();
        let mut fees: u64 = 0;
        // reject completely empty blocks
        if self.transactions.is_empty() {
            return Err(BtcError::InvalidTransaction);
        }

        for transaction in self.transactions.iter().skip(1) {
            transaction.validate(utxos)?;

            for input in &transaction.inputs {
                if !spent_utxos.insert(input.prev_tx_output_hash.clone()) {
                    return Err(BtcError::InvalidTransaction);
                }
            }

            fees += transaction.calculate_fee(utxos)?;
        }
        self.verify_coinbase_transaction(predicted_block_height, fees)?;
        Ok(())
    }
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }

    pub fn mine(&mut self, steps: usize) -> bool {
        // if the block already matches the target return.
        if self.hash().matches_target(self.target) {
            return true;
        }
        for _ in 0..steps {
            if let Some(new_nonce) = self.nonce.checked_add(1) {
                self.nonce = new_nonce;
            } else {
                self.nonce = 0;
                self.timestamp = Utc::now();
            }
            if self.hash().matches_target(self.target) {
                return true;
            }
        }
        false
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl Saveable for Block {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize Block"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Block"))
    }
}
