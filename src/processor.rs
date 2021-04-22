// libs

//import core solana fns
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
    program::invoke,
    program::invoke_signed,
};

//import externals to be referenced
use crate::{instruction::EscrowInstruction, error::EscrowError, state::Escrow};

use spl_token::state::Account as TokenAccount;

pub struct Processor;

impl Processor {

    //this process sets up the call to init escrow
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
        //instruction data from entrypoint
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        //determines what fucntion to call
        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                //logs event 
                msg!("instruction: InitEscrow");
                Self::process_init_escrow(accounts, amount, program_id)
            }

            EscrowInstruction::Exchange { amount} => {
                msg!("instruction: Exchange");
                Self::process_exchange(accounts, amount, program_id)
            }
        }
    }

    //call init escrow
    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        //create an itterator of accounts
        let account_info_iter = &mut accounts.iter();
        //get first account from the above itterator and set as initialier
        let initializer = next_account_info(account_info_iter)?;
        println!("{:?}", initializer);
        //we expect an intializer (bool should return true) if account isnt initializer error
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature)
        }

        //expect 2nd acocunt to be temp token account - dont need to check as it is temp and inactive
        let temp_token_account = next_account_info(account_info_iter)?;

        //expect 3rd account to be initialisers token account check token recieve account is owend by token program
        let token_to_recieve_account = next_account_info(account_info_iter)?;
        if *token_to_recieve_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        //expect this account to be escrow account
        let escrow_account = next_account_info(account_info_iter)?;

        //expect thsi to be the sysvar account
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        //check  if rent expempt if not error
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExempt.into());
        }

        //escrow info is something set from unpacking data in escrow acocunt
        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())?;
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        //populate escrow struct 
        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.temp_token_account_pubkey = *temp_token_account.key;
        escrow_info.initializer_token_to_recieve_account_pubkey = *token_to_recieve_account.key;
        escrow_info.expected_amount = amount;

        //pack escrow data?
        Escrow::pack(escrow_info, &mut escrow_account.data.borrow_mut())?;

        //create program derived address - PDA to hhold temporary tokens - parsing array of seeds and program id
        //PDA dont have a privat key, their pub key derived from program ID
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        //get token program account
        let token_program = next_account_info(account_info_iter)?;

        //instruction to change/set authority of token account - change to pda being owner
        let owner_change_ix = spl_token::instruction::set_authority (
            token_program.key,
            temp_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer.key,
            &[&initializer.key],
        )?;

        msg!("Calling token program to transfer token account ownership");
        //invoke Cross program invocation to toke nprogram to chang owner
        invoke(
            &owner_change_ix,
            &[
                temp_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;
        //do i keep this?
        Ok(())
    }

    fn process_exchange(
        accounts: &[AccountInfo],
        amount_expected_by_taker: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {

        //create an iteratino of accounts
        let account_info_iter = &mut accounts.iter();
        //first address in array is taker address
        let taker = next_account_info(account_info_iter)?;
        //check if the signer is the taker - permission check
        if !taker.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //the next account is the token account of twhat the taker will send
        let takers_sending_token_account = next_account_info(account_info_iter)?;

        //the next account is the takers reviece token account
        let takers_token_to_recieve_account = next_account_info(account_info_iter)?;

        //pda token account
        let pdas_temp_token_account = next_account_info(account_info_iter)?;

        //upack pda temp token account to get info
        let pdas_temp_token_account_info = TokenAccount::unpack(&pdas_temp_token_account.data.borrow())?;

        //read hte public key of a PDA to derive the program address?
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        //if PDA doesnt have amount expected by taker
        if amount_expected_by_taker != pdas_temp_token_account_info.amount {
            return Err(EscrowError::ExpectedAmountMismatch.into());
        }

        let initializers_main_account = next_account_info(account_info_iter)?;

        let initializers_token_to_recieve_account = next_account_info(account_info_iter)?;

        let escrow_account = next_account_info(account_info_iter)?;

        //unpack escrow account to get its information
        let escrow_info = Escrow::unpack(&escrow_account.data.borrow())?;

        //ensure right temp token account is being sent to
        if escrow_info.temp_token_account_pubkey != *pdas_temp_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        //ensure we ahve the correct initializer
        if escrow_info.initializer_pubkey != *initializers_main_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let token_program = next_account_info(account_info_iter)?;

        let transfer_to_initializer_ix = spl_token::instruction::transfer(
            token_program.key, //token
            takers_sending_token_account.key, //from
            initializers_token_to_recieve_account.key, //to
            taker.key, //takers address
            &[&taker.key],
            escrow_info.expected_amount, //expected amount
        )?;

        msg!("calling the token program to transfer tokens to the escrow's initializer....");

        //invoke the signature extension transaction and send tokens to initializer
        invoke(
            &transfer_to_initializer_ix,
            &[
                takers_sending_token_account.clone(),
                initializers_token_to_recieve_account.clone(),
                taker.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_to_taker_ix = spl_token::instruction::transfer(
            token_program.key, //token
            pdas_temp_token_account.key, //from
            takers_token_to_recieve_account.key, //to
            &pda,
            &[&pda],
            pdas_temp_token_account_info.amount, //amount
        )?;

        msg!("Calling token program to transfer tokens to taker");

        //invoke signature extension transaction and send token to taker
        invoke_signed(
            &transfer_to_taker_ix,
            &[
                pdas_temp_token_account.clone(),
                takers_token_to_recieve_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        //parameters of the internal tx
        let close_pdas_temp_acc_ix = spl_token::instruction::close_account(
            token_program.key,
            pdas_temp_token_account.key,
            initializers_main_account.key,
            &pda,
            &[&pda]
        )?;

        msg!("Calling the token program to close pda's temp account");

        //invoke the internal tx to close tem token account
        invoke_signed(
            &close_pdas_temp_acc_ix,
            &[
                pdas_temp_token_account.clone(),
                initializers_main_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        //clse escrow account by transferring out the remining balance - we do this by adding hte balnc eof escrow to the initializer
        msg!("closing the escrow account....");
        **initializers_main_account.lamports.borrow_mut() = initializers_main_account.lamports()
        .checked_add(escrow_account.lamports())
        .ok_or(EscrowError::AmountOverflow)?;
        **escrow_account.lamports.borrow_mut() = 0;

        Ok(())
    }
}