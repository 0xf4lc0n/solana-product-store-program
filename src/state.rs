use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProductAccountState {
    pub discriminator: String,
    pub is_initialized: bool,
    pub seller: Pubkey,
    pub id: u64,
    pub name: String,
    pub price: f64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProductPriceCounter {
    pub discriminator: String,
    pub is_initialized: bool,
    pub counter: u64,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ProductPrice {
    pub discriminator: String,
    pub is_initialized: bool,
    pub product: Pubkey,
    pub price: f64,
}

impl Sealed for ProductAccountState {}

impl IsInitialized for ProductAccountState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for ProductPriceCounter {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for ProductPrice {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl ProductAccountState {
    pub const DISCRIMINATOR: &'static str = "product";

    pub fn get_account_size(name: String) -> usize {
        return (4 + Self::DISCRIMINATOR.len()) + 1 + (4 + name.len());
    }
}

impl ProductPriceCounter {
    pub const DISCRIMINATOR: &'static str = "counter";

    pub const SIZE: usize = (4 + Self::DISCRIMINATOR.len()) + 1 + 8;
}

impl ProductPrice {
    pub const DISCRIMINATOR: &'static str = "price";

    pub const SIZE: usize = (4 + Self::DISCRIMINATOR.len() + 1 + 32 + 8);
}
