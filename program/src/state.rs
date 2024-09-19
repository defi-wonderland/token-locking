use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

use std::convert::TryInto;
#[derive(Debug, PartialEq)]
pub struct VestingSchedule {
    pub release_time: u64,
    pub amount: u64,
}

#[derive(Debug, PartialEq)]
pub struct VestingScheduleHeader {
    pub destination_address: Pubkey,
    pub mint_address: Pubkey,
    pub is_initialized: bool,
}

impl Sealed for VestingScheduleHeader {}

impl Pack for VestingScheduleHeader {
    const LEN: usize = 65;

    fn pack_into_slice(&self, target: &mut [u8]) {
        let destination_address_bytes = self.destination_address.to_bytes();
        let mint_address_bytes = self.mint_address.to_bytes();
        for i in 0..32 {
            target[i] = destination_address_bytes[i];
        }

        for i in 32..64 {
            target[i] = mint_address_bytes[i - 32];
        }

        target[64] = self.is_initialized as u8;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < 65 {
            return Err(ProgramError::InvalidAccountData)
        }
        let destination_address = Pubkey::new(&src[..32]);
        let mint_address = Pubkey::new(&src[32..64]);
        let is_initialized = src[64] == 1;
        Ok(Self {
            destination_address,
            mint_address,
            is_initialized,
        })
    }
}

impl Sealed for VestingSchedule {}

impl Pack for VestingSchedule {
    const LEN: usize = 16;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let release_time_bytes = self.release_time.to_le_bytes();
        let amount_bytes = self.amount.to_le_bytes();
        for i in 0..8 {
            dst[i] = release_time_bytes[i];
        }

        for i in 8..16 {
            dst[i] = amount_bytes[i - 8];
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < 16 {
            return Err(ProgramError::InvalidAccountData)
        }
        let release_time = u64::from_le_bytes(src[0..8].try_into().unwrap());
        let amount = u64::from_le_bytes(src[8..16].try_into().unwrap());
        Ok(Self {
            release_time,
            amount,
        })
    }
}

impl IsInitialized for VestingScheduleHeader {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

pub fn unpack_schedule(input: &[u8]) -> Result<VestingSchedule, ProgramError> {
    let offset = 0;
    let output: VestingSchedule = VestingSchedule::unpack_from_slice(
        &input[offset..offset + VestingSchedule::LEN],
    )?;
    Ok(output)
}

pub fn pack_schedule_into_slice(schedule: VestingSchedule, target: &mut [u8]) {
    let offset = 0;
    schedule.pack_into_slice(&mut target[offset..]);
}

#[cfg(test)]
mod tests {
    use super::{unpack_schedule, VestingSchedule, VestingScheduleHeader};
    use solana_program::{program_pack::Pack, pubkey::Pubkey};

    #[test]
    fn test_state_packing() {
        let header_state = VestingScheduleHeader {
            destination_address: Pubkey::new_unique(),
            mint_address: Pubkey::new_unique(),
            is_initialized: true,
        };
        let schedule_state = VestingSchedule {
            release_time: 30767976,
            amount: 969,
        };
        let state_size = VestingScheduleHeader::LEN + VestingSchedule::LEN;
        let mut state_array = [0u8; 81];
        header_state.pack_into_slice(&mut state_array[..VestingScheduleHeader::LEN]);
        schedule_state.pack_into_slice(
            &mut state_array
                [VestingScheduleHeader::LEN..VestingScheduleHeader::LEN + VestingSchedule::LEN],
        );
        let packed = Vec::from(state_array);
        let mut expected = Vec::with_capacity(state_size);
        expected.extend_from_slice(&header_state.destination_address.to_bytes());
        expected.extend_from_slice(&header_state.mint_address.to_bytes());
        expected.extend_from_slice(&[header_state.is_initialized as u8]);
        expected.extend_from_slice(&schedule_state.release_time.to_le_bytes());
        expected.extend_from_slice(&schedule_state.amount.to_le_bytes());

        assert_eq!(expected, packed);
        assert_eq!(packed.len(), state_size);
        let unpacked_header =
            VestingScheduleHeader::unpack(&packed[..VestingScheduleHeader::LEN]).unwrap();
        assert_eq!(unpacked_header, header_state);
        let unpacked_schedules = unpack_schedule(&packed[VestingScheduleHeader::LEN..]).unwrap();
        assert_eq!(unpacked_schedules, schedule_state);
    }
}
