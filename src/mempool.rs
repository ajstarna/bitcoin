use serde::{Serialize, Deserialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use crate::transaction::Transaction;

pub type Mempool = BinaryHeap<TransactionWithTip>;

/// this struct holds the tip for the miner (the difference between the inputs and the outputs),
/// so that we can easily store this in the mem pool in a sorted order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionWithTip {
    pub miner_tip: u32,
    pub transaction: Transaction,
}

impl TransactionWithTip {
    pub fn new(transaction: Transaction, miner_tip: u32) -> Self {
        Self {transaction, miner_tip}
    }
}

// The priority queue depends on `Ord`.
impl Ord for TransactionWithTip {
    /// we compare based strictly on the miner's tip
    fn cmp(&self, other: &Self) -> Ordering {
        self.miner_tip.cmp(&other.miner_tip)
    }
}

impl PartialOrd for TransactionWithTip {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TransactionWithTip {
    fn eq(&self, other: &Self) -> bool {
        self.miner_tip == other.miner_tip
    }
}

impl Eq for TransactionWithTip {}
