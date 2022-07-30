use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg, 
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
    system_instruction::{transfer, create_account},

};

use metaplex_token_metadata::state::Metadata;
use std::str::FromStr;
use spl_token::state::Account as TokenAccount;
use crate::{ instruction::EscrowInstruction, state::{Escrow , VaultAccount} };
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        // Escrow instructions
        match instruction {
            EscrowInstruction::ListToken { amount } => {
                msg!("Instruction: ListToken");
                Self::process_init_escrow(accounts, amount, program_id)
            }
            EscrowInstruction::Exchange { amount } => {
                msg!("Instruction: Exchange");
                Self::process_exchange(accounts, amount, program_id)
            }
            EscrowInstruction::Cancel => {
                msg!("Instruction: Cancel");
                Self::process_cancel(accounts, program_id)
            }
            EscrowInstruction::UpdatePlatformAccount { amount } => {
                msg!("Instruction: Update platform accounts");
                Self::process_val_accounts(accounts, amount, program_id)
            }
        }
    }



    
    pub fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;
        // initializer is signer validation check
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let token_account = next_account_info(account_info_iter)?;

        let mint_key = next_account_info(account_info_iter)?;
        
        let escrow_account = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let token_program = next_account_info(account_info_iter)?;

        let system_program = next_account_info(account_info_iter)?;


        // mint validation check
        if *mint_key.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        let token_account_state = spl_token::state::Account::unpack(
            &**token_account.data.borrow()
        ).unwrap();

        // check if the token account have balance
        if token_account_state.amount != (1 as u64){
            msg!("invalid NFT data ** ..");
            return Err(ProgramError::InvalidAccountData);
        }
        // validate token account using mint
        if token_account_state.mint != *mint_key.key{
            msg!("invalid NFT data ** ..");
            return Err(ProgramError::InvalidAccountData);
        }
        
        invoke(
            &create_account(
                initializer.key, 
                escrow_account.key, 
                Rent::default().
                minimum_balance(
                    Escrow::LEN 
                ),
                Escrow::LEN as u64, 
                program_id,   
            ),
            &[
                initializer.clone(),
                escrow_account.clone(),
                system_program.clone(),
            ],
        )?;
    

        // check listing amount > 0
        if amount <= (0 as u64) {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.try_borrow_data()?)?;


        // set the state for escrow account
        escrow_info.is_initialized = true;
        escrow_info.seller_pubkey = *initializer.key;
        escrow_info.token_account_pubkey = *token_account.key;
        escrow_info.mint_key = *mint_key.key;
        escrow_info.expected_amount = amount;
        Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;

        // get a pda for escrow program
        let (pda, _nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

        // transfer the authority of token account from initializer to pda
        let owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[],
        )?;
        invoke(
            &owner_change_ix,
            &[
                token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;
        Ok(())
    }

    
    pub fn process_exchange(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        msg!("enter");
        let account_info_iter = &mut accounts.iter();
        let taker = next_account_info(account_info_iter)?;

        // check if the buyer is the singer for this instruction
        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        //let takers_token_to_receive_account = next_account_info(account_info_iter)?;
        let pdas_token_account = next_account_info(account_info_iter)?;
        let pdas_token_account_info =
            TokenAccount::unpack(&pdas_token_account.try_borrow_data()?)?;
        let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);

        // validation check for amount
        if amount_expected_by_taker != pdas_token_account_info.amount {
            return Err(ProgramError::InvalidInstructionData);
        }

        let initializers_main_account = next_account_info(account_info_iter)?;
        let mint_key = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        // check if owner of escrow account is the program
        if escrow_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let escrow_info = Escrow::unpack(&escrow_account.try_borrow_data()?)?;

        // validate data using Escrow state
        if escrow_info.token_account_pubkey != *pdas_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_info.seller_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_info.mint_key != *mint_key.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if *taker.key == *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let amomunt_expected_by_user = escrow_info.expected_amount.clone();

        let token_program = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let metadata_info = next_account_info(account_info_iter)?;

        // platform state account for valAccount struct
        let val_acc = next_account_info(account_info_iter)?;

        // check if owner of platform account is the program
        if *val_acc.owner != *program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        // platform team and treasury accounts
        let platform_treasury = next_account_info(account_info_iter)?;

        // get the percentages from the platform state account
        let val_acccount_info = VaultAccount::unpack(&val_acc.try_borrow_data()?)?;

        // validation checks for treasury and team accounts
        if val_acccount_info.treasury_account != *platform_treasury.key {
            return Err(ProgramError::InvalidAccountData);
        }


        // fetch onchain metadata account 
        const PREFIX: &str = "metadata";
        let metadata_program_id = Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();
        // seeds for metadata pda
        let metadata_seeds = &[
            PREFIX.as_bytes(),
            metadata_program_id.as_ref(),
            escrow_info.mint_key.as_ref(),
        ];
    
        let (metadata_key, _metadata_bump_seed) =
            Pubkey::find_program_address(metadata_seeds, &metadata_program_id);

        // validation check for correct accounts send from the client side
        if *metadata_info.key != metadata_key{
            return Err(ProgramError::InvalidAccountData);
        }

        let size = escrow_info.expected_amount;

        // unpack the metadata from the metadata pda
        let metadata = Metadata::from_account_info(metadata_info)?;

        // seller fee basis points from the metadata
        let fees = metadata.data.seller_fee_basis_points;
        let total_fee = ((fees as u64)*size)/10000;

        let mut remaining_fee = size;
        match metadata.data.creators {
            Some(creators) => {
                for creator in creators {
                    let pct = creator.share as u64;
                    let creator_fee = (pct*(total_fee))/100;
                    remaining_fee = remaining_fee - creator_fee;

                    let creator_acc_web = next_account_info(account_info_iter)?;

                    if *creator_acc_web.key != creator.address {
                        return Err(ProgramError::InvalidAccountData);
                    }

                    // send the royalties to the creators of the NFT
                    if creator_fee > 0 {

                        invoke(
                            &transfer(
                                taker.key,
                                creator_acc_web.key,
                                creator_fee,   
                            ),
                            &[
                                taker.clone(),
                                creator_acc_web.clone(),
                                taker.clone(),
                                system_program.clone(),
                            ],
                        )?;
                    }
                }
            }
            None => {
                msg!("No creators found in metadata");
            }
        }

        // get platform treasury + team % and convert it to SOL according
        // to the sale amount of the NFT
        // let base_percentge = val_acccount_info.base_percentage;

        let platform_fee = (size * val_acccount_info.base_percentage) / 10000;

        // transer SOL to platform fee account
        invoke(
            &transfer(
            taker.key,
            platform_treasury.key,
            platform_fee.clone(),   
            ),
            &[
                taker.clone(),
                platform_treasury.clone(),
                taker.clone(),
                system_program.clone(),
            ],
        )?;

        // calculate the remaining amount
        remaining_fee = remaining_fee - platform_fee;

        // transfer the remaining SOL to the seller
        let transfer_to_initializer_ix = transfer(
            taker.key,
            initializers_main_account.key,
            remaining_fee,   
        );

        invoke(
            &transfer_to_initializer_ix,
            &[
                taker.clone(),
                initializers_main_account.clone(),
                taker.clone(),
                system_program.clone(),
            ],
        )?;

        // transfer ownership authority of token account to the buyer
        let transfer_nft = spl_token::instruction::set_authority(
            token_program.key,
            pdas_token_account.key,
            Some(taker.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            &pda,
            &[],
        )?;
        invoke_signed(
            &transfer_nft,
            &[
                pdas_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        // unpack the escrow account
        // and set the state to is_initialized false
        let mut escrow_update_info = Escrow::unpack_unchecked(&escrow_account.try_borrow_data()?)?;

        escrow_update_info.is_initialized = false;
        escrow_update_info.seller_pubkey = *initializers_main_account.key;
        escrow_update_info.token_account_pubkey = *pdas_token_account.key;
        escrow_update_info.mint_key = *mint_key.key;
        escrow_update_info.expected_amount = amomunt_expected_by_user;
        Escrow::pack(escrow_update_info, &mut escrow_account.try_borrow_mut_data()?)?;

        Ok(())
    }


    pub fn process_cancel(
        accounts:&[AccountInfo],
        program_id: &Pubkey,
        // ft: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let pdas_token_account = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        // check if owner of escrow account is the program
        if escrow_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        // unpack the escrow state for some validation checks
        let escrow_info = Escrow::unpack(&escrow_account.try_borrow_data()?)?;

        let mint_key_data = escrow_info.mint_key.clone();
        let amomunt_expected_by_user = escrow_info.expected_amount.clone();

        // check if the user cancelling the listing is actually
        // the user who have listed it
        if escrow_info.seller_pubkey != *user.key {
            return Err(ProgramError::InvalidAccountData);
        }
        if escrow_info.token_account_pubkey != *pdas_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let (pda, nonce) = Pubkey::find_program_address(&[b"escrow"], program_id);
        let token_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        // transfer the ownership authority of the
        // token account back to the user who have initialized the escorw
        let cancel_listing_ix = spl_token::instruction::set_authority(
            token_program.key,
            pdas_token_account.key,
            Some(user.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            &pda,
            &[],
        )?;
        invoke_signed(
            &cancel_listing_ix,
            &[
                pdas_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[nonce]]],
        )?;

        // set the escorw state is_initialized to false
        let mut escrow_update_info = Escrow::unpack_unchecked(&escrow_account.try_borrow_data()?)?;

        escrow_update_info.is_initialized = false;

        Escrow::pack(escrow_update_info, &mut escrow_account.try_borrow_mut_data()?)?;

        Ok(())
    }

    pub fn process_val_accounts(
        accounts:&[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        msg!("fees percent {:?}", amount);


        //update authority of platform
        let admin_update_auth = Pubkey::from_str("J7A8AeFaPNxe3w7jCxnE2xHVWZz2GgjAF9LWky5AG2Jq").unwrap();
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;

        // validation check if the user calling this instruction
        // actually holds the authority for updating the platform account
        if admin_update_auth != *user.key {
            msg!("wrong update auth.....");
            return Err(ProgramError::InvalidAccountData);
        }        

        let platfrom_account = next_account_info(account_info_iter)?;

        // check if program owns platfrom_account account
        if platfrom_account.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        let treasury_acc = next_account_info(account_info_iter)?;

        if !rent.is_exempt(platfrom_account.lamports(), platfrom_account.data_len()) {
            return Err(ProgramError::InvalidInstructionData);
        }


        // unpack the platfrom_account state, to store data into
        let mut account_update_info = VaultAccount::unpack_unchecked(&platfrom_account.try_borrow_data()?)?;

        account_update_info.is_initialized = true;
        account_update_info.treasury_account = *treasury_acc.key;
        account_update_info.base_percentage = amount;

        msg!("fee percentage : {:?}", account_update_info.base_percentage);

        // pack data into the platform account
        VaultAccount::pack(account_update_info, &mut platfrom_account.try_borrow_mut_data()?)?;

        Ok(())
    }
}