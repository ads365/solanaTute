use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive (Error, Debug, Copy, Clone)]

//define error types
pub enum EscrowError {
    //invalid instruction error
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<EscrowError> for ProgramError {

    convert escrow error to program error
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e and u32)
    }
}