use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, pubkey, pubkey::Pubkey, system_instruction::transfer},
};

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

pub fn calculate_amount_totransfer(deal_amount: u64, commision_rate: u64) -> (u64, u64) {
    let commition_amount = deal_amount.checked_mul(commision_rate.into()).unwrap() / 100;
    let reciever_amount = deal_amount - commition_amount;

    return (reciever_amount, commition_amount);
}

pub const FACTORY_SEED: &str = "factoryinitone";
pub const ESCROW_SEED: &str = "escrowinitone";
pub const SIX_MONTHS: i64 = 300;
pub const MIN_AMOUNT: u64 = 1000000000;
pub const COMMISION_RATE: u64 = 5;
pub const COMMISSION_PUBKEY: Pubkey = pubkey!("BpvinfQbUZ7HbxnLvFYGvWG1hgqHUL6gQP5REKi5LcJi");

#[derive(Accounts)]
pub struct InitFactory<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 8,
        seeds = [FACTORY_SEED.as_bytes()],
        bump,
    )]
    pub factory: Account<'info, Factory>,
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Factory {
    pub last_id: u64,
}

#[account]
pub struct EscrowId {
    pub uuid: String,
    pub id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum State {
    INIT,
    FUNDED,
    ACCEPTED,
    RELEASED,
    REFUNDED,
    WITHDRAWN,
}

#[account]
pub struct Escrow {
    pub owner: Pubkey,
    pub seller: Option<Pubkey>,
    pub realesed_by: Option<Pubkey>,
    pub commissionwallet: Pubkey,
    pub minimumescrow_amount: u64,
    pub commissionrate: u64,
    pub state: State,
    pub deposit_time: i64,
    pub amount_in_escrow: u64,
    pub id: String,
    pub commission_amount: u64,
    pub released_amount: u64,
}

#[derive(Accounts)]
pub struct RealeseFund<'info> {
    #[account(
        mut,
        seeds = [ escrow.id.as_bytes()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut)]
    pub reciever: AccountInfo<'info>,
    #[account(mut)]
    pub commision_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct NewUid<'info> {
    #[account(
        init,
        payer = signer,
        space = 64 + 8,
        seeds = [ESCROW_SEED.as_bytes(), &(factory.last_id + 1).to_le_bytes()],
        bump,
    )]
    pub escrowid: Account<'info, EscrowId>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [FACTORY_SEED.as_bytes()],
        bump,
    )]
    pub factory: Account<'info, Factory>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFund<'info> {
    #[account(
        mut,
        seeds = [escrow.id.as_bytes()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub commision_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EscrowParty<'info> {
    #[account(
        mut,
        seeds = [ escrow.id.as_bytes()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EscrowParties<'info> {
    #[account(
        mut,
        seeds = [ escrow.id.as_bytes()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeEscrow<'info> {
    #[account(
        init,
        payer = payer,
        space = 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8  + 8 + 64 + 8 + 8 +  8,
        seeds = [escrowid.uuid.as_bytes()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
     
        mut,
        seeds = [ESCROW_SEED.as_bytes(), &escrowid.id.to_le_bytes()],
        bump,
    )]
    pub escrowid: Account<'info, EscrowId>,

    pub system_program: Program<'info, System>,
}

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

pub fn verify_seller( seller: Option<Pubkey>) -> Result<()> {
    if seller == None {
        return err!(EscrowError::InvalidAdress);
    }
    Ok(())
}