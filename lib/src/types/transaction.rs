use crate::crypto::{PublicKey, Signature};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::Saveable;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    pub prev_tx_output_hash: Hash,
    pub signature: Signature,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    pub value: u64,
    pub unique_id: Uuid,
    pub pub_key: PublicKey,
}

impl Transaction {
    pub fn new(inputs: Vec<TransactionInput>, outputs: Vec<TransactionOutput>) -> Self {
        Transaction { inputs, outputs }
    }

    pub fn validate(&self, utxos: &HashMap<Hash, (bool, TransactionOutput)>) -> Result<()> {
        let mut inputs = HashSet::new();
        let mut input_value: u64 = 0;
        let output_value: u64;

        for input in &self.inputs {
            let prev_output = utxos.get(&input.prev_tx_output_hash);
            if prev_output.is_none() {
                return Err(BtcError::InvalidTransaction);
            }

            let prev_output = prev_output.unwrap();

            if inputs.contains(&input.prev_tx_output_hash) {
                return Err(BtcError::InvalidTransaction);
            }

            if !input
                .signature
                .verify(&input.prev_tx_output_hash, &prev_output.1.pub_key)
            {
                return Err(BtcError::InvalidTransaction);
            }
            input_value += prev_output.1.value;
            inputs.insert(input.prev_tx_output_hash);
        }
        output_value = self.outputs.iter().map(|output| output.value).sum();

        if input_value >= output_value {
            return Err(BtcError::InvalidTransaction);
        }
        Ok(())
    }

    pub fn calculate_fee(&self, utxos: &HashMap<Hash, (bool, TransactionOutput)>) -> Result<u64> {
        let input_value: u64 = self
            .inputs
            .iter()
            .map(|input| {
                utxos
                    .get(&input.prev_tx_output_hash)
                    .map(|(_, output)| output.value)
                    .ok_or(BtcError::InvalidTransaction)
            })
            .sum::<Result<u64>>()?;

        let output_value: u64 = self.outputs.iter().map(|output| output.value).sum();

        input_value
            .checked_sub(output_value)
            .ok_or(BtcError::InvalidTransaction)
    }

    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl TransactionOutput {
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

impl Saveable for Transaction {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader).map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidData,
                "Failed to deserialize Transaction",
            )
        })
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Transaction"))
    }
}
