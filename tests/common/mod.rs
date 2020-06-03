use std::cmp;
use std::collections::HashSet;
use std::sync::Once;

use rand::Rng;
use rand_core::RngCore;
use secp256k1::{PublicKey, Secp256k1, SecretKey, Signature};

use blockchain::transaction::{Transaction, TransactionInput, TransactionOutput};
use blockchain::utxo::{Utxo, UtxoData, UtxoId};
use blockchain::utxo_pool::UtxoPool;
use blockchain::Hash;

static INIT: Once = Once::new();

pub const VOUT_MAX: usize = 10;
pub const AMOUNT_MAX: u32 = 1000000;
pub const INPUTS_LEN_MAX: usize = 10;
pub const OUTPUTS_LEN_MAX: usize = 10;
pub const UTXO_POOL_SIZE_MAX: usize = 10;
pub const KEYS: usize = 10;
pub const RECIPIENTS_MAX: usize = 10;
pub const UTXOS_PER_KEY_MAX: usize = 10;

pub fn log_setup() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

fn random_hash() -> Hash {
    let mut hash = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut hash);
    Hash::from(hash)
}

pub fn random_utxo_id(txid: Option<Hash>, vout: Option<usize>) -> UtxoId {
    let txid = txid.unwrap_or_else(random_hash);
    let vout = vout.unwrap_or_else(|| rand::thread_rng().gen_range(0, VOUT_MAX));
    UtxoId::new(txid, vout)
}

pub fn random_utxo_data(amount: Option<u32>, public_key: Option<PublicKey>) -> UtxoData {
    let amount = amount.unwrap_or_else(|| rand::thread_rng().gen_range(0, AMOUNT_MAX));
    let public_key = public_key.unwrap_or_else(random_public_key);
    UtxoData::new(amount, public_key)
}

pub fn random_utxo_with(
    txid: Option<Hash>,
    vout: Option<usize>,
    amount: Option<u32>,
    public_key: Option<PublicKey>,
) -> Utxo {
    let utxo_id = random_utxo_id(txid, vout);
    let utxo_data = random_utxo_data(amount, public_key);
    Utxo::new(utxo_id, utxo_data)
}

pub fn random_utxo(id: Option<UtxoId>, data: Option<UtxoData>) -> Utxo {
    random_utxo_with(
        id.map(|id| *id.txid()),
        id.map(|id| id.vout()),
        data.map(|data| data.amount()),
        data.map(|data| *data.public_key()),
    )
}

pub fn random_public_key() -> PublicKey {
    let mut secret_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_key);
    let secret_key = SecretKey::from_slice(&secret_key).unwrap();
    let secp = Secp256k1::new();
    PublicKey::from_secret_key(&secp, &secret_key)
}

pub fn random_secret_key() -> SecretKey {
    let mut secret_key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_key);
    SecretKey::from_slice(&secret_key).unwrap()
}

pub fn random_transaction_input(txid: Option<Hash>, vout: Option<usize>) -> TransactionInput {
    let utxo_id = random_utxo_id(txid, vout);
    let mut sig = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut sig);
    let sig = Signature::from_compact(&sig).unwrap();
    TransactionInput::new(utxo_id, sig)
}

pub fn random_transaction_output(
    amount: Option<u32>,
    public_key: Option<PublicKey>,
) -> TransactionOutput {
    TransactionOutput::from(random_utxo_data(amount, public_key))
}

pub fn random_transaction(
    inputs: Option<Vec<TransactionInput>>,
    outputs: Option<Vec<TransactionOutput>>,
) -> Transaction {
    let mut rng = rand::thread_rng();
    let outputs = outputs.unwrap_or_else(|| {
        let outputs_len = rng.gen_range(1, OUTPUTS_LEN_MAX);
        (0..outputs_len)
            .map(|_| random_transaction_output(None, None))
            .collect()
    });
    if let Some(inputs) = inputs {
        Transaction::new(inputs, outputs)
    } else {
        let sk = random_secret_key();
        let inputs_len = rng.gen_range(1, INPUTS_LEN_MAX);
        let mut utxo_ids = HashSet::new();
        while utxo_ids.len() != inputs_len {
            utxo_ids.insert(random_utxo_id(None, None));
        }
        Transaction::sign(utxo_ids.into_iter().collect(), outputs, &sk)
    }
}

pub fn random_transaction_with(
    sender: Option<SecretKey>,
    recipients: Option<Vec<PublicKey>>,
    inputs: Option<Vec<Utxo>>,
    amounts: Option<Vec<u32>>,
) -> Transaction {
    if recipients.is_some() && amounts.is_some() {
        assert_eq!(
            recipients.as_ref().map(Vec::len),
            amounts.as_ref().map(Vec::len)
        );
    }

    let mut rng = rand::thread_rng();
    let secret_key = sender.unwrap_or_else(random_secret_key);
    let input_utxo_ids = if let Some(utxos) = inputs.as_ref() {
        utxos.iter().map(|utxo| *utxo.id()).collect()
    } else {
        let inputs_len = rng.gen_range(1, INPUTS_LEN_MAX);
        let mut utxo_ids = HashSet::new();
        while utxo_ids.len() != inputs_len {
            utxo_ids.insert(random_utxo_id(None, None));
        }
        utxo_ids.into_iter().collect()
    };
    let mut sum = if let Some(amounts) = amounts.as_ref() {
        amounts.iter().sum()
    } else if let Some(utxos) = inputs {
        utxos.iter().map(|u| u.amount()).sum()
    } else {
        let amounts_len = recipients
            .as_ref()
            .map(Vec::len)
            .unwrap_or_else(|| rng.gen_range(1, RECIPIENTS_MAX)) as u32;
        rng.gen_range(amounts_len, amounts_len * (AMOUNT_MAX - 1) + 1)
    };
    let recipients = recipients.unwrap_or_else(|| {
        let recipients_len = amounts
            .as_ref()
            .map(Vec::len)
            .unwrap_or_else(|| rng.gen_range(1, cmp::min(OUTPUTS_LEN_MAX, (sum + 1) as usize)));
        (0..recipients_len).map(|_| random_public_key()).collect()
    });
    let amounts = amounts.unwrap_or_else(|| {
        let mut amounts = Vec::with_capacity(recipients.len());
        for i in (0..recipients.len() as u32 - 1).rev() {
            let amount = rng.gen_range(1, sum + 1 - i);
            amounts.push(amount);
            sum -= amount;
        }
        amounts.push(sum);
        amounts
    });
    let outputs = amounts
        .into_iter()
        .zip(recipients)
        .map(|(a, r)| TransactionOutput::new(a, r))
        .collect();
    Transaction::sign(input_utxo_ids, outputs, &secret_key)
}

pub fn random_utxo_pool(utxos: Option<HashSet<Utxo>>) -> UtxoPool {
    let mut rng = rand::thread_rng();
    let utxos = utxos.unwrap_or_else(|| {
        let utxos_len = rng.gen_range(1, UTXO_POOL_SIZE_MAX);
        (0..utxos_len).map(|_| random_utxo(None, None)).collect()
    });
    UtxoPool::from(utxos)
}

pub fn random_key() -> (PublicKey, SecretKey) {
    let secret_key = random_secret_key();
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    (public_key, secret_key)
}
