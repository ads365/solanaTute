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
    program::invoke
};

//import the program API?
use crate::{instruction::EscrowInstruction, error::EscrowError, state::Escrow};

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
        }
    }

    //call init escrow
    fn process_init_escrow(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        //itterate through accounts
        let account_info_iter = &mut accounts.iter();
        //expect first account to be initializer im guessing next account info is being parsed the itteration of an address and return bool
        let initializer = next_account_info(account_info_iter)?;

        //if account isnt initializer error
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

        //create program derived address - PDA to hhold temporary tokens - parsing array of seed and program
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
}
