
use solana_program::program_error::ProgramError;
use std::convert::TryInto;
// use borsh::{BorshDeserialize, BorshSerialize};

// use crate::error::EscrowError::InvalidInstruction;
// use std::io::Read;

#[derive(Debug, PartialEq)]
pub enum EscrowInstruction {

    ListToken {
        amount: u64,
    },

    Exchange {
        /// the amount the taker expects to be paid in the other token, as a u64 because that's the max possible supply of a token
        amount: u64,
    },
    
    
    Cancel,

    UpdatePlatformAccount{
        amount: u64,
    },
}


impl EscrowInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::ListToken {
                amount: Self::unpack_amount(rest)?,
            },
            1 => Self::Exchange {
                amount: Self::unpack_amount(rest)?,
            },
            2 => Self::Cancel,
            3 => Self::UpdatePlatformAccount {
                amount: Self::unpack_amount(rest)?,
            },
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(ProgramError::InvalidInstructionData)?;
        Ok(amount)
    }
}