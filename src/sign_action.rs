use alloy::{
    dyn_abi::Eip712Domain,
    primitives::{Address, B256, keccak256},
    signers::{Signature, SignerSync, local::PrivateKeySigner},
    sol,
    sol_types::{SolStruct, eip712_domain},
};
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

sol! {
    #[derive(Debug)]
    struct Agent {
        string source;
        bytes32 connectionId;
    }
}

impl Eip712 for Agent {
    fn domain(&self) -> Eip712Domain {
        eip712_domain! {
            name: "Exchange",
            version: "1",
            chain_id: 1337,
            verifying_contract: Address::ZERO,
        }
    }
    fn struct_hash(&self) -> B256 {
        self.eip712_hash_struct()
    }
}

pub async fn sign_action(
    private_key: &str,
    action: &Action,
    nonce: u64,
    expires_after: u64,
) -> anyhow::Result<Signature> {
    let connection_id = action_hash(action, nonce, expires_after);
    let source = "a".to_string();
    let payload = Agent {
        source,
        connectionId: connection_id,
    };
    sign_typed_data(&payload, private_key)
}

fn action_hash(action: &Action, nonce: u64, expires_after: u64) -> B256 {
    let mut data = rmp_serde::to_vec_named(action).expect("Failed to msgpack serialize action");

    // Append nonce as 8 bytes big endian
    data.extend_from_slice(&nonce.to_be_bytes());
    data.push(0x00);
    data.push(0x00);
    data.extend_from_slice(&expires_after.to_be_bytes());

    keccak256(&data)
}

fn sign_typed_data<T: Eip712>(payload: &T, private_key: &str) -> anyhow::Result<Signature> {
    let pk = private_key.strip_prefix("0x").unwrap_or(private_key);
    let signer: PrivateKeySigner = pk.parse()?;
    signer
        .sign_hash_sync(&payload.eip712_signing_hash())
        .map_err(|e| anyhow::anyhow!("Failed to sign typed data: {}", e))
}

#[derive(Serialize, Deserialize)]
pub struct Action {
    pub action: SignAction,
}

#[derive(Serialize, Deserialize)]
pub struct SignAction {
    #[serde(rename = "type")]
    pub type_: String,
    pub orders: Vec<Order>,
    pub grouping: String,
}

#[derive(Serialize, Deserialize)]
pub struct Order {
    pub a: u32,
    pub b: bool,
    pub p: String,
    pub s: String,
    pub r: bool,
    pub t: OrderType,
}

#[derive(Serialize, Deserialize)]
pub struct OrderType {
    pub limit: Limit,
}

#[derive(Serialize, Deserialize)]
pub struct Limit {
    pub tif: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangePayload {
    pub action: Action,
    #[serde(serialize_with = "serialize_sig")]
    pub signature: Signature,
    pub nonce: u64,
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

pub trait Eip712 {
    fn domain(&self) -> Eip712Domain;
    fn struct_hash(&self) -> B256;

    fn eip712_signing_hash(&self) -> B256 {
        let mut digest_input = [0u8; 2 + 32 + 32];
        digest_input[0] = 0x19;
        digest_input[1] = 0x01;
        digest_input[2..34].copy_from_slice(&self.domain().hash_struct()[..]);
        digest_input[34..66].copy_from_slice(&self.struct_hash()[..]);
        keccak256(digest_input)
    }
}
