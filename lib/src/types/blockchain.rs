use super::{Block, Transaction, TransactionOutput};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::util::Saveable;
use crate::U256;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    utxos: HashMap<Hash, (bool, TransactionOutput)>,
    target: U256,
    blocks: Vec<Block>,
    #[serde(default, skip_serializing)]
    mempool: Vec<(DateTime<Utc>, Transaction)>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            utxos: HashMap::new(),
            target: crate::MIN_TARGET,
            blocks: vec![],
            mempool: vec![],
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
    }

    pub fn target(&self) -> U256 {
        self.target
    }

    pub fn utxos(&self) -> &HashMap<Hash, (bool, TransactionOutput)> {
        &self.utxos
    }

    pub fn calculate_block_reward(&self) -> u64 {
        let block_height = self.block_height();
        let halvings = block_height / crate::HALVING_INTERVAL;
        (crate::INITIAL_REWARD * 10u64.pow(8)) >> halvings
    }

    pub fn mempool(&self) -> &[(DateTime<Utc>, Transaction)] {
        &self.mempool
    }
    // Cleanup mempool - remove transactions older than
    // MAX_MEMPOOL_TRANSACTION_AGE
    pub fn cleanup_mempool(&mut self) {
        let now = Utc::now();
        let mut utxo_hashes_to_unmark: Vec<Hash> = vec![];
        self.mempool.retain(|(timestamp, transaction)| {
            if now - *timestamp
                > chrono::Duration::seconds(crate::MAX_MEMPOOL_TRANSACTION_AGE as i64)
            {
                // push all utxos to unmark to the vector
                // so we can unmark them later
                utxo_hashes_to_unmark.extend(
                    transaction
                        .inputs
                        .iter()
                        .map(|input| input.prev_tx_output_hash),
                );
                false
            } else {
                true
            }
        });
        // unmark all of the UTXOs
        for hash in utxo_hashes_to_unmark {
            self.utxos.entry(hash).and_modify(|(marked, _)| {
                *marked = false;
            });
        }
    }

    pub fn add_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        transaction.validate(&self.utxos)?;

        //if any of the utxos have the bool mark set to true
        // and if so, find the transaction that references them
        // in mempool, remove it, and set all the utxos it references
        // to false
        for input in &transaction.inputs {
            if let Some((true, _)) = self.utxos.get(&input.prev_tx_output_hash) {
                // find the transaction that references the UTXO
                // we are trying to reference
                let referencing_transaction =
                    self.mempool
                        .iter()
                        .enumerate()
                        .find(|(_, (_, transaction))| {
                            transaction
                                .outputs
                                .iter()
                                .any(|output| output.hash() == input.prev_tx_output_hash)
                        });
                // If we have found one, unmark all of its UTXOs
                if let Some((idx, (_, referencing_transaction))) = referencing_transaction {
                    for input in &referencing_transaction.inputs {
                        // set all utxos from this transaction to false
                        self.utxos
                            .entry(input.prev_tx_output_hash)
                            .and_modify(|(marked, _)| {
                                *marked = false;
                            });
                    }
                    // remove the transaction from the mempool
                    self.mempool.remove(idx);
                } else {
                    // if, somehow, there is no matching transaction,
                    // set this utxo to false
                    self.utxos
                        .entry(input.prev_tx_output_hash)
                        .and_modify(|(marked, _)| {
                            *marked = false;
                        });
                }
            }
        }

        // Mark the UTXOs as used
        for input in &transaction.inputs {
            self.utxos
                .entry(input.prev_tx_output_hash)
                .and_modify(|(marked, _)| {
                    *marked = true;
                });
        }
        self.mempool.push((Utc::now(), transaction));

        // sort by miner fee
        self.mempool
            .sort_by_key(|(_, tx)| tx.calculate_fee(&self.utxos).unwrap());
        Ok(())
    }

    pub fn rebuild_utxos(&mut self) {
        for block in &self.blocks {
            for transaction in &block.transactions {
                for input in &transaction.inputs {
                    self.utxos.remove(&input.prev_tx_output_hash);
                }

                for output in transaction.outputs.iter() {
                    self.utxos
                        .insert(transaction.hash(), (false, output.clone()));
                }
            }
        }
    }

    pub fn add_block(&mut self, block: Block) -> Result<()> {
        if self.blocks.is_empty() {
            if block.header.prev_block_hash != Hash::zero() {
                println!("zero hash");
                return Err(BtcError::InvalidBlock);
            }
        } else {
            let last_block = self.blocks.last().unwrap();

            if block.header.prev_block_hash != last_block.hash() {
                println!("prev hash is wrong");
                return Err(BtcError::InvalidBlock);
            }

            if !block.header.hash().matches_target(block.header.target) {
                println!("does not match target");
                return Err(BtcError::InvalidBlock);
            }

            let calculated_merkle_root = MerkleRoot::calculate(&block.transactions);
            if calculated_merkle_root != block.header.merkle_root {
                println!("invalid merkle root");
                return Err(BtcError::InvalidMerkleRoot);
            }

            if block.header.timestamp <= last_block.header.timestamp {
                return Err(BtcError::InvalidBlock);
            }

            block.verify_transactions(self.block_height(), &self.utxos)?;
        }
        let block_transactions: HashSet<_> =
            block.transactions.iter().map(|tx| tx.hash()).collect();

        self.mempool
            .retain(|(_, tx)| !block_transactions.contains(&tx.hash()));

        self.blocks.push(block);
        self.try_adjust_target();
        Ok(())
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.is_empty() {
            return;
        }
        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INTERVAL as usize != 0 {
            return;
        }
        // measure time it took to mine the last
        // crate::DIFFICULTY_UPDATE_INTERVAL blocks with chrono
        let start_time = self.blocks
            [self.blocks.len() - crate::DIFFICULTY_UPDATE_INTERVAL as usize]
            .header
            .timestamp;
        let end_time = self.blocks.last().unwrap().header.timestamp;

        let diff_time = end_time - start_time;

        let diff_time_seconds = diff_time.num_seconds();

        let target_seconds = crate::DIFFICULTY_UPDATE_INTERVAL * crate::IDEAL_BLOCK_TIME;
        let new_target = BigDecimal::parse_bytes(&self.target.to_string().as_bytes(), 10)
            .expect("BUG: Impossible")
            * (BigDecimal::from(diff_time_seconds) / BigDecimal::from(target_seconds));

        // cut off decimals from the new target
        let new_target_str = new_target
            .to_string()
            .split('.')
            .next()
            .expect("BUG: Expected a decimal point")
            .to_owned();

        // convert from string to U256
        let new_target: U256 = U256::from_str_radix(&new_target_str, 10).expect("BUG: Impossible");

        // clamp new_target to be in the range of
        // self.target * 4 and self.target / 4
        let new_target = if new_target < self.target / 4 {
            self.target / 4
        } else if new_target > self.target * 4 {
            self.target * 4
        } else {
            new_target
        };
        self.target = new_target.min(crate::MIN_TARGET);
    }

    pub fn block_height(&self) -> u64 {
        self.utxos.len() as u64
    }
}

impl Saveable for Blockchain {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize Blockchain"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Blockchain"))
    }
}
