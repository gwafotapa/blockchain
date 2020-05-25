use blockchain::utxo::{UtxoData, UtxoId};

pub mod common;

#[test]
fn test_utxo_id_ser_deser() {
    let utxo_id = common::random_utxo_id(None, None);
    let utxo_id2 = UtxoId::deserialize(utxo_id.serialize());
    assert_eq!(utxo_id, utxo_id2);
}

#[test]
fn test_utxo_data_ser_deser() {
    let utxo_data = common::random_utxo_data(None, None);
    let utxo_data2 = UtxoData::deserialize(utxo_data.serialize());
    assert_eq!(utxo_data, utxo_data2);
}
