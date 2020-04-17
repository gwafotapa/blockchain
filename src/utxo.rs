use rand::Rng;
use std::collections::HashMap;

/// Number of initial utxos;
const TOTAL_UTXO: usize = 10;

/// Amount of initial utxos
const UTXO_AMOUNT: u32 = 10;

/// For now, the cryptographic puzzle is just a unique number identifying the utxo
// #[derive(Hash)]
pub struct Utxo {
    amount: u32,
    puzzle: usize,
}

impl PartialEq for Utxo {
    fn eq(&self, other: &Self) -> bool {
        self.puzzle == other.puzzle
    }
}

pub struct UtxoPool {
    total: usize,
    data: HashMap<usize, u32>,
}

impl UtxoPool {
    pub fn new() -> Self {
        Self {
            total: TOTAL_UTXO,
            data: (0..TOTAL_UTXO)
                .into_iter()
                .map(|x| (x, UTXO_AMOUNT))
                .collect(),
        }
    }

    pub fn contains(&self, utxo: &Utxo) -> bool {
        self.data.contains_key(&utxo.puzzle)
    }

    pub fn random_utxo(&self) -> Utxo {
        let mut rng = rand::thread_rng();
        let n = rng.gen_range(0, self.data.keys().len());
        let puzzle = *self.data.keys().nth(n).unwrap();
        let amount = *self.data.get(&puzzle).unwrap();
        Utxo { amount, puzzle }
    }

    // pub fn process(&mut self, transaction: Transaction) {}
}
