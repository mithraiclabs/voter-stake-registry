use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AccelerateVesting<'info> {
    pub registrar: AccountLoader<'info, Registrar>,
    // checking the PDA address it just an extra precaution,
    // the other constraints must be exhaustive
    #[account(
      mut,
      seeds = [registrar.key().as_ref(), b"voter".as_ref(), voter_authority.key().as_ref()],
      bump = voter.load()?.voter_bump,
      has_one = voter_authority,
      has_one = registrar)]
    pub voter: AccountLoader<'info, Voter>,
    pub voter_authority: Signer<'info>,
    /// Authority for making a grant to this voter account
    ///
    /// Instruction validates grant_authority is the VotingMintConfig.grant_authority or
    /// Registrar.realm_authority.
    pub grant_authority: Signer<'info>,
}

pub fn accelerate_vesting(ctx: Context<AccelerateVesting>, deposit_entry_index: u8) -> Result<()> {
    // Load accounts.
    let registrar = &ctx.accounts.registrar.load()?;
    let voter = &mut ctx.accounts.voter.load_mut()?;

    let deposit_entry = voter.active_deposit_mut(deposit_entry_index)?;
    // Get the grant_authority for the DepositEntry
    let mint_idx = deposit_entry.voting_mint_config_idx;
    let mint_config: &VotingMintConfig = &registrar.voting_mints[mint_idx as usize];
    let grant_authority = ctx.accounts.grant_authority.key();

    // Validate grant_authority is appropriate to accelerate vesting
    require!(
        grant_authority == registrar.realm_authority
            || grant_authority == mint_config.grant_authority,
        VsrError::BadAccelerationAuthority
    );

    Ok(())
}
