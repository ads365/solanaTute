// libs

//import core solana fns
use solana_program::{
    account_info::(next_account_info, AccountInfo),
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program_pack::{Pack, IsInitialized},
    sysvar::{rent::Rent, Sysvar},
};

//import the program API?
use crate::instruction::EscrowInstruction;

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
        program_id &Pubkey,
    ) -> ProgramResult {
        //itterate through accounts
        let account_info_iter = &mut accounts.iter();
        //expect first account to be initializer im guessing next account info is being parsed the itteration of an address and return bool
        let initializer = next_account_info(account_info_iter)?;

        //if account isnt initializer error
        if !initializer.is_signer {
            return Err(programError::MissingRequiredSignatures)
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

        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err (EscrowError::NotRentExempt.into());
        }

        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.data.borrow())

        Ok(())
    }
}
