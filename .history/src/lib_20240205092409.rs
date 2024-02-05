use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    program_pack::Pack,
    sysvar::{rent::Rent, Sysvar},
};

use borsh::{BorshDeserialize, BorshSerialize};
use std::cmp::min;

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct ReflectionToken {
    pub total_supply: u64,
    pub balances: Vec<u64>,
}

impl ReflectionToken {
    pub fn new(total_supply: u64) -> Self {
        let balances = vec![0; total_supply as usize];
        ReflectionToken { total_supply, balances }
    }

    pub fn transfer(&mut self, sender_index: usize, recipient_index: usize, amount: u64) {
        let reflection_amount = amount / 10;
        let fee = amount - reflection_amount;

        self.balances[sender_index] -= amount;
        self.balances[recipient_index] += amount - fee;
        self.balances[0] += reflection_amount; // Reflect the fee to the first holder
    }
}

#[entrypoint]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let accounts_data = &mut accounts_iter.skip(1); // Skip program account

    let token_account = next_account_info(accounts_iter)?;

    let mut token_data = ReflectionToken::try_from_slice(&token_account.data.borrow())?;

    match instruction_data {
        // Initialize the token with total supply
        b"initialize" => {
            let rent = Rent::get()?;
            let required_lamports = rent.minimum_balance(ReflectionToken::get_packed_len());

            if token_account.lamports() < required_lamports {
                return Err(ProgramError::InsufficientFunds);
            }

            token_data = ReflectionToken::new(u64::from_le_bytes(instruction_data[8..].try_into().unwrap()));
            token_data.serialize(&mut token_account.data.borrow_mut())?;
        }
        // Transfer tokens
        b"transfer" => {
            let sender_info = next_account_info(accounts_iter)?;
            let recipient_info = next_account_info(accounts_iter)?;

            let sender_index = sender_info.data.borrow()[8..].try_into().unwrap();
            let recipient_index = recipient_info.data.borrow()[8..].try_into().unwrap();
            let amount = u64::from_le_bytes(instruction_data[16..].try_into().unwrap());

            if sender_info.is_signer && sender_info.is_writable
                && recipient_info.is_writable
                && sender_index < token_data.balances.len()
                && recipient_index < token_data.balances.len()
            {
                token_data.transfer(sender_index, recipient_index, amount);
                token_data.serialize(&mut token_account.data.borrow_mut())?;
            } else {
                return Err(ProgramError::InvalidArgument);
            }
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}

entrypoint!(process_instruction);
