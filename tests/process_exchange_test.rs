#[cfg(test)]
mod tests {
    // using FromStr trait, Pubkey implements
    // this trait to generate pubkey from_str
    use std::str::FromStr;

    use solana_program::{
        program_pack::Pack,
        rent::Rent,
        system_program,
        pubkey::Pubkey
    };
    use solana_sdk::{
        account::create_is_signer_account_infos,
        account::Account as TestAccount,
    };

    use metaplex_token_metadata::state::Key;
    use metaplex_token_metadata::state::{
        Metadata,
        Data,
        Creator
    };
    use borsh::BorshSerialize;
    use spl_token::state::Account as TokenAccount;
    
    use escrow_buy::{
        processor::Processor,
        state::Escrow
    };
    
    // escrow exchange test
    #[test]
    fn process_process_exchange() {
        let escrow_program_id = Pubkey::from_str(
            &"escrowprogram111111111111111111111111111111"
        ).unwrap();

        let initializer_pubkey = Pubkey::new_unique();
        let temp_token_pubkey = Pubkey::new_unique();
        let mint_key_pubkey = Pubkey::new_unique();

        // token program id and account
        let token_program_id = spl_token::id();
        let mut token_program_account = TestAccount::default();

        // system program id and account
        let system_program_id = system_program::id();
        let mut system_program_account = TestAccount::default();

        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], &escrow_program_id); // temp_token_account owner pubkey

        let amount = 123;

        // setup escrow account
        let mut escrow_account = TestAccount {
            owner: pda,
            data: vec![0; Escrow::get_packed_len()],
            ..TestAccount::default()
        };
        Escrow {
            is_initialized: true,
            seller_pubkey: initializer_pubkey,
            token_account_pubkey: temp_token_pubkey,
            mint_key: mint_key_pubkey,
            expected_amount: amount,
        }
        .pack_into_slice(&mut escrow_account.data);

        // temp_token_account (account that ownership was set in  initialization)
        let mut temp_token_account = TestAccount {
            owner: pda,
            data: vec![0; TokenAccount::get_packed_len()],
            ..TestAccount::default()
        };

        TokenAccount {
            amount,
            state: spl_token::state::AccountState::Initialized,
            ..TokenAccount::default()
        }
        .pack_into_slice(&mut temp_token_account.data);

        let mut taker_account = TestAccount::default();
        let mut taker_token_receive_account = TestAccount::default();
        let mut initializer_account = TestAccount::default();
        let mut mint_key = TestAccount::default();
        let mut creators_account = TestAccount::default();
        let mut val_account = TestAccount::default();
        let mut pda_temp_account = TestAccount::default();

        let rent = Rent::default();

        // size of all the fields to be stored
        // in the metadata
        pub const MAX_NAME_LENGTH: usize = 32;

        pub const MAX_SYMBOL_LENGTH: usize = 10;

        pub const MAX_URI_LENGTH: usize = 200;

        pub const MAX_DATA_SIZE: usize = 4
            + MAX_NAME_LENGTH
            + 4
            + MAX_SYMBOL_LENGTH
            + 4
            + MAX_URI_LENGTH
            + 2
            + 1
            + 4
            + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN;

        pub const MAX_CREATOR_LIMIT: usize = 5;

        pub const MAX_CREATOR_LEN: usize = 32 + 1 + 1;

        pub const MAX_METADATA_LEN: usize = 1 + 32 + 32 + MAX_DATA_SIZE + 1 + 1 + 9 + 172;
        ///////////////////////////////////////

        let metadata_min_balance_needed = rent.minimum_balance(
            MAX_METADATA_LEN
        );
        
        // create a metadata test account with
        // len of metaplex metadata size 
        let mut metadata_account = TestAccount::new(
            metadata_min_balance_needed,
            MAX_METADATA_LEN,
            &Pubkey::new_unique()
        );

        let taker_pubkey = Pubkey::new_unique();
        let taker_token_receive_pubkey = Pubkey::new_unique();
        let escrow_pubkey = Pubkey::new_unique();
        let creators_pubkey = Pubkey::from_str(
            &"metadatacreatorL5LYvXwxBNSaVkinzjzvTt1j3XsQ"
        ).unwrap();
        let val_pubkey = Pubkey::from_str(
            &"paXi61MzXmioYZL5LYvXwxBNSaVkinzjzvTt1j3XsQz"
        ).unwrap();
    
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

        let mut accounts = [
            (
                &taker_pubkey, 
                true, 
                &mut taker_account
            ),
            (
                &taker_token_receive_pubkey,
                false,
                &mut taker_token_receive_account,
            ),
            (
                &temp_token_pubkey, 
                false, 
                &mut temp_token_account
            ),
            (
                &initializer_pubkey, 
                false, 
                &mut initializer_account
            ),
            (
                &mint_key_pubkey,
                false,
                &mut mint_key,
            ),
            (
                &escrow_pubkey, 
                false, 
                &mut escrow_account
            ),
            (
                &token_program_id, 
                false, 
                &mut token_program_account
            ),
            (
                &system_program_id, 
                false, 
                &mut system_program_account
            ),
            (
                &pda, 
                false, 
                &mut pda_temp_account
            ),
            (
                &metadata_key,
                false,
                &mut metadata_account,
            ),
            (
                &val_pubkey, 
                false, 
                &mut val_account
            ),
            (
                &creators_pubkey, 
                false, 
                &mut creators_account
            )
        ];

        let accounts = create_is_signer_account_infos(&mut accounts);

        let metadata = Metadata {
            key: Key::MetadataV1,
            update_authority: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            data: Data {
                name: String::from("Hello, world!"),
                symbol: String::from("Hello, world!"),
                uri: String::from("Hello, world!"),
                seller_fee_basis_points: 4000,
                creators: Some(vec![
                    Creator {
                        address: creators_pubkey,
                        verified: true,
                        share: 70
                    }
                ])
            },
            primary_sale_happened: false,
            is_mutable: false,
            edition_nonce: None
        };

        let metadata_accountinfo = &accounts[9];
        metadata.serialize(
            &mut &mut metadata_accountinfo.data.borrow_mut()[..]
        ).unwrap();

        let escrow_account_test = accounts[5].clone();
        let escrow_state_test = Escrow::unpack_from_slice(
            &escrow_account_test.data.borrow()
        ).unwrap();

        // assertion tests before escrow exchange
        // check if is_initialized is set to true
        assert_eq!(escrow_state_test.is_initialized, true);

        Processor::process_exchange(&accounts, amount, &escrow_program_id)
            .expect("error: process_exchange()");

        let escrow_account_test = accounts[5].clone();
        let escrow_state_test = Escrow::unpack_from_slice(
            &escrow_account_test.data.borrow()
        ).unwrap();

        // assertion tests after escrow exchange
        // check if is_initialized is set to false
        assert_eq!(escrow_state_test.is_initialized, false);
    }
}