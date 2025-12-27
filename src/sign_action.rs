use alloy::{
    primitives::{B256, keccak256},
    signers::{Signature, SignerSync, local::PrivateKeySigner},
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

// Compute domain separator manually to match Python SDK exactly
fn domain_separator() -> B256 {
    // EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)
    let type_hash = keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );
    let name_hash = keccak256("Exchange");
    let version_hash = keccak256("1");

    let mut encoded = Vec::new();
    encoded.extend_from_slice(type_hash.as_slice());
    encoded.extend_from_slice(name_hash.as_slice());
    encoded.extend_from_slice(version_hash.as_slice());
    // chainId as uint256 (32 bytes, big endian)
    encoded.extend_from_slice(&[0u8; 24]); // padding
    encoded.extend_from_slice(&1337u64.to_be_bytes());
    // verifyingContract as address (32 bytes, left-padded)
    encoded.extend_from_slice(&[0u8; 32]); // Address::ZERO

    let result = keccak256(&encoded);
    result
}

// Compute Agent struct hash manually
fn agent_struct_hash(source: &str, connection_id: B256) -> B256 {
    // Agent(string source,bytes32 connectionId)
    let type_hash = keccak256("Agent(string source,bytes32 connectionId)");
    let source_hash = keccak256(source);

    let mut encoded = Vec::new();
    encoded.extend_from_slice(type_hash.as_slice());
    encoded.extend_from_slice(source_hash.as_slice());
    encoded.extend_from_slice(connection_id.as_slice());

    let result = keccak256(&encoded);
    result
}

fn eip712_signing_hash(source: &str, connection_id: B256) -> B256 {
    let domain_sep = domain_separator();
    let struct_hash = agent_struct_hash(source, connection_id);

    let mut digest_input = [0u8; 2 + 32 + 32];
    digest_input[0] = 0x19;
    digest_input[1] = 0x01;
    digest_input[2..34].copy_from_slice(domain_sep.as_slice());
    digest_input[34..66].copy_from_slice(struct_hash.as_slice());

    let result = keccak256(digest_input);
    result
}

pub async fn sign_action(
    private_key: &str,
    action: &SignAction,
    nonce: u64,
    expires_after: u64,
) -> anyhow::Result<Signature> {
    let connection_id = action_hash(action, nonce, expires_after);
    let source = "a"; // mainnet

    let signing_hash = eip712_signing_hash(source, connection_id);

    let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer: PrivateKeySigner = pk.parse()?;

    signer
        .sign_hash_sync(&signing_hash)
        .map_err(|e| anyhow::anyhow!("Failed to sign: {}", e))
}

fn action_hash(action: &SignAction, nonce: u64, expires_after: u64) -> B256 {
    let mut data = rmp_serde::to_vec_named(action).expect("Failed to msgpack serialize action");

    // Append nonce as 8 bytes big endian
    data.extend_from_slice(&nonce.to_be_bytes());

    // No vault address (0x00 indicates null)
    data.push(0x00);

    // Expires after: 0x00 prefix + 8 bytes big endian
    data.push(0x00);
    data.extend_from_slice(&expires_after.to_be_bytes());

    let hash = keccak256(&data);
    hash
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignAction {
    #[serde(rename = "type")]
    pub type_: String,
    pub orders: Vec<Order>,
    pub grouping: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Order {
    pub a: u32,
    pub b: bool,
    pub p: String,
    pub s: String,
    pub r: bool,
    pub t: OrderType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderType {
    pub limit: Limit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Limit {
    pub tif: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExchangePayload {
    pub action: SignAction,
    pub nonce: u64,
    #[serde(serialize_with = "serialize_sig")]
    pub signature: Signature,
    pub vault_address: Option<String>,
    pub expires_after: u64,
}

fn serialize_sig<S>(sig: &Signature, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut state = s.serialize_struct("Signature", 3)?;
    state.serialize_field("r", &sig.r())?;
    state.serialize_field("s", &sig.s())?;
    state.serialize_field("v", &(27 + sig.v() as u64))?;
    state.end()
}
