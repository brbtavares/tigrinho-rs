use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Symbol {
    A,
    B,
    C,
    D,
    Wild,
}

impl Symbol {
    pub fn from_index(i: u8) -> Self {
        match i % 5 {
            0 => Symbol::A,
            1 => Symbol::B,
            2 => Symbol::C,
            3 => Symbol::D,
            _ => Symbol::Wild,
        }
    }

    pub fn to_index(self) -> u8 {
        match self {
            Symbol::A => 0,
            Symbol::B => 1,
            Symbol::C => 2,
            Symbol::D => 3,
            Symbol::Wild => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReelsConfig {
    pub reels: Vec<Vec<Symbol>>, // each reel strips
    pub rows: usize,             // visible rows
}

impl ReelsConfig {
    pub fn default_3x3() -> Self {
        let reel = vec![
            Symbol::A,
            Symbol::B,
            Symbol::C,
            Symbol::D,
            Symbol::Wild,
            Symbol::A,
            Symbol::B,
            Symbol::C,
            Symbol::D,
        ];
        Self {
            reels: vec![reel.clone(), reel.clone(), reel],
            rows: 3,
        }
    }
}
