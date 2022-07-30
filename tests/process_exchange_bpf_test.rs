#[cfg(feature = "test-bpf")]
use escrow_buy::{processor, state::Escrow};
#[cfg(feature = "test-bpf")]
use solana_program::{instruction::{AccountMeta, Instruction}, program_pack::Pack, pubkey::Pubkey, rent::Rent, sysvar, system_program};
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

    // // let sellers token
    // let sellers_token = Keypair::new();

    // // minters key
    // let minter = Keypair::new();

    // required accounts
    let initers_key = Keypair::new();
    let temp_seller_token_account = Keypair::new();
    let mint_key = Keypair::new();
    let mint_key_pubkey = mint_key.pubkey();
    let escrow_account = Keypair::new();
    // let metadata_account = Keypair::new();
    let taker_keypair = Keypair::new();
    let taker_to_recieve_keypair = Keypair::new();
    let creators_pubkey = Pubkey::from_str(
        &"metadatacreatorL5LYvXwxBNSaVkinzjzvTt1j3XsQ"
    ).unwrap();
    let valhalla_pubkey = Pubkey::from_str(
        &"paXi61MzXmioYZL5LYvXwxBNSaVkinzjzvTt1j3XsQz"
    ).unwrap();

    let (pda, _bump_seed) = Pubkey::find_program_address(
        &[b"escrow"], 
        &escrow_program_id
    );

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

    const PREFIX: &str = "metadata";
    // This is the program_id of the token_metadata program
    let metadata_program_id =
        Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();

    let metadata_seeds = &[
        PREFIX.as_bytes(),
        metadata_program_id.as_ref(),
        mint_key_pubkey.as_ref(),
    ];

    let (metadata_key, _metadata_bump_seed) =
        Pubkey::find_program_address(metadata_seeds, &metadata_program_id);

    // token metadata account
    program_test.add_account(
        metadata_key,
        Account {
            lamports: 5616720,
            owner: spl_token::id(),
            data: vec![
                4, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 
                32, 119, 111, 114, 108, 100, 33, 13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 
                119, 111, 114, 108, 100, 33, 13, 0, 0, 0, 72, 101, 108, 108, 111, 44, 32, 119, 
                111, 114, 108, 100, 33, 160, 15, 1, 1, 0, 0, 0, 11, 112, 101, 173, 83, 76, 32, 
                208, 98, 85, 112, 157, 31, 154, 198, 47, 233, 236, 44, 180, 235, 183, 174, 111, 
                61, 22, 187, 135, 21, 148, 93, 83, 1, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            ..Account::default()
        }
    );

    program_test.add_account(
        initers_key.pubkey(),
        Account {
            lamports: 5616720,
            ..Account::default()
        }
    );

    program_test.add_account(
        taker_keypair.pubkey(),
        Account {
            lamports: 5616720,
            ..Account::default()
        }
    );

    // temp token account for token reciever
    program_test.add_account(
        taker_to_recieve_keypair.pubkey(),
        Account {
            lamports: Rent::default().minimum_balance(
                spl_token::state::Account::LEN
            ),
            owner: spl_token::id(),
            data: vec![0; spl_token::state::Account::LEN],
            ..Account::default()
        }
    );
    
    // TEST
    // this is a mint token account, this mint will
    // be stored in the temp sell token account
    program_test.add_account(
        mint_key.pubkey(), 
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
                &mint_key.pubkey(),
                &initers_key.pubkey(),
                None,
                0
            ).unwrap(),

            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &temp_seller_token_account.pubkey(),
                &mint_key.pubkey(),
                &initers_key.pubkey()
            ).unwrap(),

            spl_token::instruction::mint_to(
                &spl_token::id(),
                &mint_key.pubkey(),
                &temp_seller_token_account.pubkey(),
                &initers_key.pubkey(),
                &[],
                amount
            ).unwrap(),

            // initialize token account for token reciever
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &taker_to_recieve_keypair.pubkey(),
                &mint_key.pubkey(),
                &taker_keypair.pubkey()
            ).unwrap(),
        ],
        Some(&payer.pubkey())
    );
    transaction.sign(&[&payer, &initers_key], recent_blockhash);
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

    let temp_seller_token_account_test = client.get_account(
        temp_seller_token_account.pubkey()
    ).await
    .expect("Some Error happened")
    .expect("Some Error happened");

    // to assert after exchange if token is transferred
    // to the reciever's token account
    let mint = spl_token::state::Account::unpack(
        &temp_seller_token_account_test.data
    ).unwrap().mint;

    transaction.sign(&[&payer, &initers_key], recent_blockhash);
    client.process_transaction(transaction).await.unwrap();

    // assert tests after the process_escrow_init has been completed
    let escrow_account_test = client.get_account(
        escrow_account.pubkey()
    ).await
    .expect("Unable to find escrow account")
    .expect("Unable to find escrow account");

    let temp_token_account = client.get_account(
        temp_seller_token_account.pubkey()
    ).await
    .expect("Unable to find escrow account")
    .expect("Unable to find escrow account");

    let escrow_account_state = Escrow::unpack(
        &escrow_account_test.data
    ).unwrap();

    let token_account_state = spl_token::state::Account::unpack(
        &temp_token_account.data
    ).unwrap();

    // check if temp token account owner is now the pda
    // and not the initializer
    assert_ne!(token_account_state.owner, initers_key.pubkey());

    assert_eq!(
        escrow_account_state.is_initialized, 
        true
    );

    data[0] = 1;
    let mut transaction = Transaction::new_with_payer(
        &[
            Instruction::new_with_bytes(
                escrow_program_id,
                &data,
                vec![
                    AccountMeta::new(taker_keypair.pubkey(), true),
                    AccountMeta::new(taker_to_recieve_keypair.pubkey(), false),
                    AccountMeta::new(temp_seller_token_account.pubkey(), false),
                    AccountMeta::new(initers_key.pubkey(), false),
                    AccountMeta::new(mint_key.pubkey(), false),
                    AccountMeta::new(escrow_account.pubkey(), false),
                    AccountMeta::new(spl_token::id(), false),
                    AccountMeta::new(system_program::id(), false),
                    AccountMeta::new(pda, false),
                    AccountMeta::new(metadata_key, false),
                    AccountMeta::new(valhalla_pubkey, false),
                    AccountMeta::new(creators_pubkey, false)
                ],
            )
        ],
        Some(&payer.pubkey())
    );

    let initers_test_before = client.get_account(
        initers_key.pubkey()
    ).await
    .expect("Error while finding creators account")
    .expect("Error while finding creators account");

    println!("Initers account {:?}", initers_test_before);

    transaction.sign(&[&payer, &taker_keypair], recent_blockhash);
    client.process_transaction(transaction).await.unwrap();

    // assert tests after the process_exchange
    let escrow_account_test = client.get_account(
        escrow_account.pubkey()
    ).await
    .expect("Error while finding escrow account")
    .expect("Error while finding escrow account");

    let escrow_account_state = Escrow::unpack_unchecked(
        &escrow_account_test.data
    ).unwrap();

    let taker_token_recieve_account_test = client.get_account(
        taker_to_recieve_keypair.pubkey()
    ).await
    .expect("Error while finding taker token to recieve account")
    .expect("Error while finding taker token to recieve account");

    let taker_token_recieve_account_test_state = spl_token::state::Account::unpack(
        &taker_token_recieve_account_test.data
    ).unwrap();

    let creators_test = client.get_account(
        creators_pubkey
    ).await
    .expect("Error while finding creators account")
    .expect("Error while finding creators account");

    let valhalla_test = client.get_account(
        valhalla_pubkey
    ).await
    .expect("Error while finding creators account")
    .expect("Error while finding creators account");

    let initers_test = client.get_account(
        initers_key.pubkey()
    ).await
    .expect("Error while finding creators account")
    .expect("Error while finding creators account");

    let taker_test = client.get_account(
        taker_keypair.pubkey()
    ).await
    .expect("Error while finding creators account")
    .expect("Error while finding creators account");

    // assertion tests

    // assert if escrow state is_initialized false
    assert_eq!(
        escrow_account_state.is_initialized, 
        false
    );

    // assert of token reciever has got the token into 
    // their account after the exchange
    assert_eq!(mint, taker_token_recieve_account_test_state.mint);

    // initial creator account lamport was 0
    // escrow amount - 123
    // seller_fee_basis_points - 4000
    // creator share - 70

    // formula used in process Exchange
    // (4000 * 123) / 10000 = 49.2
    // (70*49.2) / 100 = 34.44 round down to 34
    assert_eq!(creators_test.lamports, 34);

    // size = 123
    // val share = 250
    // amount to be sent to valhalla - (123*250) / 10000 = 3.075 round down to 3
    // i think decimals are stripped out because lamports
    // are itself the smallest unit of SOL

    assert_eq!(valhalla_test.lamports, 3);

    // assert if takers lamports are debited by 123
    // taker account initial lamports are 5616720
    assert_eq!(taker_test.lamports, 5616720-123);

    // seller paid 2039280 lamports for creating a 
    // associated temp token account while escrow init
    // now that the exchange is done, the temp token amount
    // will be closed, and the lamports needed to rent_exempt
    // the account will be refunded to sellers account
    // so the seller's account's lamport will be now
    // seller account initial lamports are 5616720
    // 5616720 + 2039280 + 86 (86 as the share he will get, after the sale)
    assert_eq!(initers_test.lamports, 5616720 + 2039280 + 86);

    // we have initialized the listing of NFT with 123 lamports
    // 86 - sellers share
    // 3 - valhalla's share
    // 34 - creators share
    // 86 + 3 + 34 = 123
}