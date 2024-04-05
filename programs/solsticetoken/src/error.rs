use anchor_lang::prelude::*;

#[error_code]
pub enum MyContractError {
    #[msg("The purchase would exceed the total tokens allocated for sale.")]
    OverPurchase,
}
