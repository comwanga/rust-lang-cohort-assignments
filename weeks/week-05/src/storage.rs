use std::fs;
use std::path::Path;

use crate::{Block, Blockchain, BtcLibError, Validate};

/// Save a single block as pretty JSON.
pub fn save_block_to_file<P: AsRef<Path>>(block: &Block, path: P) -> Result<(), BtcLibError> {
    // Steps:
    // 1. Validate `block` before saving it.
    // 2. Serialize it with `serde_json::to_string_pretty`.
    // 3. Write the JSON string to `path` with `std::fs::write`.
    // 4. Convert serde and IO errors through `?`.
    block.validate()?;
    let json = serde_json::to_string_pretty(block)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load a single block from JSON and validate it.
pub fn load_block_from_file<P: AsRef<Path>>(path: P) -> Result<Block, BtcLibError> {
    // Steps:
    // 1. Read the file into a string with `std::fs::read_to_string`.
    // 2. Deserialize a `Block` with `serde_json::from_str`.
    // 3. Validate the loaded block.
    // 4. Return the block only when all steps succeed.
    let json = fs::read_to_string(path)?;
    let block = serde_json::from_str::<Block>(&json)?;
    block.validate()?;
    Ok(block)
}

/// Save a full blockchain snapshot as pretty JSON.
pub fn save_chain_to_file<P: AsRef<Path>>(chain: &Blockchain, path: P) -> Result<(), BtcLibError> {
    // Steps:
    // 1. Validate the chain before saving.
    // 2. Serialize it with `serde_json::to_string_pretty`.
    // 3. Write it to `path`.
    // 4. Propagate errors with `?`.
    chain.validate()?;
    let json = serde_json::to_string_pretty(chain)?;
    fs::write(path, json)?;
    Ok(())
}

/// Load a full blockchain snapshot from JSON and validate it.
pub fn load_chain_from_file<P: AsRef<Path>>(path: P) -> Result<Blockchain, BtcLibError> {
    // Steps:
    // 1. Read the file into a string.
    // 2. Deserialize a `Blockchain`.
    // 3. Validate the loaded chain before returning it.
    // 4. Propagate serde and IO failures through `BtcLibError`.
    let json = fs::read_to_string(path)?;
    let chain = serde_json::from_str::<Blockchain>(&json)?;
    chain.validate()?;
    Ok(chain)
}
