use bytemuck::{Pod, Zeroable};
use intermediate::Intermediate;
use schedule::Schedule;
use serde::{Deserialize, Serialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

pub use swiftness_stark::types::{Felt, LegacyCache, StarkProof};
use task::{RawTask, Tasks};
use verify::stark_verify::table_decommit::TableDecommitCache;

pub mod intermediate;
pub mod schedule;
pub mod task;
mod verify;

// declare and export the program's entrypoint
entrypoint!(process_instruction_data);

pub const PROGRAM_ID: &str = "HbyQVcEA8R6fp7SA6YbJsLNZsBcWjPgHckng1ZZGfUm2";

#[repr(u8)]
#[derive(Serialize, Deserialize)]
pub enum Entrypoint<'a> {
    PublishFragment { offset: usize, data: &'a [u8] },
    Schedule,
    VerifyProof,
}

#[derive(Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct ProofAccount {
    pub proof: StarkProof,                 // The proof to verify.
    pub cache: Cache,                      // Inner-task data.
    pub intermediate: Intermediate, // Values calculated while proving, and used for subsequent tasks.
    pub schedule: Schedule<RawTask, 1000>, // Tasks remaining to be executed.
}

#[derive(Debug, Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct Cache {
    pub legacy: LegacyCache,
    pub table: TableDecommitCache,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum VerificationStage {
    #[default]
    Publish = 0,
    Verify = 1,
    Verified = 2,
}

impl TryFrom<u8> for VerificationStage {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VerificationStage::Publish),
            1 => Ok(VerificationStage::Verify),
            2 => Ok(VerificationStage::Verified),
            _ => Err(ProgramError::Custom(6)),
        }
    }
}

pub fn process_instruction_data(
    _program_id: &Pubkey,
    account_info: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction: Entrypoint = bincode::deserialize(instruction_data).unwrap();
    let accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>> = &mut account_info.iter();
    let account = next_account_info(accounts_iter).unwrap();

    let mut account_data = account.try_borrow_mut_data()?;
    let mut stage = VerificationStage::try_from(account_data[0])?;

    // Skipping the first byte as stage, and 7 as padding to get the correct alignment.
    stage = process_instruction(instruction, &mut account_data[8..], stage)?;
    account_data[0] = stage as u8;

    Ok(())
}

// program entrypoint's implementation
pub fn process_instruction<'a>(
    instruction: Entrypoint<'a>,
    account_data: &mut [u8],
    stage: VerificationStage,
) -> Result<VerificationStage, ProgramError> {
    let stage_after = match instruction {
        Entrypoint::PublishFragment { offset, data } => {
            if stage != VerificationStage::Publish {
                return Err(ProgramError::Custom(7));
            }

            account_data[offset..offset + data.len()].copy_from_slice(data);
            msg!("PublishFragment");
            VerificationStage::Publish
        }

        Entrypoint::Schedule => {
            if stage != VerificationStage::Publish {
                return Err(ProgramError::Custom(8));
            }

            let ProofAccount { schedule, .. } =
                bytemuck::from_bytes_mut::<ProofAccount>(account_data);

            schedule.flush();
            schedule.push(Tasks::VerifyProofWithoutStark.into());

            msg!("Schedule");

            VerificationStage::Verify
        }

        Entrypoint::VerifyProof => {
            if stage != VerificationStage::Verify {
                return Err(ProgramError::Custom(9));
            }

            let ProofAccount {
                proof,
                cache,
                schedule,
                intermediate,
            } = bytemuck::from_bytes_mut::<ProofAccount>(account_data);

            let Some(task) = schedule.next() else {
                return Err(ProgramError::Custom(3));
            };

            let task = Tasks::try_from(task)?;
            let mut task = task.view(proof, cache, intermediate);

            let children = task.execute().unwrap();

            schedule.push_slice(
                &children
                    .iter()
                    .copied()
                    .map(|c| c.into())
                    .collect::<Vec<_>>(),
            );

            // return Err(ProgramError::Custom(42));

            if schedule.finished() {
                VerificationStage::Verified
            } else {
                VerificationStage::Verify
            }
        }
    };

    Ok(stage_after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use swiftness::{TransformTo, parse};

    pub fn read_proof() -> ProofAccount {
        let small_json = include_str!("../resources/small.json");
        let stark_proof = parse(small_json).unwrap();
        let proof = stark_proof.transform_to();

        ProofAccount {
            proof,
            ..Default::default()
        }
    }

    #[test]
    fn test_deserialize_proof() {
        let mut proof_account: ProofAccount = read_proof();
        let account_data = bytemuck::bytes_of_mut(&mut proof_account);

        let mut stage = VerificationStage::Publish;

        stage = process_instruction(Entrypoint::Schedule, account_data, stage).unwrap();
        let mut c = 0;

        while stage != VerificationStage::Verified {
            stage = process_instruction(Entrypoint::VerifyProof, account_data, stage).unwrap();
            c += 1;
        }

        assert_eq!(c, 6);

        let ProofAccount { intermediate, .. } = bytemuck::from_bytes::<ProofAccount>(account_data);

        assert_eq!(
            intermediate.program_hash().to_string(),
            "1134405407503728996667931466883426118808998438966777289406309056327695405399"
        );
        assert_eq!(
            intermediate.output(),
            &[Felt::from(0), Felt::from(1), Felt::from(5)]
        );
    }
}
