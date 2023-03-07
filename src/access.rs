use anchor_lang::prelude::*;
use crate::error::*;

pub fn verify_minimumamount(f1: u64, f2: u64) -> Result<()> {
    if f1 < f2 {
        return err!(EscrowError::InvalidDeposit);
    }
    Ok(())
}

pub fn verify_owner(owner: Pubkey, signer: Pubkey) -> Result<()> {
    if owner != signer {
        return err!(EscrowError::InvalidSigner);
    }
    Ok(())
}

pub fn verify_wallet(a1: Pubkey, a2: Pubkey) -> Result<()> {
    if a1 != a2 {
        return err!(EscrowError::WrongAddress);
    }
    Ok(())
}

pub fn verify_unique_address(a1: Pubkey, a2: Pubkey) -> Result<()> {
    if a1 == a2 {
        return err!(EscrowError::SameBuyerSeller);
    }
    Ok(())
}

pub fn verify_buyer_seller(buyer: Option<Pubkey>, seller: Option<Pubkey>) -> Result<()> {
    if buyer == None || seller == None {
        return err!(EscrowError::InvalidAdress);
    }
    Ok(())
}
