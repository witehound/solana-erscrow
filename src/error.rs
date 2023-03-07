use anchor_lang::prelude::error_code;

#[error_code]
pub enum EscrowError {
    #[msg("Inavlid time range")]
    InvalidTimeRange,
    #[msg("Inavlid time commissionrate")]
    InvalidCommisionRate,
    #[msg("Inavlid time minimumescrow_amount")]
    InvalidEscrowAmount,
    #[msg("Signer address is not valid")]
    InvalidSigner,
    #[msg("Escrow has invalid state")]
    InvalidEscrowState,
    #[msg("Buyer and seller cant be the same public key")]
    SameBuyerSeller,
    #[msg("Buyer public key error")]
    NoValidBuyer,
    #[msg("Seller public key error")]
    NoValidSeller,
    #[msg("Deposit does not meet minimum required amount")]
    InvalidDeposit,
    #[msg("Invalid signer address")]
    InvalidAdress,
    #[msg("Invalid address")]
    WrongAddress,
}
