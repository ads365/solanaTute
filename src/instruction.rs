//program api - used to process instruction data

//decalre libs?
use std::convert::TryInto;
use solana_program::program_error::ProgramError;

use crate::error::EscrowError::InvalidInstruction;


//instruction for escrow program

pub enum EscrowInstruction {

    //definition of accounts
    //0. [signer] account who initialises escrow
    //1. [writable] temporary token account
    //2. [read-only] initialisers token account for token they will reiceve
    //3. [writable] escrow account
    //4. [read-only] rent sysvar? sysvar is parameter of solana cluster you are on -- tells you the rent fee
    //5. [read-only] token program

    InitEscrow {

        ///ammount of Y token expected ot be recieved - will be parsed in instruction data
        amount: u64
    }
}

impl EscrowInstruction {
    
    //public function which expects an input u8/number and returns self or error
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        //is this fetching/decoding instructions?
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        //once fetched
        Ok(match tag {
            //self
          0 => Self::InitEscrow {
              amount: Self::unpack_amount(rest)?,
          },
          //error
          _=> return Err(InvalidInstruction.into()) 
        })
    }

    //function to unpack the ammount expects a u8 reurns a u64 - called by unpack fn
    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            //is this fetching u8?
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            //get a u64 from bytes
            .map(u64::from_le_bytes)
            //did it error?
            .ok_or(InvalidInstruction)?;
        //return amount
        Ok(amount)
    }
}