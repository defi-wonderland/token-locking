use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::PrintProgramError,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    sysvar::{clock::Clock, Sysvar},
};

use num_traits::FromPrimitive;
use spl_token::{instruction::transfer, state::Account};

use crate::{
    error::VestingError,
    instruction::{Schedule, VestingInstruction, SCHEDULE_SIZE},
    state::{pack_schedule_into_slice, unpack_schedule, VestingSchedule, VestingScheduleHeader},
};

pub struct Processor {}

impl Processor {
    pub fn process_init(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32]
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let system_program_account = next_account_info(accounts_iter)?;
        let rent_sysvar_account = next_account_info(accounts_iter)?;
        let payer = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;

        let rent = Rent::from_account_info(rent_sysvar_account)?;

        // Find the non reversible public key for the vesting contract via the seed
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
        if vesting_account_key != *vesting_account.key {
            msg!("Provided vesting account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        let state_size = VestingSchedule::LEN + VestingScheduleHeader::LEN;

        let init_vesting_account = create_account(
            &payer.key,
            &vesting_account_key,
            rent.minimum_balance(state_size),
            state_size as u64,
            &program_id,
        );

        invoke_signed(
            &init_vesting_account,
            &[
                system_program_account.clone(),
                payer.clone(),
                vesting_account.clone(),
            ],
            &[&[&seeds]],
        )?;
        Ok(())
    }

    pub fn process_create(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32],
        mint_address: &Pubkey,
        destination_token_address: &Pubkey,
        schedule: Schedule,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let source_token_account_owner = next_account_info(accounts_iter)?;
        let source_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Provided vesting account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        if !source_token_account_owner.is_signer {
            msg!("Source token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument);
        }

        if *vesting_account.owner != *program_id {
            msg!("Program should own vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        // Verifying that no SVC was already created with this seed
        let is_initialized =
            vesting_account.try_borrow_data()?[VestingScheduleHeader::LEN - 1] == 1;

        if is_initialized {
            msg!("Cannot overwrite an existing vesting contract.");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        if vesting_token_account_data.delegate.is_some() {
            msg!("The vesting token account should not have a delegate authority");
            return Err(ProgramError::InvalidAccountData);
        }

        if vesting_token_account_data.close_authority.is_some() {
            msg!("The vesting token account should not have a close authority");
            return Err(ProgramError::InvalidAccountData);
        }

        let state_header = VestingScheduleHeader {
            destination_address: *destination_token_address,
            mint_address: *mint_address,
            is_initialized: true,
        };

        let mut data = vesting_account.data.borrow_mut();
        if data.len() != VestingScheduleHeader::LEN + VestingSchedule::LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        state_header.pack_into_slice(&mut data);

        let offset = VestingScheduleHeader::LEN;
        let mut total_amount: u64 = 0;

        let state_schedule = VestingSchedule {
            release_time: schedule.release_time,
            amount: schedule.amount,
        };
        state_schedule.pack_into_slice(&mut data[offset..]);
        let delta = total_amount.checked_add(schedule.amount);
        match delta {
            Some(n) => total_amount = n,
            None => return Err(ProgramError::InvalidInstructionData), // Total amount overflows u64
        }
        
        if Account::unpack(&source_token_account.data.borrow())?.amount < total_amount {
            msg!("The source token account has insufficient funds.");
            return Err(ProgramError::InsufficientFunds)
        };

        let transfer_tokens_to_vesting_account = transfer(
            spl_token_account.key,
            source_token_account.key,
            vesting_token_account.key,
            source_token_account_owner.key,
            &[],
            total_amount,
        )?;

        invoke(
            &transfer_tokens_to_vesting_account,
            &[
                source_token_account.clone(),
                vesting_token_account.clone(),
                spl_token_account.clone(),
                source_token_account_owner.clone(),
            ],
        )?;
        Ok(())
    }

    pub fn process_unlock(
        program_id: &Pubkey,
        _accounts: &[AccountInfo],
        seeds: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut _accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let clock_sysvar_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        if spl_token_account.key != &spl_token::id() {
            msg!("The provided spl token program account is invalid");
            return Err(ProgramError::InvalidArgument)
        }

        let packed_state = &vesting_account.data;
        let header_state =
            VestingScheduleHeader::unpack(&packed_state.borrow()[..VestingScheduleHeader::LEN])?;

        if header_state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        // Unlock the schedules that have reached maturity
        let clock = Clock::from_account_info(&clock_sysvar_account)?;
        let mut total_amount_to_transfer = 0;
        let mut schedule = unpack_schedule(&packed_state.borrow()[VestingScheduleHeader::LEN..])?;

        if schedule.release_time == 0 {
            msg!("Should initialize withdrawal first");
            return Err(ProgramError::InvalidArgument);
        }

        if clock.unix_timestamp as u64 >= schedule.release_time {
            total_amount_to_transfer += schedule.amount;
            schedule.amount = 0;
        }

        if total_amount_to_transfer == 0 {
            msg!("Vesting contract has not yet reached release time");
            return Err(ProgramError::InvalidArgument);
        }

        let transfer_tokens_from_vesting_account = transfer(
            &spl_token_account.key,
            &vesting_token_account.key,
            destination_token_account.key,
            &vesting_account_key,
            &[],
            total_amount_to_transfer,
        )?;

        invoke_signed(
            &transfer_tokens_from_vesting_account,
            &[
                spl_token_account.clone(),
                vesting_token_account.clone(),
                destination_token_account.clone(),
                vesting_account.clone(),
            ],
            &[&[&seeds]],
        )?;

        // Reset released amounts to 0. This makes the simple unlock safe with complex scheduling contracts
        pack_schedule_into_slice(
            schedule,
            &mut packed_state.borrow_mut()[VestingScheduleHeader::LEN..],
        );

        Ok(())
    }

    pub fn process_initialize_unlock(
        program_id: &Pubkey,
        _accounts: &[AccountInfo],
        seeds: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut _accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let clock_sysvar_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;

        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        if spl_token_account.key != &spl_token::id() {
            msg!("The provided spl token program account is invalid");
            return Err(ProgramError::InvalidArgument)
        }

        let packed_state = &vesting_account.data;
        let header_state =
            VestingScheduleHeader::unpack(&packed_state.borrow()[..VestingScheduleHeader::LEN])?;

        if header_state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        // Unlock the schedules that have reached maturity
        let clock = Clock::from_account_info(&clock_sysvar_account)?;
        let mut schedule = unpack_schedule(&packed_state.borrow()[VestingScheduleHeader::LEN..])?;

        if schedule.amount == 0 {
            msg!("Vesting contract already claimed");
            return Err(ProgramError::InvalidArgument);
        }

        if schedule.release_time != 0 {
            msg!("Shouldn't initialize withdrawal for already initialized schedule");
            return Err(ProgramError::InvalidArgument);
        }
        
        // TODO: make test advance in time between initialize and unlock
        schedule.release_time = 1; // clock.unix_timestamp as u64 + 604800; // 7 days

        // Reset released amounts to 0. This makes the simple unlock safe with complex scheduling contracts
        pack_schedule_into_slice(
            schedule,
            &mut packed_state.borrow_mut()[VestingScheduleHeader::LEN..],
        );

        Ok(())
    }

    pub fn process_change_destination(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let vesting_account = next_account_info(accounts_iter)?;
        let destination_token_account = next_account_info(accounts_iter)?;
        let destination_token_account_owner = next_account_info(accounts_iter)?;
        let new_destination_token_account = next_account_info(accounts_iter)?;

        if vesting_account.data.borrow().len() < VestingScheduleHeader::LEN {
            return Err(ProgramError::InvalidAccountData)
        }
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        let state = VestingScheduleHeader::unpack(
            &vesting_account.data.borrow()[..VestingScheduleHeader::LEN],
        )?;

        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        if state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        if !destination_token_account_owner.is_signer {
            msg!("Destination token account owner should be a signer.");
            return Err(ProgramError::InvalidArgument);
        }

        let destination_token_account = Account::unpack(&destination_token_account.data.borrow())?;

        if destination_token_account.owner != *destination_token_account_owner.key {
            msg!("The current destination token account isn't owned by the provided owner");
            return Err(ProgramError::InvalidArgument);
        }

        let mut new_state = state;
        new_state.destination_address = *new_destination_token_account.key;
        new_state
            .pack_into_slice(&mut vesting_account.data.borrow_mut()[..VestingScheduleHeader::LEN]);

        Ok(())
    }

    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Instruction unpacked");
        match instruction {
            VestingInstruction::Init {
                seeds,
            } => {
                msg!("Instruction: Init");
                Self::process_init(program_id, accounts, seeds)
            }
            VestingInstruction::Unlock { seeds } => {
                msg!("Instruction: Unlock");
                Self::process_unlock(program_id, accounts, seeds)
            }
            VestingInstruction::InitializeUnlock { seeds } => {
                msg!("Instruction: InitializeUnlock");
                Self::process_initialize_unlock(program_id, accounts, seeds)
            }
            VestingInstruction::ChangeDestination { seeds } => {
                msg!("Instruction: Change Destination");
                Self::process_change_destination(program_id, accounts, seeds)
            }
            VestingInstruction::Create {
                seeds,
                mint_address,
                destination_token_address,
                schedule,
            } => {
                msg!("Instruction: Create Schedule");
                Self::process_create(
                    program_id,
                    accounts,
                    seeds,
                    &mint_address,
                    &destination_token_address,
                    schedule,
                )
            }
        }
    }
}

impl PrintProgramError for VestingError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            VestingError::InvalidInstruction => msg!("Error: Invalid instruction!"),
        }
    }
}
