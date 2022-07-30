#[cfg(test)]
mod tests {
    use solana_program::{
        pubkey::Pubkey,
        program_pack::Pack
    };
    use escrow_buy::state::Escrow;

    // unit test for state pack unpack
    #[test]
    fn state_pack_unpack_test() {
        // create a test Escrow state
        let state = Escrow {
            is_initialized: true,
            seller_pubkey: Pubkey::new(&[1; 32]),
            token_account_pubkey: Pubkey::new(&[2; 32]),
            mint_key: Pubkey::new(&[3; 32]),
            expected_amount: 123
        };
        // temp packed data vec
        let mut packed_data = vec![0; Escrow::get_packed_len()];
        // pack escrow state
        Escrow::pack(state, &mut packed_data).unwrap();
        // unpack packed escrow state
        let unpacked_data = Escrow::unpack(&packed_data).unwrap();
        // do an assert check on the test escrow state and unpacked escrow state
        assert_eq!(state, unpacked_data);
    }
}