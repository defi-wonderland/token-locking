use crate::error::VestingError;

use solana_program::{
    instruction::{AccountMeta, Instruction},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey
};

use std::convert::TryInto;
use std::mem::size_of;

#[cfg(feature = "fuzz")]
use arbitrary::Arbitrary;

#[cfg(feature = "fuzz")]
impl Arbitrary for VestingInstruction {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let seeds: [u8; 32] = u.arbitrary()?;
        let choice = u.choose(&[0, 1, 2])?;
        match choice {
            0 => {
                let number_of_schedules = u.arbitrary()?;
                return Ok(Self::Init {
                    seeds,
                    number_of_schedules,
                });
            }
            1 => {
                let schedules: [Schedule; 10] = u.arbitrary()?;
                let key_bytes: [u8; 32] = u.arbitrary()?;
                let mint_address: Pubkey = Pubkey::new_from_array(key_bytes);
                let key_bytes: [u8; 32] = u.arbitrary()?;
                let destination_token_address: Pubkey = Pubkey::new_from_array(key_bytes);
                return Ok(Self::Create {
                    seeds,
                    mint_address,
                    destination_token_address,
                    schedules: schedules.to_vec(),
                });
            }
            _ => return Ok(Self::Unlock { seeds }),
        }
    }
}

#[cfg_attr(feature = "fuzz", derive(Arbitrary))]
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    // Schedule release time in unix timestamp
    pub release_time: u64,
    pub amount: u64,
}

pub const SCHEDULE_SIZE: usize = 16;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum VestingInstruction {
    /// Initializes an empty program account for the token_vesting program
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The system program account
    ///   1. `[]` The sysvar Rent account
    ///   1. `[signer]` The fee payer account
    ///   1. `[]` The vesting account
    Init {
        // The seed used to derive the vesting accounts address
        seeds: [u8; 32],
        // The number of release schedules for this contract to hold
        number_of_schedules: u32,
    },
    /// Creates a new vesting schedule contract
    ///
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The spl-token program account
    ///   1. `[writable]` The vesting account
    ///   2. `[writable]` The vesting spl-token account
    ///   3. `[signer]` The source spl-token account owner
    ///   4. `[writable]` The source spl-token account
    Create {
        seeds: [u8; 32],
        mint_address: Pubkey,
        schedules: Vec<Schedule>,
    },
    /// Unlocks a simple vesting contract (SVC) - can only be invoked by the program itself
    /// Accounts expected by this instruction:
    ///
    ///   * Single owner
    ///   0. `[]` The spl-token program account
    ///   1. `[]` The clock sysvar account
    ///   1. `[writable]` The vesting account
    ///   2. `[writable]` The vesting spl-token account
    ///   3. `[writable]` The destination spl-token account
    Unlock { seeds: [u8; 32] },
}

impl VestingInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use VestingError::InvalidInstruction;
        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let seeds: [u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();
                let number_of_schedules = rest
                    .get(32..36)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u32::from_le_bytes)
                    .ok_or(InvalidInstruction)?;
                Self::Init {
                    seeds,
                    number_of_schedules,
                }
            }
            1 => {
                let seeds: [u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();
                let mint_address = rest
                    .get(32..64)
                    .and_then(|slice| slice.try_into().ok())
                    .map(Pubkey::new_from_array)
                    .ok_or(InvalidInstruction)?;
                let number_of_schedules = rest[64..].len() / SCHEDULE_SIZE;
                let mut schedules: Vec<Schedule> = Vec::with_capacity(number_of_schedules);
                let mut offset = 64;
                for _ in 0..number_of_schedules {
                    let release_time = rest
                        .get(offset..offset + 8)
                        .and_then(|slice| slice.try_into().ok())
                        .map(u64::from_le_bytes)
                        .ok_or(InvalidInstruction)?;
                    let amount = rest
                        .get(offset + 8..offset + 16)
                        .and_then(|slice| slice.try_into().ok())
                        .map(u64::from_le_bytes)
                        .ok_or(InvalidInstruction)?;
                    offset += SCHEDULE_SIZE;
                    schedules.push(Schedule {
                        release_time,
                        amount,
                    })
                }
                Self::Create {
                    seeds,
                    mint_address,
                    schedules,
                }
            }
            2 => {
                let seeds: [u8; 32] = rest
                    .get(..32)
                    .and_then(|slice| slice.try_into().ok())
                    .unwrap();
                Self::Unlock { seeds }
            }
            _ => {
                msg!("Unsupported tag");
                return Err(InvalidInstruction.into());
            }
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::Init {
                seeds,
                number_of_schedules,
            } => {
                buf.push(0);
                buf.extend_from_slice(&seeds);
                buf.extend_from_slice(&number_of_schedules.to_le_bytes())
            }
            Self::Create {
                seeds,
                mint_address,
                schedules,
            } => {
                buf.push(1);
                buf.extend_from_slice(seeds);
                buf.extend_from_slice(&mint_address.to_bytes());
                for s in schedules.iter() {
                    buf.extend_from_slice(&s.release_time.to_le_bytes());
                    buf.extend_from_slice(&s.amount.to_le_bytes());
                }
            }
            &Self::Unlock { seeds } => {
                buf.push(2);
                buf.extend_from_slice(&seeds);
            }
        };
        buf
    }
}

// Creates a `Init` instruction to create and initialize the vesting token account.
pub fn init(
    system_program_id: &Pubkey,
    rent_program_id: &Pubkey,
    vesting_program_id: &Pubkey,
    payer_key: &Pubkey,
    vesting_account: &Pubkey,
    seeds: [u8; 32],
    number_of_schedules: u32,
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::Init {
        seeds,
        number_of_schedules,
    }
    .pack();
    let accounts = vec![
        AccountMeta::new_readonly(*system_program_id, false),
        AccountMeta::new_readonly(*rent_program_id, false),
        AccountMeta::new(*payer_key, true),
        AccountMeta::new(*vesting_account, false),
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data,
    })
}

// Creates a `CreateSchedule` instruction
pub fn create(
    vesting_program_id: &Pubkey,
    token_program_id: &Pubkey,
    vesting_account_key: &Pubkey,
    vesting_token_account_key: &Pubkey,
    source_token_account_owner_key: &Pubkey,
    source_token_account_key: &Pubkey,
    mint_address: &Pubkey,
    schedules: Vec<Schedule>,
    seeds: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::Create {
        mint_address: *mint_address,
        seeds,
        schedules,
    }
    .pack();
    let accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new(*vesting_token_account_key, false),
        AccountMeta::new_readonly(*source_token_account_owner_key, true),
        AccountMeta::new(*source_token_account_key, false),
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data,
    })
}

// Creates an `Unlock` instruction
pub fn unlock(
    vesting_program_id: &Pubkey,
    token_program_id: &Pubkey,
    clock_sysvar_id: &Pubkey,
    vesting_account_key: &Pubkey,
    vesting_token_account_key: &Pubkey,
    destination_token_account_key: &Pubkey,
    seeds: [u8; 32],
) -> Result<Instruction, ProgramError> {
    let data = VestingInstruction::Unlock { seeds }.pack();
    let accounts = vec![
        AccountMeta::new_readonly(*token_program_id, false),
        AccountMeta::new_readonly(*clock_sysvar_id, false),
        AccountMeta::new(*vesting_account_key, false),
        AccountMeta::new(*vesting_token_account_key, false),
        AccountMeta::new(*destination_token_account_key, false),
    ];
    Ok(Instruction {
        program_id: *vesting_program_id,
        accounts,
        data,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instruction_packing() {
        let mint_address = Pubkey::new_unique();

        let original_create = VestingInstruction::Create {
            seeds: [50u8; 32],
            schedules: vec![Schedule {
                amount: 42,
                release_time: 250,
            }],
            mint_address: mint_address.clone(),
        };
        let packed_create = original_create.pack();
        let unpacked_create = VestingInstruction::unpack(&packed_create).unwrap();
        assert_eq!(original_create, unpacked_create);

        let original_unlock = VestingInstruction::Unlock { seeds: [50u8; 32] };
        assert_eq!(
            original_unlock,
            VestingInstruction::unpack(&original_unlock.pack()).unwrap()
        );

        let original_init = VestingInstruction::Init {
            number_of_schedules: 42,
            seeds: [50u8; 32],
        };
        assert_eq!(
            original_init,
            VestingInstruction::unpack(&original_init.pack()).unwrap()
        );
    }
}
