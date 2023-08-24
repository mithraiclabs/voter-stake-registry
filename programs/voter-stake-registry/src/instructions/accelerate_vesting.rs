use crate::state::*;
use crate::error::*;
use anchor_lang::prelude::*;

mod acceleration_authority {
    use anchor_lang::declare_id;

    declare_id!("ENSuopuKKCDgdmT6dXHqJSjeDjUoLXUNikr33e21bNtp");
}

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
    #[account(
      address = acceleration_authority::ID @ VsrError::BadAccelerationAuthority
    )]
    pub accelerated_authority: Signer<'info>,
}

pub fn accelerate_vesting(
    _ctx: Context<AccelerateVesting>,
    _deposit_entry_index: u8,
) -> Result<()> {
    Ok(())
}
