use crate::account::*;
use crate::error::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
use spl_governance::state::realm;
use std::mem::size_of;

#[derive(Accounts)]
#[instruction(vote_weight_decimals: u8, registrar_bump: u8)]
pub struct CreateRegistrar<'info> {
    /// a voting registrar. There can only be a single registrar
    /// per governance realm and governing mint.
    #[account(
        init,
        seeds = [realm.key().as_ref(), b"registrar".as_ref(), realm_governing_token_mint.key().as_ref()],
        bump = registrar_bump,
        payer = payer,
        space = 8 + size_of::<Registrar>()
    )]
    pub registrar: Box<Account<'info, Registrar>>,

    // realm is validated in the instruction:
    // - realm is owned by the governance_program_id
    // - realm_governing_token_mint must be the community or council mint
    // - realm_authority is realm.authority
    pub realm: UncheckedAccount<'info>,

    pub governance_program_id: UncheckedAccount<'info>,
    pub realm_governing_token_mint: Account<'info, Mint>,
    pub realm_authority: Signer<'info>,

    pub clawback_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// Creates a new voting registrar. There can only be a single registrar
/// per governance realm.
pub fn create_registrar(
    ctx: Context<CreateRegistrar>,
    vote_weight_decimals: u8,
    registrar_bump: u8,
) -> Result<()> {
    msg!("--------create_registrar--------");
    let registrar = &mut ctx.accounts.registrar;
    registrar.bump = registrar_bump;
    registrar.governance_program_id = ctx.accounts.governance_program_id.key();
    registrar.realm = ctx.accounts.realm.key();
    registrar.realm_governing_token_mint = ctx.accounts.realm_governing_token_mint.key();
    registrar.realm_authority = ctx.accounts.realm_authority.key();
    registrar.clawback_authority = ctx.accounts.clawback_authority.key();
    registrar.vote_weight_decimals = vote_weight_decimals;
    registrar.time_offset = 0;

    // Verify that "realm_authority" is the expected authority on "realm"
    // and that the mint matches one of the realm mints too.
    let realm = realm::get_realm_data_for_governing_token_mint(
        &registrar.governance_program_id,
        &ctx.accounts.realm.to_account_info(),
        &registrar.realm_governing_token_mint,
    )?;
    require!(
        realm.authority.unwrap() == ctx.accounts.realm_authority.key(),
        ErrorCode::InvalidRealmAuthority
    );

    Ok(())
}