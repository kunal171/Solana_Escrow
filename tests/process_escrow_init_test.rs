#[cfg(test)]
mod tests {
    // using FromStr trait, Pubkey implements
    // this trait to generate pubkey from_str
    use std::str::FromStr;
    use solana_program::{
        program_pack::Pack,
        rent::Rent,
        sysvar,
        pubkey::Pubkey
    };
    use solana_sdk::{
        account::create_account_for_test,
        account::create_is_signer_account_infos,
        account::Account as TestAccount,
        account::WritableAccount
    };

    use escrow_buy::{
        processor::Processor,
        state::Escrow
    };

    // Init escrow test
    #[test]
    fn init_escrow_test() {
        // create a program id for escrow program
        let escrow_program_id = Pubkey::from_str(
            &"escrowprogram111111111111111111111111111111"
        ).unwrap();

        // create rent account
        let rent = Rent::default();
        let mut rent_account = create_account_for_test(&rent);
        
        // escrow state packed length
        // to define space for new escrow account
        let escrow_state_length = Escrow::get_packed_len();

        // minimum balance needed to store the state
        let escrow_account_min_balance_needed = rent.minimum_balance(
            escrow_state_length
        );

        // token program id
        let token_program_id = spl_token::id();

        // initializers account
        let mut initializer_account = TestAccount::default();

        // temp token account
        let mut temp_token_account = TestAccount::default();

        // token recieve account
        // set its owner field to token program id
        let mut mint_key = TestAccount::default();
        mint_key.set_owner(token_program_id);

        // 3. escrow account with required lamports
        let mut escrow_account = TestAccount::new(
            escrow_account_min_balance_needed,
            escrow_state_length,
            &escrow_program_id
        );

        // 5. token program account
        let mut token_program_account = TestAccount::default();

        // create accounts for calling the process_init_escrow
        let mut accounts = [
            (
                &Pubkey::new_unique(), 
                true, 
                &mut initializer_account
            ),
            (
                &Pubkey::new_unique(), 
                true, 
                &mut temp_token_account
            ),
            (
                &Pubkey::new_unique(), 
                true, 
                &mut mint_key
            ),
            (
                &Pubkey::new_unique(),
                true,
                &mut escrow_account
            ),
            (
                &sysvar::rent::id(),
                true,
                &mut rent_account
            ),
            (
                &token_program_id,
                true,
                &mut token_program_account
            )
        ];

        let accounts = create_is_signer_account_infos(&mut accounts);
        
        // first test here
        Processor::process_init_escrow(
            &accounts, 
            123, 
            &escrow_program_id
        ).expect("Some error happened, test failed");
        
        let initializer_account = &accounts[0];
        let temp_token_account = &accounts[1];
        let mint_key = &accounts[2];
        let escrow_account = &accounts[3];

        let escrow_state = Escrow::unpack(
            &escrow_account.data.borrow()
        ).unwrap();
        
        // assertion tests for escrow account state
        assert_eq!(escrow_state.is_initialized, true);
        assert_eq!(escrow_state.seller_pubkey, *initializer_account.key);
        assert_eq!(escrow_state.token_account_pubkey, *temp_token_account.key);
        assert_eq!(escrow_state.mint_key, *mint_key.key);
        assert_eq!(escrow_state.expected_amount, 123 as u64);
    }
}