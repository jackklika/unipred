use super::ClientResult;
use alloy_primitives::U256;
use alloy_primitives::{hex::encode_prefixed, Address};
use alloy_signer::{Signer, SignerSync};
use alloy_sol_types::{eip712_domain, sol};
use anyhow::Context;

pub trait EthSigner: Signer + SignerSync + Send + Sync {}

impl<T: Signer + SignerSync + Send + Sync> EthSigner for T {}

sol! {
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }
}

sol! {
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

pub fn sign_clob_auth_message(
    signer: &impl EthSigner,
    timestamp: String,
    nonce: U256,
) -> ClientResult<String> {
    // TODO: move these constants out
    let message = "This message attests that I control the given wallet".to_owned();
    let polygon = 137;

    let my_struct = ClobAuth {
        address: signer.address(),
        timestamp,
        nonce,
        message,
    };

    let my_domain = eip712_domain!(
        name: "ClobAuthDomain",
        version: "1",
        chain_id: polygon,
    );

    let val = signer
        .sign_typed_data_sync(&my_struct, &my_domain)
        .context("Error creating EIP-712 signature")?;

    Ok(encode_prefixed(val.as_bytes()))
}

pub fn sign_order_message(
    signer: &(impl EthSigner + Sized),
    order: Order,
    chain_id: u64,
    verifying_contract: Address,
) -> ClientResult<String> {
    let domain = eip712_domain!(
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: chain_id,
        verifying_contract: verifying_contract,

    );

    let val = signer
        .sign_typed_data_sync(&order, &domain)
        .context("Error creating EIP-712 signature for order")?;

    Ok(encode_prefixed(val.as_bytes()))
}
