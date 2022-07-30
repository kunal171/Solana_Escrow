#[cfg(feature = "test-bpf")]
use escrow_buy::{processor, state::Escrow};
#[cfg(feature = "test-bpf")]
use solana_program::{instruction::{AccountMeta, Instruction}, program_pack::Pack, pubkey::Pubkey, rent::Rent, sysvar};
#[cfg(feature = "test-bpf")]
use solana_program_test::{ProgramTest, processor};
#[cfg(feature = "test-bpf")]
use solana_sdk::{account::Account, signature::{Keypair, Signer}, transaction::Transaction};
use std::str::FromStr;


#[tokio::test]
#[cfg(feature = "test-bpf")]
async fn process_escrow_init_success() {
    let mut data = [0u8; 9];
    hex::decode_to_slice("007b00000000000000", &mut data as &mut [u8]).unwrap();

    let amount: u64 = 123;
    let escrow_program_id = Pubkey::from_str(
        &"escrowprogram111111111111111111111111111111"
    ).unwrap();

    // let sellers token
    let sellers_token = Keypair::new();

    // minters key
    let minter = Keypair::new();

    // required accounts
    let initers_key = Keypair::new();
    let temp_seller_token_account = Keypair::new();
    let mint_key = Keypair::new();
    let escrow_account = Keypair::new();

    let mut program_test = ProgramTest::new(
        "escrow_buy",
        escrow_program_id,
        processor!(processor::Processor::process)
    );

    // temp token account for the sell token
    program_test.add_account(
        temp_seller_token_account.pubkey(),
        Account {
            lamports: Rent::default().minimum_balance(
                spl_token::state::Account::LEN
            ),
            owner: spl_token::id(),
            data: vec![0; spl_token::state::Account::LEN],
            ..Account::default()
        }
    );

    // initers buy token which will be recieved 
    // in an exchange to the sell token
    program_test.add_account(
        mint_key.pubkey(),
        Account {
            lamports: 123,
            owner: spl_token::id(),
            ..Account::default()
        }
    );

    // escrow account
    program_test.add_account(
        escrow_account.pubkey(), 
        Account {
            lamports: Rent::default().minimum_balance(Escrow::get_packed_len()),
            owner: escrow_program_id,
            data: vec![0; Escrow::get_packed_len()],
            ..Account::default()
        },
    );
    
    // TEST
    // this is a mint token account, this mint will
    // be stored in the temp sell token account
    program_test.add_account(
        sellers_token.pubkey(), 
        Account {
            lamports: Rent::default().minimum_balance(spl_token::state::Mint::LEN),
            owner: spl_token::id(), 
            data: vec![0; spl_token::state::Mint::LEN],
            ..Account::default()
        },
    );

    let (mut client, payer, recent_blockhash) = program_test.start().await;
    println!("Creating a mint and minting it to temp token account...");

    let mut transaction = Transaction::new_with_payer(
        &[
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &sellers_token.pubkey(),
                &minter.pubkey(),
                None,
                0
            ).unwrap(),

            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &temp_seller_token_account.pubkey(),
                &sellers_token.pubkey(),
                &initers_key.pubkey()
            ).unwrap(),

            spl_token::instruction::mint_to(
                &spl_token::id(),
                &sellers_token.pubkey(),
                &temp_seller_token_account.pubkey(),
                &minter.pubkey(),
                &[],
                amount
            ).unwrap()
        ],
        Some(&payer.pubkey())
    );
    transaction.sign(&[&payer, &minter], recent_blockhash);
    client.process_transaction(transaction).await.unwrap();
    println!("Done...");

    let mut transaction = Transaction::new_with_payer(
        &[
            Instruction::new_with_bytes(
                escrow_program_id,
                &data,
                vec![
                    AccountMeta::new(initers_key.pubkey(), true),
                    AccountMeta::new(temp_seller_token_account.pubkey(), false),
                    AccountMeta::new(mint_key.pubkey(), false),
                    AccountMeta::new(escrow_account.pubkey(), false),
                    AccountMeta::new(sysvar::rent::id(), false),
                    AccountMeta::new(spl_token::id(), false)
                ],
            )
        ],
        Some(&payer.pubkey())
    );
    transaction.sign(&[&payer, &initers_key], recent_blockhash);
    client.process_transaction(transaction).await.unwrap();

    // assert tests after the process_escrow_init has been completed
    // escrow
    let escrow_account = client.get_account(
        escrow_account.pubkey()
    ).await
    .expect("Unable to find escrow account")
    .expect("Unable to find escrow account");

    // assert tests after the process_escrow_init has been completed
    // temp token account
    let temp_token_account = client.get_account(
        temp_seller_token_account.pubkey()
    ).await
    .expect("Unable to find escrow account")
    .expect("Unable to find escrow account");

    let escrow_account_state = Escrow::unpack(
        &escrow_account.data
    ).unwrap();

    let token_account_state = spl_token::state::Account::unpack(
        &temp_token_account.data
    ).unwrap();

    // check if temp token account owner is now the pda
    // and not the initializer
    assert_ne!(token_account_state.owner, initers_key.pubkey());

    // assert checks for escrow account data
    assert_eq!(
        escrow_account_state.is_initialized, 
        true
    );
    assert_eq!(
        escrow_account_state.seller_pubkey, 
        initers_key.pubkey()
    );
    assert_eq!(
        escrow_account_state.token_account_pubkey, 
        temp_seller_token_account.pubkey()
    );
    assert_eq!(
        escrow_account_state.mint_key, 
        mint_key.pubkey()
    );
    assert_eq!(
        escrow_account_state.expected_amount,
        amount
    );
}