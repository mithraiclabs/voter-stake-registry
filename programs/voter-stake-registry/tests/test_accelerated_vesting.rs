use anchor_spl::token::TokenAccount;
use program_test::*;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};
use std::cell::RefCell;
use std::sync::Arc;
use voter_stake_registry::state::LockupKind;

mod program_test;

#[allow(unaligned_references)]
#[tokio::test]
async fn test_accelerated_vesting() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let addin = &context.addin;

    let payer = &context.users[0].key;
    let realm_authority = Keypair::new();
    let realm = context
        .governance
        .create_realm(
            "testrealm",
            realm_authority.pubkey(),
            &context.mints[0],
            &payer,
            &context.addin.program_id,
        )
        .await;

    let voter_authority = &context.users[1].key;
    let token_owner_record = realm
        .create_token_owner_record(voter_authority.pubkey(), &payer)
        .await;

    let registrar = addin
        .create_registrar(&realm, &realm_authority, payer)
        .await;
    let mngo_voting_mint = addin
        .configure_voting_mint(
            &registrar,
            &realm_authority,
            payer,
            0,
            &context.mints[0],
            0,
            1.0,
            0.0,
            5 * 365 * 24 * 60 * 60,
            None,
            None,
        )
        .await;

    let voter = addin
        .create_voter(&registrar, &token_owner_record, &voter_authority, &payer)
        .await;

    let reference_account = context.users[1].token_accounts[0];
    let withdraw = |index: u8, amount: u64| {
        addin.withdraw(
            &registrar,
            &voter,
            &mngo_voting_mint,
            &voter_authority,
            reference_account,
            index,
            amount,
        )
    };
    let deposit = |index: u8, amount: u64| {
        addin.deposit(
            &registrar,
            &voter,
            &mngo_voting_mint,
            &voter_authority,
            reference_account,
            index,
            amount,
        )
    };

    let accelerate_vesting = |index: u8| {
        addin.accelerate_vesting(
            &registrar,
            &voter,
            &voter_authority,
            &realm_authority,
            index,
        )
    };
    let time_offset = Arc::new(RefCell::new(0i64));
    let advance_time = |extra: u64| {
        *time_offset.borrow_mut() += extra as i64;
        addin.set_time_offset(&registrar, &realm_authority, *time_offset.borrow())
    };
    let lockup_status = |index: u8| {
        get_lockup_data_struct(&context.solana, voter.address, index, *time_offset.borrow())
    };

    let month = LockupKind::Monthly.period_secs();
    let day = 24 * 60 * 60;
    let hour = 60 * 60;

    // TODO: Test bad grant_authority
    let deposit_entry_index = 1;
    addin
        .create_deposit_entry(
            &registrar,
            &voter,
            &voter_authority,
            &mngo_voting_mint,
            deposit_entry_index,
            LockupKind::Daily,
            None,
            3,
            false,
        )
        .await
        .unwrap();
    deposit(deposit_entry_index, 80).await.unwrap();
    assert_eq!(
        lockup_status(deposit_entry_index).await,
        LockupData {
            time_passed: 0,
            duration: 3 * day,
            amount_initially_locked_native: 80,
            amount_deposited_native: 80,
            amount_unlocked: 0
        }
    );
    addin
        .accelerate_vesting(
            &registrar,
            &voter,
            &voter_authority,
            &voter_authority,
            deposit_entry_index,
        )
        .await
        .expect_err("BadAccelerationAuthority");

    // TODO: tests for daily vesting
    let deposit_entry_index = 2;
    addin
        .create_deposit_entry(
            &registrar,
            &voter,
            &voter_authority,
            &mngo_voting_mint,
            deposit_entry_index,
            LockupKind::Daily,
            None,
            3,
            false,
        )
        .await
        .unwrap();
    deposit(deposit_entry_index, 80).await.unwrap();
    assert_eq!(
        lockup_status(deposit_entry_index).await,
        LockupData {
            time_passed: 0,
            duration: 3 * day,
            amount_initially_locked_native: 80,
            amount_deposited_native: 80,
            amount_unlocked: 0
        }
    );
    deposit(deposit_entry_index, 10).await.unwrap();
    assert_eq!(
        lockup_status(deposit_entry_index).await,
        LockupData {
            time_passed: 0,
            duration: 3 * day,
            amount_initially_locked_native: 90,
            amount_deposited_native: 90,
            amount_unlocked: 0
        }
    );

    accelerate_vesting(deposit_entry_index).await?;
    assert_eq!(
        lockup_status(deposit_entry_index).await,
        LockupData {
            time_passed: 0,
            duration: 0,
            amount_initially_locked_native: 90,
            amount_deposited_native: 90,
            amount_unlocked: 90
        }
    );

    context.solana.advance_clock_by_slots(2).await; // avoid deposit and withdraw in one slot

    withdraw(deposit_entry_index, 90).await.unwrap(); // withdraw all previously locked tokens
    assert_eq!(
        lockup_status(deposit_entry_index).await,
        LockupData {
            time_passed: 0,
            duration: 0,
            amount_initially_locked_native: 90,
            amount_deposited_native: 0,
            amount_unlocked: 90
        }
    );
    // assert_eq!(lockup_status(7).await, (0, 10 * day, 70, 70, 0));

    // accelerate_vesting(7).await.unwrap();
    // assert_eq!(lockup_status(7).await, (0, 1 * month, 70, 70, 0));

    // accelerate_vesting(7)
    //     .await
    //     .expect_err("decreasing strictness");
    // accelerate_vesting(7)
    //     .await
    //     .expect_err("decreasing strictness");
    // accelerate_vesting(7).await.expect_err("period shortnend");
    // accelerate_vesting(7).await.unwrap();
    // assert_eq!(lockup_status(7).await, (0, 31 * day, 70, 70, 0));

    // TODO: tests for cliff vesting
    // addin
    //     .create_deposit_entry(
    //         &registrar,
    //         &voter,
    //         &voter_authority,
    //         &mngo_voting_mint,
    //         5,
    //         LockupKind::Cliff,
    //         None,
    //         3,
    //         false,
    //     )
    //     .await
    //     .unwrap();
    // deposit(5, 80).await.unwrap();
    // assert_eq!(lockup_status(5).await, (0, 3 * day, 80, 80, 0));
    // accelerate_vesting(5)
    //     .await
    //     .expect_err("can't relock for less periods");
    // accelerate_vesting(5).await.unwrap(); // just resets start to current timestamp
    // assert_eq!(lockup_status(5).await, (0, 3 * day, 80, 80, 0));
    // accelerate_vesting(5).await.unwrap();
    // assert_eq!(lockup_status(5).await, (0, 4 * day, 80, 80, 0));

    // // advance to end of cliff
    // advance_time(4 * day + hour).await;
    // context.solana.advance_clock_by_slots(2).await;

    // assert_eq!(lockup_status(5).await, (4 * day, 4 * day, 80, 80, 80));
    // accelerate_vesting(5).await.unwrap();
    // assert_eq!(lockup_status(5).await, (0, 1 * day, 80, 80, 0));
    // withdraw(5, 10).await.expect_err("nothing unlocked");

    // // advance to end of cliff again
    // advance_time(day + hour).await;
    // context.solana.advance_clock_by_slots(2).await;

    // assert_eq!(lockup_status(5).await, (day, day, 80, 80, 80));
    // withdraw(5, 10).await.unwrap();
    // assert_eq!(lockup_status(5).await, (day, day, 80, 70, 70));
    // deposit(5, 5).await.unwrap();
    // assert_eq!(lockup_status(5).await, (0, 0, 5, 75, 75));
    // accelerate_vesting(5).await.unwrap();
    // assert_eq!(lockup_status(5).await, (0, 1 * day, 75, 75, 0));
    // deposit(5, 15).await.unwrap();
    // assert_eq!(lockup_status(5).await, (0, 1 * day, 90, 90, 0));

    // TODO: Test failure if hardcoded governance authority has not signed the transaciton

    Ok(())
}
