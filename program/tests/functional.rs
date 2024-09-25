#![cfg(feature = "test-bpf")]
use std::str::FromStr;

use solana_program::{hash::Hash, pubkey::Pubkey, rent::Rent, system_program, sysvar};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{account::Account, signature::Keypair, signature::Signer, system_instruction,transaction::Transaction};
use solana_test_framework::ProgramTestContextExtension;
use spl_token::{self,instruction::{initialize_account, initialize_mint, mint_to}};
use token_vesting::instruction::{create, init, initialize_unlock, unlock};
use token_vesting::{entrypoint::process_instruction, instruction::Schedule};

#[tokio::test]
async fn test_token_vesting() {
    // Create program and test environment
    let program_id = Pubkey::from_str("VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA").unwrap();
    let mint_authority = Keypair::new();
    let mint = Keypair::new();

    let source_account = Keypair::new();
    let source_token_account = Keypair::new();

    let mut seeds = [42u8; 32];
    let (vesting_account_key, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);
    seeds[31] = bump;
    let vesting_token_account = Keypair::new();

    let mut program_test =
        ProgramTest::new("token_vesting", program_id, processor!(process_instruction));

    // Add accounts
    program_test.add_account(
        source_account.pubkey(),
        Account {
            lamports: 5000000,
            ..Account::default()
        },
    );

    // Start and process transactions on the test network
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Initialize the vesting program account
    let init_instruction = [init(
        &system_program::id(),
        &sysvar::rent::id(),
        &program_id,
        &payer.pubkey(),
        &vesting_account_key,
        seeds,
    )
    .unwrap()];
    let mut init_transaction = Transaction::new_with_payer(
        &init_instruction, 
        Some(&payer.pubkey()),
    );
    init_transaction.partial_sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(init_transaction).await.unwrap();

    // Initialize the token accounts
    banks_client.process_transaction(mint_init_transaction(
            &payer,
            &mint,
            &mint_authority,
            recent_blockhash,
        ))
        .await.unwrap();
    
    banks_client.process_transaction(create_token_account(
            &payer,
            &mint,
            recent_blockhash,
            &source_token_account,
            &source_account.pubkey(),
        )).await.unwrap();
    banks_client.process_transaction(
            create_token_account(
            &payer,
            &mint,
            recent_blockhash,
            &vesting_token_account,
            &vesting_account_key,
        )).await.unwrap();

    // Create and process the vesting transactions
    let setup_instructions = [mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        &source_token_account.pubkey(),
        &mint_authority.pubkey(),
        &[],
        100,
    )
    .unwrap()];

    let schedule = Schedule {
        amount: 100,
        time_delta: 1,
    };

    let test_instructions = [
        create(
            &program_id,
            &spl_token::id(),
            &vesting_account_key,
            &vesting_token_account.pubkey(),
            &source_account.pubkey(),
            &source_token_account.pubkey(),
            &mint.pubkey(),
            schedule,
            seeds.clone(),
        )
        .unwrap(),
        unlock(
            &program_id,
            &spl_token::id(),
            &sysvar::clock::id(),
            &vesting_account_key,
            &vesting_token_account.pubkey(),
            &source_token_account.pubkey(),
            seeds.clone(),
        )
        .unwrap(),
    ];

    // Process transaction on test network
    let mut setup_transaction =
        Transaction::new_with_payer(&setup_instructions, Some(&payer.pubkey()));
    setup_transaction.partial_sign(&[&payer, &mint_authority], recent_blockhash);

    banks_client.process_transaction(setup_transaction).await.unwrap();

    // Process transaction on test network
    let mut test_transaction =
        Transaction::new_with_payer(&test_instructions, Some(&payer.pubkey()));
    test_transaction.partial_sign(&[&payer, &source_account], recent_blockhash);

    // NOTE: we're NOT doing `now() + time_delta` in the program (but we should), that's why this test passes
    // TODO: add warp_to_timestamp to correctly test the behaviour
    banks_client
        .process_transaction(test_transaction)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_token_unlocking() {
    // Create program and test environment
    let program_id = Pubkey::from_str("VestingbGKPFXCWuBvfkegQfZyiNwAJb9Ss623VQ5DA").unwrap();
    let mint_authority = Keypair::new();
    let mint = Keypair::new();

    let source_account = Keypair::new();
    let source_token_account = Keypair::new();

    let mut seeds = [42u8; 32];
    let (vesting_account_key, bump) = Pubkey::find_program_address(&[&seeds[..31]], &program_id);
    seeds[31] = bump;
    let vesting_token_account = Keypair::new();

    let mut program_test =
        ProgramTest::new("token_vesting", program_id, processor!(process_instruction));

    // Add accounts
    program_test.add_account(
        source_account.pubkey(),
        Account {
            lamports: 5000000,
            ..Account::default()
        },
    );

    // Start and process transactions on the test network
    let mut context: ProgramTestContext = program_test.start_with_context().await;

    // NOTE: using scopes to allow partial borrows when context.warp_to_timestamp is called
    {
        let banks_client = &mut context.banks_client;
        let payer = &mut context.payer;
        let recent_blockhash = context.last_blockhash;

        // Initialize the vesting program account
        let init_instruction = [init(
            &system_program::id(),
            &sysvar::rent::id(),
            &program_id,
            &payer.pubkey(),
            &vesting_account_key,
            seeds,
        )
        .unwrap()];
        let mut init_transaction = Transaction::new_with_payer(
            &init_instruction, 
            Some(&payer.pubkey()),
        );
        init_transaction.partial_sign(&[&payer], recent_blockhash);
        banks_client
            .process_transaction(init_transaction)
            .await
            .unwrap();

        // Initialize the token accounts
        banks_client
            .process_transaction(mint_init_transaction(
                &payer,
                &mint,
                &mint_authority,
                recent_blockhash,
            ))
            .await
            .unwrap();

        banks_client.process_transaction(
            create_token_account(
                &payer,
                &mint,
                recent_blockhash,
                &source_token_account,
                &source_account.pubkey(),
            ))
            .await
            .unwrap();
        banks_client.process_transaction(
            create_token_account(
                &payer,
                &mint,
                recent_blockhash,
                &vesting_token_account,
                &vesting_account_key,
            ))
            .await
            .unwrap();

        // Create and process the vesting transactions
        let setup_instructions = [mint_to(
            &spl_token::id(),
            &mint.pubkey(),
            &source_token_account.pubkey(),
            &mint_authority.pubkey(),
            &[],
            100,
        )
        .unwrap()];

        let schedule = Schedule {
            amount: 100,
            time_delta: 0,
        };

        let test_instructions = [
            create(
                &program_id,
                &spl_token::id(),
                &vesting_account_key,
                &vesting_token_account.pubkey(),
                &source_account.pubkey(),
                &source_token_account.pubkey(),
                &mint.pubkey(),
                schedule,
                seeds.clone(),
            )
            .unwrap(),
            initialize_unlock(
                &program_id,
                &spl_token::id(),
                &sysvar::clock::id(),
                &vesting_account_key,
                &vesting_token_account.pubkey(),
                &source_token_account.pubkey(),
                seeds.clone(),
            )
            .unwrap(),
            // TODO: move unlock after warp_to_timestamp
            unlock(
                &program_id,
                &spl_token::id(),
                &sysvar::clock::id(),
                &vesting_account_key,
                &vesting_token_account.pubkey(),
                &source_token_account.pubkey(),
                seeds.clone(),
            )
            .unwrap()
        ];

        // Process transaction on test network
        let mut setup_transaction =
            Transaction::new_with_payer(&setup_instructions, Some(&payer.pubkey()));
        setup_transaction.partial_sign(&[&payer, &mint_authority], recent_blockhash);

        banks_client
            .process_transaction(setup_transaction)
            .await
            .unwrap();

        // Process transaction on test network
        let mut test_transaction =
            Transaction::new_with_payer(&test_instructions, Some(&payer.pubkey()));
        test_transaction.partial_sign(&[&payer, &source_account], recent_blockhash);

        banks_client
            .process_transaction(test_transaction)
            .await
            .unwrap();
    }

    // Warp to a future slot
    // TODO: this should increase the timestamp, and it does not.
    let _ = context.warp_to_timestamp(99999999).await;

    // {
    //     let banks_client = &mut context.banks_client;
    //     let payer = &mut context.payer;
    //     let recent_blockhash = context.last_blockhash;

    //     let unlock_instructions = [unlock(
    //         &program_id,
    //         &spl_token::id(),
    //         &sysvar::clock::id(),
    //         &vesting_account_key,
    //         &vesting_token_account.pubkey(),
    //         &source_token_account.pubkey(),
    //         seeds.clone(),
    //     )
    //     .unwrap()];

    //     let mut unlock_transaction =
    //         Transaction::new_with_payer(&unlock_instructions, Some(&payer.pubkey()));

    //     unlock_transaction.partial_sign(&[&payer], recent_blockhash);

    //     banks_client
    //         .process_transaction(unlock_transaction)
    //         .await
    //         .unwrap();
    // }
}

fn mint_init_transaction(
    payer: &Keypair,
    mint: &Keypair,
    mint_authority: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = [
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            Rent::default().minimum_balance(82),
            82,
            &spl_token::id(),
        ),
        initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            0,
        )
        .unwrap(),
    ];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, mint], recent_blockhash);
    transaction
}

fn create_token_account(
    payer: &Keypair,
    mint: &Keypair,
    recent_blockhash: Hash,
    token_account: &Keypair,
    token_account_owner: &Pubkey,
) -> Transaction {
    let instructions = [
        system_instruction::create_account(
            &payer.pubkey(),
            &token_account.pubkey(),
            Rent::default().minimum_balance(165),
            165,
            &spl_token::id(),
        ),
        initialize_account(
            &spl_token::id(),
            &token_account.pubkey(),
            &mint.pubkey(),
            token_account_owner,
        )
        .unwrap(),
    ];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, token_account], recent_blockhash);
    transaction
}
