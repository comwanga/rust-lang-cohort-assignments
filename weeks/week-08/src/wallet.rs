use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    derive_wallet_txid, OutPoint, Transaction, TxInput, TxOutput, WalletError, WalletUtxo,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    pub owner: String,
    pub utxos: BTreeMap<OutPoint, WalletUtxo>,
    pub pending: Vec<Transaction>,
    pub history: Vec<Transaction>,
}

impl Wallet {
    /// Create an empty wallet for one owner label.
    pub fn new(owner: &str) -> Self {
        // Steps:
        // 1. Convert `owner` into an owned `String`.
        // 2. Start with empty UTXO, pending, and history collections.
        // 3. Return the wallet.
        Self {
            owner: owner.to_string(),
            utxos: BTreeMap::new(),
            pending: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Import one UTXO into the wallet.
    pub fn import_utxo(&mut self, utxo: WalletUtxo) {
        // Steps:
        // 1. Insert the UTXO by cloning its outpoint as the map key.
        // 2. Replace any existing entry at the same outpoint.
        self.utxos.insert(utxo.outpoint.clone(), utxo);
    }

    /// Sum confirmed spendable UTXOs owned by this wallet.
    pub fn confirmed_balance(&self) -> u64 {
        // Steps:
        // 1. Iterate over wallet UTXOs.
        // 2. Include only UTXOs whose owner matches `self.owner`.
        // 3. Include only UTXOs with confirmations greater than 0.
        // 4. Return the sum.
        self.utxos
            .values()
            .filter(|utxo| utxo.owner == self.owner && utxo.confirmations > 0)
            .map(|utxo| utxo.value_sats)
            .sum()
    }

    /// Sum outputs in pending transactions that pay this wallet.
    pub fn pending_incoming_balance(&self) -> u64 {
        // Steps:
        // 1. Iterate over pending transactions.
        // 2. Add outputs whose recipient matches `self.owner`.
        // 3. Return the sum.
        self.pending
            .iter()
            .flat_map(|transaction| &transaction.outputs)
            .filter(|output| output.recipient == self.owner)
            .map(|output| output.value_sats)
            .sum()
    }

    /// Return owned, confirmed UTXOs in deterministic outpoint order.
    pub fn available_utxos(&self) -> Vec<WalletUtxo> {
        // Steps:
        // 1. Iterate over the `BTreeMap` values.
        // 2. Keep UTXOs owned by `self.owner` with confirmations > 0.
        // 3. Clone them into a vector.
        // 4. Return the vector.
        self.utxos
            .values()
            .filter(|utxo| utxo.owner == self.owner && utxo.confirmations > 0)
            .cloned()
            .collect()
    }

    /// Select UTXOs until `amount_sats + fee_sats` is covered.
    pub fn select_utxos(
        &self,
        amount_sats: u64,
        fee_sats: u64,
    ) -> Result<Vec<WalletUtxo>, WalletError> {
        // Steps:
        // 1. Reject `amount_sats == 0` with `InvalidAmount`.
        // 2. Iterate through `available_utxos()` in order.
        // 3. Keep selecting until total >= amount + fee.
        // 4. Return selected UTXOs.
        // 5. Return `InsufficientFunds` if the total never covers the target.
        if amount_sats == 0 {
            return Err(WalletError::InvalidAmount);
        }
        let target = amount_sats
            .checked_add(fee_sats)
            .ok_or(WalletError::InvalidAmount)?;
        let mut selected = Vec::new();
        let mut total = 0_u64;
        for utxo in self.available_utxos() {
            total = total
                .checked_add(utxo.value_sats)
                .ok_or(WalletError::InvalidAmount)?;
            selected.push(utxo);
            if total >= target {
                return Ok(selected);
            }
        }
        Err(WalletError::InsufficientFunds)
    }

    /// Build but do not record a send transaction.
    pub fn build_transaction(
        &self,
        recipient: &str,
        amount_sats: u64,
        fee_sats: u64,
    ) -> Result<Transaction, WalletError> {
        // Steps:
        // 1. Reject empty recipient or zero amount with `InvalidAmount`.
        // 2. Select UTXOs for amount + fee.
        // 3. Create one input for each selected UTXO.
        // 4. Create a recipient output for `amount_sats`.
        // 5. If selected total is greater than amount + fee, add a change output to `self.owner`.
        // 6. Derive a deterministic txid with `derive_wallet_txid`.
        // 7. Return the transaction without mutating wallet state.
        if recipient.trim().is_empty() || amount_sats == 0 {
            return Err(WalletError::InvalidAmount);
        }
        let selected = self.select_utxos(amount_sats, fee_sats)?;
        let target = amount_sats
            .checked_add(fee_sats)
            .ok_or(WalletError::InvalidAmount)?;
        let selected_total = selected.iter().try_fold(0_u64, |total, utxo| {
            total
                .checked_add(utxo.value_sats)
                .ok_or(WalletError::InvalidAmount)
        })?;
        let inputs = selected
            .iter()
            .map(|utxo| TxInput {
                previous_output: utxo.outpoint.clone(),
            })
            .collect();
        let mut outputs = vec![TxOutput::new(amount_sats, recipient)];
        let change = selected_total - target;
        if change > 0 {
            outputs.push(TxOutput::new(change, &self.owner));
        }
        let txid = derive_wallet_txid(&self.owner, recipient, amount_sats, fee_sats, &selected);
        Ok(Transaction::new(&txid, inputs, outputs, fee_sats))
    }

    /// Record a transaction as pending and remove the spent UTXOs.
    pub fn record_pending(&mut self, transaction: Transaction) -> Result<(), WalletError> {
        // Steps:
        // 1. Before mutating, check that every input exists in `self.utxos`.
        // 2. Remove every spent UTXO.
        // 3. Push the transaction into `pending`.
        // 4. Push the transaction into `history`.
        // 5. Return `Ok(())`.
        let mut remaining = self.utxos.clone();
        for input in &transaction.inputs {
            let outpoint = &input.previous_output;
            if remaining.remove(outpoint).is_none() {
                return Err(WalletError::MissingUtxo(outpoint.label()));
            }
        }
        self.utxos = remaining;
        self.pending.push(transaction.clone());
        self.history.push(transaction);
        Ok(())
    }

    /// Apply a confirmed transaction from the node.
    pub fn apply_confirmed_transaction(&mut self, transaction: Transaction) {
        // Steps:
        // 1. Remove any pending transaction with the same txid.
        // 2. For every output paying `self.owner`, import it as a confirmed UTXO.
        // 3. Use the output index as `vout`.
        // 4. Add the transaction to history if it is not already present.
        self.pending
            .retain(|pending| pending.txid != transaction.txid);
        for (vout, output) in transaction.outputs.iter().enumerate() {
            if output.recipient == self.owner {
                self.import_utxo(WalletUtxo::new(
                    &transaction.txid,
                    vout as u32,
                    output.value_sats,
                    &self.owner,
                    1,
                ));
            }
        }
        if !self
            .history
            .iter()
            .any(|entry| entry.txid == transaction.txid)
        {
            self.history.push(transaction);
        }
    }

    /// Return compact history lines in insertion order.
    ///
    /// Each line must be: `<txid>|outputs:<total_output>|fee:<fee>`.
    pub fn history_lines(&self) -> Vec<String> {
        // Steps:
        // 1. Iterate over `self.history`.
        // 2. Format each transaction exactly as documented above.
        // 3. Return the lines.
        self.history
            .iter()
            .map(|transaction| {
                format!(
                    "{}|outputs:{}|fee:{}",
                    transaction.txid,
                    transaction.total_output_value(),
                    transaction.fee_sats
                )
            })
            .collect()
    }
}

/// Return a compact wallet summary.
///
/// Use exactly:
/// `owner:<owner>|confirmed:<confirmed>|pending_in:<pending>|pending_txs:<count>|history:<count>`
pub fn wallet_summary(wallet: &Wallet) -> String {
    // Steps:
    // 1. Read owner, confirmed balance, pending incoming balance, pending count, and history count.
    // 2. Return the exact format documented above.
    format!(
        "owner:{}|confirmed:{}|pending_in:{}|pending_txs:{}|history:{}",
        wallet.owner,
        wallet.confirmed_balance(),
        wallet.pending_incoming_balance(),
        wallet.pending.len(),
        wallet.history.len()
    )
}
