//init libraries
use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct Escrow {
    //determine if escrow account is initializes/in use
    pub is_initialized: bool,
    pub initializer_pubkey: Pubkey,
    //expected temp token account address
    pub temp_token_account_pubkey: Pubkey,
    //initializers address of account to send tokens to
    pub initializer_token_to_recieve_account_pubkey: Pubkey,
    //used to check correct number of tokens are sent.
    pub expected_amount: u64,
}

impl Sealed for Escrow {}

//parse self and return bool if initialized
impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Escrow {
    //define the legnth of our Escrow type/structe - the gloabls are this many bytes
    //bools are 1 byte, addresses/pukeys are 32 ea,  u64 is 8
    const LEN: usize = 105;
    //deserialize
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        //turn array into escrow struct
        let src = array_ref![src, 0, Escrow::LEN];
        //adding initaial data types into array
        let (
            is_initialized,
            initializer_pubkey,
            temp_token_account_pubkey,
            initializer_token_to_recieve_account_pubkey,
            expected_amount,
        ) = array_refs![src, 1, 32, 32, 32, 8];
        //map the bools for is_initialized
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Escrow {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            temp_token_account_pubkey: Pubkey::new_from_array(*temp_token_account_pubkey),
            initializer_token_to_recieve_account_pubkey: Pubkey::new_from_array(*initializer_token_to_recieve_account_pubkey),
            expected_amount: u64::from_le_bytes(*expected_amount),
        })
    }

    //serialize parsing self - converting to serialized data
    //self exists now as we are serializing info
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Escrow::LEN];

        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            temp_token_account_pubkey_dst,
            initializer_token_to_recieve_account_pubkey_dst,
            expected_amount_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8];

        let Escrow {
            is_initialized,
            initializer_pubkey,
            temp_token_account_pubkey,
            initializer_token_to_recieve_account_pubkey,
            expected_amount,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        temp_token_account_pubkey_dst.copy_from_slice(temp_token_account_pubkey.as_ref());
        initializer_token_to_recieve_account_pubkey_dst.copy_from_slice(initializer_token_to_recieve_account_pubkey.as_ref());
        *expected_amount_dst = expected_amount.to_le_bytes();
    }
}