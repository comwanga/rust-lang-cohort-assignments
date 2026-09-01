#![allow(unused_variables)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const COINBASE_PREVIOUS_OUTPUT: &str = "-";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
    Signet,
    Regtest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxStatus {
    Spent,
    Unspent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxInput {
    pub previous_txid: String,
    pub previous_vout: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    pub value_sats: u64,
    pub unique_id: Uuid,
    pub recipient: String,
    pub status: TxStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub txid: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub block_hash: String,
    pub previous_block_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub height: u64,
    pub network: Network,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmountSummary {
    pub output_count: usize,
    pub total_sats: u64,
    pub spent_sats: u64,
    pub unspent_sats: u64,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum BtcLibError {
    #[error("malformed transaction data")]
    MalformedData,
    #[error("missing transaction")]
    MissingTransaction,
    #[error("empty transaction id")]
    EmptyTxId,
    #[error("missing transaction inputs")]
    MissingInputs,
    #[error("missing transaction outputs")]
    MissingOutputs,
    #[error("zero value output")]
    ZeroValueOutput,
    #[error("empty block")]
    EmptyBlock,
    #[error("duplicate transaction id")]
    DuplicateTxId,
    #[error("invalid hash hex")]
    InvalidHash,
    #[error("io error: {0}")]
    Io(String),
}

pub trait Hashable {
    /// Return stable string material that should be fed into `hash_hex`.
    fn hash_material(&self) -> String;

    /// Compute a SHA-256 hex digest for this value.
    ///
    /// This default implementation is complete; students do not need to edit it.
    fn hash_hex(&self) -> String {
        sha256::digest(self.hash_material())
    }
}

pub trait Validate {
    /// Validate a value and return a specific `BtcLibError` for bad data.
    fn validate(&self) -> Result<(), BtcLibError>;
}

impl From<std::io::Error> for BtcLibError {
    /// Convert an IO error into `BtcLibError::Io` while preserving its message.
    fn from(error: std::io::Error) -> Self {
        // Steps:
        // 1. Convert the IO error to a string.
        // 2. Store that string inside `BtcLibError::Io`.
        Self::Io(error.to_string())
    }
}

impl TxInput {
    /// Build a transaction input by copying the previous txid and storing vout.
    pub fn new(previous_txid: &str, previous_vout: u32) -> Self {
        // Steps:
        // 1. Convert `previous_txid` into an owned `String`.
        // 2. Store `previous_vout` unchanged.
        // 3. Return a `TxInput`.
        Self {
            previous_txid: previous_txid.to_string(),
            previous_vout,
        }
    }
}

impl TxOutput {
    /// Build a transaction output with a fresh UUID.
    pub fn new(value_sats: u64, recipient: &str, status: TxStatus) -> Self {
        // Steps:
        // 1. Store `value_sats` unchanged.
        // 2. Generate `unique_id` using `Uuid::new_v4()`.
        // 3. Convert `recipient` into an owned `String`.
        // 4. Store `status` unchanged.
        // 5. Return a `TxOutput`.
        Self {
            value_sats,
            unique_id: Uuid::new_v4(),
            recipient: recipient.to_string(),
            status,
        }
    }

    /// Return true when this output status is `TxStatus::Unspent`.
    pub fn is_unspent(&self) -> bool {
        // Steps:
        // 1. Compare `self.status` with `TxStatus::Unspent`.
        // 2. Return the boolean result.
        self.status == TxStatus::Unspent
    }
}

impl Validate for TxOutput {
    /// Reject zero-value outputs.
    fn validate(&self) -> Result<(), BtcLibError> {
        // Steps:
        // 1. If `value_sats` is 0, return `Err(BtcLibError::ZeroValueOutput)`.
        // 2. Otherwise return `Ok(())`.
        if self.value_sats == 0 {
            Err(BtcLibError::ZeroValueOutput)
        } else {
            Ok(())
        }
    }
}

impl Transaction {
    /// Build a transaction by copying the txid and storing inputs/outputs.
    pub fn new(txid: &str, inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        // Steps:
        // 1. Convert `txid` into an owned `String`.
        // 2. Move `inputs` and `outputs` into the transaction.
        // 3. Return a `Transaction`.
        Self {
            txid: txid.to_string(),
            inputs,
            outputs,
        }
    }

    /// Return true for the simplified coinbase rule used in this assignment.
    ///
    /// A coinbase transaction has txid `"coinbase"` and no inputs.
    pub fn is_coinbase(&self) -> bool {
        // Steps:
        // 1. Check that `self.txid == "coinbase"`.
        // 2. Check that `self.inputs` is empty.
        // 3. Return true only when both checks pass.
        self.txid == "coinbase" && self.inputs.is_empty()
    }

    /// Sum the satoshi value of every output in this transaction.
    pub fn total_output_value(&self) -> u64 {
        // Steps:
        // 1. Start a total at 0.
        // 2. Add every output's `value_sats`.
        // 3. Return the total.
        self.outputs.iter().map(|output| output.value_sats).sum()
    }

    /// Count outputs whose status is `TxStatus::Unspent`.
    pub fn unspent_output_count(&self) -> usize {
        // Steps:
        // 1. Walk through `self.outputs`.
        // 2. Count outputs whose status is `TxStatus::Unspent`.
        // 3. Return the count.
        self.outputs
            .iter()
            .filter(|output| output.is_unspent())
            .count()
    }

    /// Count outputs whose status is `TxStatus::Spent`.
    pub fn spent_output_count(&self) -> usize {
        // Steps:
        // 1. Walk through `self.outputs`.
        // 2. Count outputs whose status is `TxStatus::Spent`.
        // 3. Return the count.
        self.outputs
            .iter()
            .filter(|output| output.status == TxStatus::Spent)
            .count()
    }
}

impl Hashable for Transaction {
    /// Return deterministic transaction hash material.
    ///
    /// Use exactly this format:
    /// `tx:<txid>|inputs:<prev_txid>:<vout>;...|outputs:<value>:<recipient>:<status>;...`
    ///
    /// Status text must be lowercase: `spent` or `unspent`.
    fn hash_material(&self) -> String {
        // Steps:
        // 1. Start with `tx:<txid>|inputs:`.
        // 2. Append each input as `<previous_txid>:<previous_vout>;`.
        // 3. Append `|outputs:`.
        // 4. Append each output as `<value_sats>:<recipient>:<status>;`.
        // 5. Return the final string.
        let inputs = self
            .inputs
            .iter()
            .map(|input| format!("{}:{};", input.previous_txid, input.previous_vout))
            .collect::<String>();
        let outputs = self
            .outputs
            .iter()
            .map(|output| {
                let status = match output.status {
                    TxStatus::Spent => "spent",
                    TxStatus::Unspent => "unspent",
                };
                format!("{}:{}:{};", output.value_sats, output.recipient, status)
            })
            .collect::<String>();

        format!("tx:{}|inputs:{inputs}|outputs:{outputs}", self.txid)
    }
}

impl Validate for Transaction {
    /// Validate a transaction using the Week 3 model plus Week 4 errors.
    fn validate(&self) -> Result<(), BtcLibError> {
        // Steps:
        // 1. If `txid` is empty, return `Err(BtcLibError::EmptyTxId)`.
        // 2. If the transaction is not coinbase and has no inputs, return
        //    `Err(BtcLibError::MissingInputs)`.
        // 3. If there are no outputs, return `Err(BtcLibError::MissingOutputs)`.
        // 4. Validate every output and return the first output error.
        // 5. Otherwise return `Ok(())`.
        if self.txid.is_empty() {
            return Err(BtcLibError::EmptyTxId);
        }
        if !self.is_coinbase() && self.inputs.is_empty() {
            return Err(BtcLibError::MissingInputs);
        }
        if self.outputs.is_empty() {
            return Err(BtcLibError::MissingOutputs);
        }

        validate_all(&self.outputs)
    }
}

impl BlockHeader {
    /// Build a block header by copying string fields and storing numbers.
    pub fn new(
        block_hash: &str,
        previous_block_hash: &str,
        merkle_root: &str,
        timestamp: u64,
        nonce: u64,
    ) -> Self {
        // Steps:
        // 1. Convert the three string fields into owned `String`s.
        // 2. Store `timestamp` and `nonce` unchanged.
        // 3. Return a `BlockHeader`.
        Self {
            block_hash: block_hash.to_string(),
            previous_block_hash: previous_block_hash.to_string(),
            merkle_root: merkle_root.to_string(),
            timestamp,
            nonce,
        }
    }
}

impl Block {
    /// Build a block from the provided header, transactions, height, and network.
    pub fn new(
        header: BlockHeader,
        transactions: Vec<Transaction>,
        height: u64,
        network: Network,
    ) -> Self {
        // Steps:
        // 1. Move `header` and `transactions` into the block.
        // 2. Store `height` and `network` unchanged.
        // 3. Return a `Block`.
        Self {
            header,
            transactions,
            height,
            network,
        }
    }

    /// Return how many transactions are in this block.
    pub fn transaction_count(&self) -> usize {
        // Steps:
        // 1. Return `self.transactions.len()`.
        self.transactions.len()
    }

    /// Sum the total output value of every transaction in the block.
    pub fn total_output_value(&self) -> u64 {
        // Steps:
        // 1. Start a total at 0.
        // 2. Add `transaction.total_output_value()` for each transaction.
        // 3. Return the total.
        self.transactions
            .iter()
            .map(Transaction::total_output_value)
            .sum()
    }

    /// Return the first transaction with the matching txid, if one exists.
    pub fn find_transaction(&self, txid: &str) -> Option<&Transaction> {
        // Steps:
        // 1. Walk through transactions in order.
        // 2. Return `Some(transaction)` for the first exact txid match.
        // 3. Return `None` when no match exists.
        self.transactions
            .iter()
            .find(|transaction| transaction.txid == txid)
    }
}

impl Hashable for Block {
    /// Return deterministic block hash material.
    ///
    /// Use exactly this format:
    /// `block:<block_hash>|prev:<previous_block_hash>|height:<height>|txs:<txid>;...`
    fn hash_material(&self) -> String {
        // Steps:
        // 1. Start with block hash, previous hash, and height in the format above.
        // 2. Append each transaction id followed by `;`.
        // 3. Return the final string.
        let txids = self
            .transactions
            .iter()
            .map(|transaction| format!("{};", transaction.txid))
            .collect::<String>();

        format!(
            "block:{}|prev:{}|height:{}|txs:{txids}",
            self.header.block_hash, self.header.previous_block_hash, self.height
        )
    }
}

impl Validate for Block {
    /// Validate a block and its transactions.
    fn validate(&self) -> Result<(), BtcLibError> {
        // Steps:
        // 1. If there are no transactions, return `Err(BtcLibError::EmptyBlock)`.
        // 2. Check for duplicate transaction ids. Return `DuplicateTxId` on repeat.
        // 3. Validate every transaction and return the first validation error.
        // 4. Otherwise return `Ok(())`.
        if self.transactions.is_empty() {
            return Err(BtcLibError::EmptyBlock);
        }

        let mut txids = std::collections::HashSet::new();
        for transaction in &self.transactions {
            if !txids.insert(transaction.txid.as_str()) {
                return Err(BtcLibError::DuplicateTxId);
            }
        }

        validate_all(&self.transactions)
    }
}

/// Parse `spent` or `unspent` into a `TxStatus`.
///
/// Trim whitespace, ignore ASCII case, and reject unknown values.
pub fn parse_status(input: &str) -> Result<TxStatus, BtcLibError> {
    // Steps:
    // 1. Trim whitespace from `input`.
    // 2. Compare using lowercase text.
    // 3. Return `Ok(TxStatus::Spent)` for "spent".
    // 4. Return `Ok(TxStatus::Unspent)` for "unspent".
    // 5. Return `Err(BtcLibError::MalformedData)` for anything else.
    match input.trim().to_ascii_lowercase().as_str() {
        "spent" => Ok(TxStatus::Spent),
        "unspent" => Ok(TxStatus::Unspent),
        _ => Err(BtcLibError::MalformedData),
    }
}

/// Parse a previous output reference.
///
/// The coinbase marker is `-`. Normal outpoints use `previous_txid:vout`.
pub fn parse_outpoint(input: &str) -> Result<Option<TxInput>, BtcLibError> {
    // Steps:
    // 1. Trim `input`.
    // 2. If it is exactly `COINBASE_PREVIOUS_OUTPUT`, return `Ok(None)`.
    // 3. Otherwise split once on `:`.
    // 4. Reject missing txid, missing vout, or non-numeric vout.
    // 5. Return `Ok(Some(TxInput::new(previous_txid, vout)))`.
    let input = input.trim();
    if input == COINBASE_PREVIOUS_OUTPUT {
        return Ok(None);
    }

    let (previous_txid, previous_vout) = input.split_once(':').ok_or(BtcLibError::MalformedData)?;
    let previous_txid = previous_txid.trim();
    let previous_vout = previous_vout
        .trim()
        .parse::<u32>()
        .map_err(|_| BtcLibError::MalformedData)?;

    if previous_txid.is_empty() {
        return Err(BtcLibError::MalformedData);
    }

    Ok(Some(TxInput::new(previous_txid, previous_vout)))
}

/// Parse a row into the Week 3 transaction model.
///
/// Row format: `txid,previous_txid:vout,recipient,amount_sats,status`.
/// For coinbase, use `coinbase,-,recipient,amount_sats,status`.
pub fn parse_transaction(input: &str) -> Result<Transaction, BtcLibError> {
    // Steps:
    // 1. Split the row by commas.
    // 2. Require exactly five fields.
    // 3. Trim every field and reject empty txid, recipient, amount, or status.
    // 4. Parse the previous output field with `parse_outpoint`.
    // 5. If the previous output is `-`, require `txid == "coinbase"`.
    // 6. Parse amount as `u64` and reject zero.
    // 7. Parse status with `parse_status`.
    // 8. Build one output and a transaction with zero or one input.
    // 9. Do not use `unwrap()` or `expect()` in this parser.
    let fields = input.split(',').map(str::trim).collect::<Vec<_>>();
    let [txid, previous_output, recipient, amount_sats, status] = fields.as_slice() else {
        return Err(BtcLibError::MalformedData);
    };

    if txid.is_empty()
        || previous_output.is_empty()
        || recipient.is_empty()
        || amount_sats.is_empty()
        || status.is_empty()
    {
        return Err(BtcLibError::MalformedData);
    }

    let input = parse_outpoint(previous_output)?;
    if input.is_none() && *txid != "coinbase" {
        return Err(BtcLibError::MalformedData);
    }

    let amount_sats = amount_sats
        .parse::<u64>()
        .map_err(|_| BtcLibError::MalformedData)?;
    if amount_sats == 0 {
        return Err(BtcLibError::MalformedData);
    }

    let status = parse_status(status)?;
    let inputs = input.into_iter().collect();
    let outputs = vec![TxOutput::new(amount_sats, recipient, status)];

    Ok(Transaction::new(txid, inputs, outputs))
}

/// Parse every row into a transaction.
///
/// Stop and return the first error if any row is malformed.
pub fn parse_transactions(lines: &[&str]) -> Result<Vec<Transaction>, BtcLibError> {
    // Steps:
    // 1. Create an empty `Vec<Transaction>`.
    // 2. Parse rows from first to last with `parse_transaction`.
    // 3. Push valid transactions into the vector.
    // 4. If a row returns an error, return that error immediately.
    // 5. Return `Ok(vec)` when all rows parse successfully.
    lines.iter().map(|line| parse_transaction(line)).collect()
}

/// Parse all valid rows and skip malformed rows.
pub fn valid_transactions_only(lines: &[&str]) -> Vec<Transaction> {
    // Steps:
    // 1. Create an empty `Vec<Transaction>`.
    // 2. Try to parse every row.
    // 3. Push only successfully parsed transactions.
    // 4. Silently skip malformed rows.
    lines
        .iter()
        .filter_map(|line| parse_transaction(line).ok())
        .collect()
}

/// Build and validate a block from parsed transaction rows.
pub fn build_block_from_rows(
    header: BlockHeader,
    rows: &[&str],
    height: u64,
    network: Network,
) -> Result<Block, BtcLibError> {
    // Steps:
    // 1. Parse all rows with `parse_transactions`.
    // 2. Build a `Block` from the parsed transactions.
    // 3. Validate the block.
    // 4. Return the block only when parsing and validation succeed.
    let transactions = parse_transactions(rows)?;
    let block = Block::new(header, transactions, height, network);
    block.validate()?;
    Ok(block)
}

/// Validate every item in order.
///
/// Stop and return the first validation error, otherwise return `Ok(())`.
pub fn validate_all<T: Validate>(items: &[T]) -> Result<(), BtcLibError> {
    // Steps:
    // 1. Walk through `items` in order.
    // 2. Call `validate()` on each item.
    // 3. Return the first error immediately.
    // 4. Return `Ok(())` if every item is valid.
    items.iter().try_for_each(Validate::validate)
}

/// Return the SHA-256 hex hash for every hashable item, preserving input order.
pub fn hash_all<T: Hashable>(items: &[T]) -> Vec<String> {
    // Steps:
    // 1. Create a new `Vec<String>`.
    // 2. For each item, call `hash_hex()`.
    // 3. Push the hash into the output vector.
    // 4. Preserve the original order.
    items.iter().map(Hashable::hash_hex).collect()
}

/// Decode a 64-character SHA-256 hex string into 32 bytes.
pub fn decode_hash_hex(input: &str) -> Result<[u8; 32], BtcLibError> {
    // Steps:
    // 1. Trim whitespace from `input`.
    // 2. Decode the string with the `hex` crate.
    // 3. Reject invalid hex or decoded values that are not exactly 32 bytes.
    // 4. Convert the decoded bytes into `[u8; 32]`.
    // 5. Return `Err(BtcLibError::InvalidHash)` for invalid input.
    let bytes = hex::decode(input.trim()).map_err(|_| BtcLibError::InvalidHash)?;
    bytes.try_into().map_err(|_| BtcLibError::InvalidHash)
}

/// Sum unspent output amounts across all transactions.
pub fn total_unspent(transactions: &[Transaction]) -> u64 {
    // Steps:
    // 1. Walk through every transaction and every output.
    // 2. Add `value_sats` only when the output is unspent.
    // 3. Return the total.
    transactions
        .iter()
        .flat_map(|transaction| transaction.outputs.iter())
        .filter(|output| output.is_unspent())
        .map(|output| output.value_sats)
        .sum()
}

/// Return a borrowed transaction with the matching txid, if one exists.
pub fn find_by_txid<'a>(transactions: &'a [Transaction], txid: &str) -> Option<&'a Transaction> {
    // Steps:
    // 1. Walk through the slice from first to last.
    // 2. Compare each transaction's txid with `txid`.
    // 3. Return `Some(transaction)` for the first exact match.
    // 4. Return `None` if no match exists.
    transactions
        .iter()
        .find(|transaction| transaction.txid == txid)
}

/// Return the matching transaction or `BtcLibError::MissingTransaction`.
pub fn require_transaction<'a>(
    transactions: &'a [Transaction],
    txid: &str,
) -> Result<&'a Transaction, BtcLibError> {
    // Steps:
    // 1. Reuse `find_by_txid` or perform the same lookup.
    // 2. Return `Ok(transaction)` when found.
    // 3. Return `Err(BtcLibError::MissingTransaction)` when missing.
    find_by_txid(transactions, txid).ok_or(BtcLibError::MissingTransaction)
}

/// Build an amount summary from all transaction outputs.
pub fn summarize_amounts(transactions: &[Transaction]) -> AmountSummary {
    // Steps:
    // 1. Count every output across every transaction.
    // 2. Sum every output amount into `total_sats`.
    // 3. Sum spent output amounts into `spent_sats`.
    // 4. Sum unspent output amounts into `unspent_sats`.
    // 5. Return an `AmountSummary` with all four fields filled.
    transactions
        .iter()
        .flat_map(|transaction| transaction.outputs.iter())
        .fold(
            AmountSummary {
                output_count: 0,
                total_sats: 0,
                spent_sats: 0,
                unspent_sats: 0,
            },
            |mut summary, output| {
                summary.output_count += 1;
                summary.total_sats += output.value_sats;
                match output.status {
                    TxStatus::Spent => summary.spent_sats += output.value_sats,
                    TxStatus::Unspent => summary.unspent_sats += output.value_sats,
                }
                summary
            },
        )
}
