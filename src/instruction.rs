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
    },

        //exhcange where B takes the other side fo the trade
        //definition of accounts
        // 0. [signer] account of persont aking trade (B)
        // 1. [writable] takers token account for the token they send
        // 2. [writable] takers tokena ccount for the token they will recieve
        // 3. [writable] PDAs temp token account to get tokens from and eventually close
        // 4. [writable] initialisers main account to send their rent
        // 5. [writable] initialisers token account to recieve token
        // 6. [writable] escrow account holding the escrow info 
        // 7. [] token program
        // 8. PDA account

        Exchange {
            //amount the taker expects to recieve of the other token
            amount:u64,

        }

}

impl EscrowInstruction {
    
    //public function which expects an input u8/number and returns self or error
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        //is this fetching/decoding instructions?
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        //once fetched
        Ok(match tag {
            //return init escrow instruction and then call the unpack amount fn to return the amount.
          0 => Self::InitEscrow {
              amount: Self::unpack_amount(rest)?,
          },

          1 => Self::Exchange {
              amount: Self::unpack_amount(rest)?,
          },
          //error
          _=> return Err(InvalidInstruction.into()) 
        })
    }

    //function to unpack the ammount expects a u8 reurns a u64 - called by unpack fn returns the u64 to pass to program
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