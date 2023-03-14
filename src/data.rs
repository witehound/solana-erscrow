use anchor_lang::prelude::*;

use crate::constants::*;

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
