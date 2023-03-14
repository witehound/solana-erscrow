use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, system_instruction::transfer},
};

mod access;
mod constants;
mod data;
mod error;
mod helper;

use crate::{access::*, constants::*, data::*, error::*, helper::*};

declare_id!("8pZkBXTmvLdudXm5R7JmtihPJ3C5anAd6N9EvGZzBJ3n");

#[program]
mod hello_anchor {
    use super::*;

    pub fn init_factory(_ctx: Context<InitFactory>) -> Result<()> {
        Ok(())
    }

    pub fn new_escrow_id(ctx: Context<NewUid>, uuid: String) -> Result<()> {
        let f = &mut ctx.accounts.factory;
        let nu = &mut ctx.accounts.escrowid;

        f.last_id += 1;
        nu.uuid = uuid;
        nu.id = f.last_id;

        Ok(())
    }

    pub fn initialize_deal(ctx: Context<InitializeEscrow>) -> Result<()> {
        let es = &mut ctx.accounts.escrow;
        let s = &ctx.accounts.payer;
        let nu = &ctx.accounts.escrowid;

        es.state = State::INIT;
        es.owner = s.key();
        es.commissionrate = COMMISION_RATE;
        es.minimumescrow_amount = MIN_AMOUNT;
        es.commissionwallet = COMMISSION_PUBKEY;
        es.id = nu.uuid.clone();

        Ok(())
    }

    pub fn deposit(ctx: Context<EscrowParty>, deposit: u64) -> Result<()> {
        let es = &mut ctx.accounts.escrow;
        let s = &ctx.accounts.signer;

        if es.state != State::INIT {
            return err!(EscrowError::InvalidEscrowState);
        }

        if s.key() != es.owner {
            return err!(EscrowError::SameBuyerSeller);
        }

        verify_minimumamount(deposit, es.minimumescrow_amount)?;

        invoke(
            &transfer(&s.key(), &es.key(), deposit),
            &[
                s.to_account_info(),
                es.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        es.state = State::FUNDED;
        es.deposit_time = Clock::get().unwrap().unix_timestamp;
        es.amount_in_escrow = deposit;

        Ok(())
    }

    pub fn accept_deal(ctx: Context<EscrowParty>) -> Result<()> {
        let es = &mut ctx.accounts.escrow;
        let s = &ctx.accounts.signer;

        if es.state != State::FUNDED {
            return err!(EscrowError::InvalidEscrowState);
        }

        verify_unique_address(es.owner, s.key())?;

        es.seller = Some(s.key());

        verify_seller(es.seller)?;

        es.state = State::ACCEPTED;

        Ok(())
    }

    pub fn release_fund(ctx: Context<RealeseFund>) -> Result<()> {
        let es = &mut ctx.accounts.escrow;
        let s = &ctx.accounts.signer;
        let r = &ctx.accounts.reciever;
        let cs = &ctx.accounts.commision_account;
        let mut tf_state = false;

        if es.state != State::ACCEPTED {
            return err!(EscrowError::InvalidEscrowState);
        }

        verify_wallet(es.commissionwallet, cs.key())?;

        let sk = es.seller.unwrap();

        if es.owner != s.key() && sk != s.key() {
            return err!(EscrowError::WrongAddress);
        }

        let (signer_amount, commision_amount) =
            calculate_amount_totransfer(es.amount_in_escrow, es.commissionrate);

        if es.owner == s.key() && sk == r.key() {
            **es.to_account_info().try_borrow_mut_lamports()? -= signer_amount;
            **r.to_account_info().try_borrow_mut_lamports()? += signer_amount;
            tf_state = true;
        }

        if sk == s.key() && es.owner == r.key() {
            **es.to_account_info().try_borrow_mut_lamports()? -= signer_amount;
            **r.to_account_info().try_borrow_mut_lamports()? += signer_amount;
            tf_state = true;
        }

        if !tf_state {
            return err!(EscrowError::InvalidEscrowState);
        }

        **es.to_account_info().try_borrow_mut_lamports()? -= commision_amount;
        **cs.to_account_info().try_borrow_mut_lamports()? += commision_amount;

        es.amount_in_escrow = 0;
        es.state = State::RELEASED;
        es.commission_amount = commision_amount;
        es.released_amount = signer_amount;
        es.realesed_by = Some(s.key());

        Ok(())
    }

    pub fn withdraw_fund(ctx: Context<WithdrawFund>) -> Result<()> {
        let es = &mut ctx.accounts.escrow;
        let s = &ctx.accounts.signer;
        let cs = &ctx.accounts.commision_account;

        verify_owner(es.owner, s.key())?;

        if es.state != State::FUNDED {
            return err!(EscrowError::InvalidEscrowState);
        }

        let (signer_amount, commision_amount) =
            calculate_amount_totransfer(es.amount_in_escrow, es.commissionrate);

        **es.to_account_info().try_borrow_mut_lamports()? -= signer_amount;
        **s.to_account_info().try_borrow_mut_lamports()? += signer_amount;

        **es.to_account_info().try_borrow_mut_lamports()? -= commision_amount;
        **cs.to_account_info().try_borrow_mut_lamports()? += commision_amount;

        es.amount_in_escrow = 0;
        es.state = State::REFUNDED;
        es.commission_amount = commision_amount;
        es.released_amount = signer_amount;
        es.realesed_by = Some(s.key());

        Ok(())
    }

    pub fn post_six_months(ctx: Context<EscrowParty>) -> Result<()> {
        let es = &mut ctx.accounts.escrow;
        let s = &ctx.accounts.signer;

        if es.owner != s.key() {
            return err!(EscrowError::InvalidSigner);
        }

        let timer = Clock::get().unwrap().unix_timestamp - es.deposit_time;

        if timer < SIX_MONTHS {
            return err!(EscrowError::InvalidSigner);
        }

        let tf_amount = es.amount_in_escrow;

        **es.to_account_info().try_borrow_mut_lamports()? -= tf_amount;
        **s.to_account_info().try_borrow_mut_lamports()? += tf_amount;

        es.amount_in_escrow = 0;
        es.state = State::WITHDRAWN;

        Ok(())
    }
}
