use crate::{Block, BtcLibError, Transaction};

pub trait Hashable {
    /// Return stable material that will be hashed.
    fn hash_material(&self) -> String;

    /// Hash `hash_material()` with SHA-256 and return lowercase hex.
    fn hash_hex(&self) -> String {
        sha256::digest(self.hash_material())
    }
}

impl Hashable for Transaction {
    /// Return deterministic transaction material.
    ///
    /// Use exactly:
    /// `tx:<txid>|inputs:<previous_txid>:<previous_vout>;...|outputs:<value>:<recipient>:<status>;...`
    ///
    /// Status text must be lowercase: `spent` or `unspent`.
    fn hash_material(&self) -> String {
        // Steps:
        // 1. Start with `tx:<txid>|inputs:`.
        // 2. Append every input as `<previous_txid>:<previous_vout>;`.
        // 3. Append `|outputs:`.
        // 4. Append every output as `<value_sats>:<recipient>:<status>;`.
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
                    crate::TxStatus::Spent => "spent",
                    crate::TxStatus::Unspent => "unspent",
                };
                format!("{}:{}:{};", output.value_sats, output.recipient, status)
            })
            .collect::<String>();

        format!("tx:{}|inputs:{inputs}|outputs:{outputs}", self.txid)
    }
}

impl Hashable for Block {
    /// Return deterministic block material.
    ///
    /// Use exactly:
    /// `block:<block_hash>|prev:<previous_block_hash>|merkle:<merkle_root>|height:<height>|txs:<txid>;...`
    fn hash_material(&self) -> String {
        // Steps:
        // 1. Start with block hash, previous hash, merkle root, and height in the format above.
        // 2. Append each transaction id followed by `;`.
        // 3. Return the final string.
        let txids = self
            .transactions
            .iter()
            .map(|transaction| format!("{};", transaction.txid))
            .collect::<String>();

        format!(
            "block:{}|prev:{}|merkle:{}|height:{}|txs:{txids}",
            self.header.block_hash,
            self.header.previous_block_hash,
            self.header.merkle_root,
            self.height
        )
    }
}

/// Hash two child hashes into their parent merkle node.
pub fn pair_hash(left: &str, right: &str) -> String {
    // Steps:
    // 1. Build the exact string `<left><right>` with no separator.
    // 2. Return `sha256::digest(joined_string)`.
    sha256::digest(format!("{left}{right}"))
}

/// Calculate a simple merkle root from transaction hashes.
///
/// If a level has an odd number of hashes, duplicate the last hash before pairing.
pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<String, BtcLibError> {
    // Steps:
    // 1. Reject an empty transaction slice with `BtcLibError::EmptyBlock`.
    // 2. Start with each transaction's `hash_hex()`.
    // 3. While more than one hash remains, pair hashes left-to-right.
    // 4. When a level has an odd count, pair the last hash with itself.
    // 5. Return the only remaining hash.
    if transactions.is_empty() {
        return Err(BtcLibError::EmptyBlock);
    }

    let mut level = transactions
        .iter()
        .map(Hashable::hash_hex)
        .collect::<Vec<_>>();

    while level.len() > 1 {
        level = level
            .chunks(2)
            .map(|pair| {
                let right = pair.get(1).unwrap_or(&pair[0]);
                pair_hash(&pair[0], right)
            })
            .collect();
    }

    level.pop().ok_or(BtcLibError::EmptyBlock)
}

/// Validate that the block header stores the merkle root for its transactions.
pub fn validate_merkle_root(block: &Block) -> Result<(), BtcLibError> {
    // Steps:
    // 1. Calculate the expected merkle root for `block.transactions`.
    // 2. Compare it with `block.header.merkle_root`.
    // 3. Return `Ok(())` on an exact match.
    // 4. Return `Err(BtcLibError::InvalidMerkleRoot)` on mismatch.
    if calculate_merkle_root(&block.transactions)? == block.header.merkle_root {
        Ok(())
    } else {
        Err(BtcLibError::InvalidMerkleRoot)
    }
}
