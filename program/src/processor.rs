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
    system_program,
    sysvar::{clock, rent, Sysvar},
};

use num_traits::FromPrimitive;
use spl_token::{instruction::transfer, state::Account};

use crate::{
    error::VestingError,
    instruction::{Schedule, VestingInstruction},
    state::{pack_schedule_into_slice, unpack_schedule, VestingSchedule, VestingScheduleHeader},
};

pub const TOKEN_MINT: Pubkey =
    solana_program::pubkey!("2ummN5q6x8iQid7BFcRp7oMgF3UMEXR8LrUSPYzuMHQH");

pub struct Processor {}

impl Processor {
    pub fn process_init(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seeds: [u8; 32],
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();

        let system_program_account = next_account_info(accounts_iter)?;
        let rent_sysvar_account = next_account_info(accounts_iter)?;
        let payer = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;

        // Validate that the system program account is correct
        if *system_program_account.key != system_program::ID {
            msg!("Invalid system program account");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the rent sysvar account is correct
        if *rent_sysvar_account.key != rent::ID {
            msg!("Invalid rent sysvar account");
            return Err(ProgramError::InvalidArgument);
        }

        let rent = Rent::from_account_info(rent_sysvar_account)?;

        // Create and validate the vesting account key with the provided seed
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], &program_id).unwrap();
        if vesting_account_key != *vesting_account.key {
            msg!("Provided vesting account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        let state_size = VestingSchedule::LEN + VestingScheduleHeader::LEN;

        // Create the vesting account creation instruction
        let init_vesting_account = create_account(
            &payer.key,
            &vesting_account_key,
            rent.minimum_balance(state_size),
            state_size as u64,
            &program_id,
        );

        // Invoke the vesting account creation instruction
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
        schedule: Schedule,
    ) -> ProgramResult {

        let accounts_iter = &mut accounts.iter();

        let spl_token_account = next_account_info(accounts_iter)?;
        let clock_sysvar_account = next_account_info(accounts_iter)?;
        let vesting_account = next_account_info(accounts_iter)?;
        let vesting_token_account = next_account_info(accounts_iter)?;
        let source_token_account_owner = next_account_info(accounts_iter)?;
        let source_token_account = next_account_info(accounts_iter)?;
            
        // Deserialize the account data into the TokenAccount struct
        let token_account = Account::unpack(&source_token_account.data.borrow())?;

        // Validate the mint address matches the expected token address
        if token_account.mint != TOKEN_MINT {
            msg!("Invalid token mint address");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate the SPL Token Program account
        if spl_token_account.key != &spl_token::id() {
            msg!("The provided spl token program account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate the Clock Sysvar account
        if *clock_sysvar_account.key != clock::ID {
            msg!("Invalid clock sysvar account");
            return Err(ProgramError::InvalidArgument);
        }

        // Get and validate vesting account key from the seeds
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the source token account owner is a signer
        if !source_token_account_owner.is_signer {
            msg!("Source token account owner should be a signer");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the vesting account is owned by the program
        if *vesting_account.owner != *program_id {
            msg!("Vesting account is not owned by this program");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the vesting account is not already initialized
        let is_initialized =
            vesting_account.try_borrow_data()?[VestingScheduleHeader::LEN - 1] == 1;

        if is_initialized {
            msg!("Cannot overwrite an existing vesting contract");
            return Err(ProgramError::InvalidArgument);
        }

        // Unpack the vesting token account and validate it
        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the vesting token account has no delegate
        if vesting_token_account_data.delegate.is_some() {
            msg!("The vesting token account should not have a delegate authority");
            return Err(ProgramError::InvalidAccountData);
        }

        // Validate that the vesting token account has no close authority
        if vesting_token_account_data.close_authority.is_some() {
            msg!("The vesting token account should not have a close authority");
            return Err(ProgramError::InvalidAccountData);
        }

        // Pack the vesting schedule header into the vesting account data
        let state_header = VestingScheduleHeader {
            destination_address: *source_token_account.key,
            is_initialized: true,
        };

        // Validate that the schedule data is not corrupted
        let mut data = vesting_account.data.borrow_mut();
        if data.len() != VestingScheduleHeader::LEN + VestingSchedule::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        state_header.pack_into_slice(&mut data);

        // Retrieve the clock sysvar and validate schedule time delta
        let clock = clock::Clock::from_account_info(&clock_sysvar_account)?;

        let release_time;
        match schedule.time_delta {
            /* Valid time_delta values:
             * 0: unlocked (with 7 day withdrawal period)
             * 3 months = 3 * 30 * 86400 = 7_776_000
             * 6 months = 6 * 30 * 86400 = 15_552_000
             * 9 months = 9 * 30 * 86400 = 23_328_000
             * 12 months = 12 * 30 * 86400 = 31_104_000
             */
            0 => {
                release_time = 0;
            }
            7_776_000 | 15_552_000 | 23_328_000 | 31_104_000 => {
                release_time = clock.unix_timestamp as u64 + schedule.time_delta;
            }
            _ => {
                msg!("Unsupported time delta: {}", schedule.time_delta);
                return Err(ProgramError::InvalidInstructionData);
            }
        }

        // Pack the schedule data
        let state_schedule = VestingSchedule {
            release_time: release_time,
            amount: schedule.amount,
        };
        state_schedule.pack_into_slice(&mut data[VestingScheduleHeader::LEN..]);

        // Validate that the source token account has sufficient funds
        if Account::unpack(&source_token_account.data.borrow())?.amount < schedule.amount {
            msg!("The source token account has insufficient funds.");
            return Err(ProgramError::InsufficientFunds);
        }

        // Create the transfer instruction
        let transfer_tokens_to_vesting_account = transfer(
            spl_token_account.key,
            source_token_account.key,
            vesting_token_account.key,
            source_token_account_owner.key,
            &[],
            schedule.amount,
        )?;

        // Invoke the transfer instruction
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

        // Validate the SPL Token Program account
        if spl_token_account.key != &spl_token::id() {
            msg!("The provided spl token program account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the clock sysvar account is correct
        if *clock_sysvar_account.key != clock::ID {
            msg!("Invalid clock sysvar account");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the vesting account is owned by the program
        if *vesting_account.owner != *program_id {
            msg!("Vesting account is not owned by this program");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the vesting account public key is derived from the seeds
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the destination token account is the correct one from the schedule header
        let packed_state = &vesting_account.data;
        let header_state =
            VestingScheduleHeader::unpack(&packed_state.borrow()[..VestingScheduleHeader::LEN])?;

        if header_state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        // Unpack the vesting token account and validate it is owned by the vesting account
        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        // Unlock the schedules that have reached maturity
        let clock = clock::Clock::from_account_info(&clock_sysvar_account)?;
        let mut schedule = unpack_schedule(&packed_state.borrow()[VestingScheduleHeader::LEN..])?;

        let mut amount_to_transfer = 0;

        // Ensure the schedule has been initialized (release time should not be 0)
        if schedule.release_time == 0 {
            msg!("Should initialize withdrawal first");
            return Err(ProgramError::InvalidArgument);
        }

        // Check if the release time has been reached and release the amount
        if clock.unix_timestamp as u64 >= schedule.release_time {
            amount_to_transfer = schedule.amount;
            schedule.amount = 0;
        }

        // Validate that there is an amount to transfer
        if amount_to_transfer == 0 {
            msg!("Vesting contract has not yet reached release time");
            return Err(ProgramError::InvalidArgument);
        }

        // Create a token transfer from instruction
        let transfer_tokens_from_vesting_account = transfer(
            &spl_token_account.key,
            &vesting_token_account.key,
            destination_token_account.key,
            &vesting_account_key,
            &[],
            amount_to_transfer,
        )?;

        // Invoke the transfer from instruction
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

        // Reset the unlocked amounts in the schedule to 0 to avoid re-using
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

        // Validate the SPL Token Program account
        if spl_token_account.key != &spl_token::id() {
            msg!("The provided SPL token program account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate the Clock Sysvar account
        if *clock_sysvar_account.key != clock::ID {
            msg!("Invalid clock sysvar account");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate the vesting account key derived from seeds
        let vesting_account_key = Pubkey::create_program_address(&[&seeds], program_id)?;
        if vesting_account_key != *vesting_account.key {
            msg!("Invalid vesting account key");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate the SPL Token Program account
        if spl_token_account.key != &spl_token::id() {
            msg!("The provided spl token program account is invalid");
            return Err(ProgramError::InvalidArgument);
        }

        // Validate that the vesting account is owned by the program
        if *vesting_account.owner != *program_id {
            msg!("Vesting account is not owned by this program");
            return Err(ProgramError::InvalidArgument);
        }

        // Unpack the vesting account's state
        let packed_state = &vesting_account.data;
        let header_state =
            VestingScheduleHeader::unpack(&packed_state.borrow()[..VestingScheduleHeader::LEN])?;

        // Validate that the destination token account matches the contract's stored destination address
        if header_state.destination_address != *destination_token_account.key {
            msg!("Contract destination account does not matched provided account");
            return Err(ProgramError::InvalidArgument);
        }

        // Unpack the vesting token account and validate ownership by the vesting account
        let vesting_token_account_data = Account::unpack(&vesting_token_account.data.borrow())?;

        if vesting_token_account_data.owner != vesting_account_key {
            msg!("The vesting token account should be owned by the vesting account.");
            return Err(ProgramError::InvalidArgument);
        }

        // Unpack the schedule data
        let clock = clock::Clock::from_account_info(&clock_sysvar_account)?;
        let mut schedule = unpack_schedule(&packed_state.borrow()[VestingScheduleHeader::LEN..])?;

        // Check if the vesting contract has already been fully claimed
        if schedule.amount == 0 {
            msg!("Vesting contract already claimed");
            return Err(ProgramError::InvalidArgument);
        }

        // Ensure the withdrawal is not already initialized
        if schedule.release_time != 0 {
            msg!("Shouldn't initialize withdrawal for already initialized schedule");
            return Err(ProgramError::InvalidArgument);
        }

        // Withdrawal period is 7 days = 7 * 86400 = 604_800
        schedule.release_time = clock.unix_timestamp as u64 + 604_800;

        // Pack the updated schedule back into the account data
        pack_schedule_into_slice(
            schedule,
            &mut packed_state.borrow_mut()[VestingScheduleHeader::LEN..],
        );

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
            VestingInstruction::Init { seeds } => {
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
            VestingInstruction::Create {
                seeds,
                schedule,
            } => {
                msg!("Instruction: Create Schedule");
                Self::process_create(program_id, accounts, seeds, schedule)
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
