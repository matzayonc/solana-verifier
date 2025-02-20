use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};

use stack::Schedule;
pub use swiftness_stark::types::{Cache, Felt, StarkProof};
use task::Task;

pub mod stack;
pub mod task;

// declare and export the program's entrypoint
entrypoint!(process_instruction_data);

pub const PROGRAM_ID: &str = "HbyQVcEA8R6fp7SA6YbJsLNZsBcWjPgHckng1ZZGfUm2";

#[repr(u8)]
#[derive(Serialize, Deserialize)]
pub enum Entrypoint<'a> {
    PublishFragment { offset: usize, data: &'a [u8] },
    VerifyProof {},
}

#[derive(Clone, Copy, Default, Zeroable, Pod)]
#[repr(C)]
pub struct ProofAccount {
    pub proof: StarkProof,
    pub cache: Cache,
    pub schedule: Schedule<u8, 1000>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum VerificationStage {
    #[default]
    Publish,
    Verify,
    Verified,
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

    if matches!(instruction, Entrypoint::VerifyProof {}) {
        stage = VerificationStage::Verify;
    }

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
    match instruction {
        Entrypoint::PublishFragment { offset, data } => {
            if stage != VerificationStage::Publish {
                return Err(ProgramError::Custom(7));
            }

            account_data[offset..offset + data.len()].copy_from_slice(data);
            msg!("PublishFragment");
        }

        Entrypoint::VerifyProof {} => {
            if stage != VerificationStage::Verify {
                return Err(ProgramError::Custom(8));
            }

            let ProofAccount {
                proof,
                cache,
                schedule,
            } = bytemuck::from_bytes_mut::<ProofAccount>(account_data);

            let Some(task) = schedule.next() else {
                return Err(ProgramError::Custom(3));
            };

            let mut task = Task::try_from(*task)?;
            let mut task = task.view(proof, cache);

            task.execute();

            return Err(ProgramError::Custom(42));
        }
    }

    Ok(stage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use swiftness::{parse, TransformTo};

    pub fn read_proof() -> ProofAccount {
        let small_json = include_str!("../resources/small.json");
        let stark_proof = parse(small_json).unwrap();
        let proof = stark_proof.transform_to();

        ProofAccount {
            proof,
            cache: Cache::default(),
            schedule: Schedule::from_vec(vec![Task::VerifyProof as u8]),
        }
    }

    #[test]
    fn test_deserialize_proof() {
        let mut proof_account: ProofAccount = read_proof();
        let proof_account_memory = bytemuck::bytes_of_mut(&mut proof_account);

        let mut account_data = proof_account_memory.to_vec();
        account_data.insert(0, 0);

        let res = process_instruction(
            Entrypoint::VerifyProof {},
            &mut account_data[1..],
            VerificationStage::Verify,
        );

        assert_eq!(res, Err(ProgramError::Custom(42)));

        // let output = output
        //     .into_iter()
        //     .map(|v| v.to_string())
        //     .collect::<Vec<_>>();

        // assert_eq!(
        //     program_hash.to_string(),
        //     "1134405407503728996667931466883426118808998438966777289406309056327695405399"
        // );
        // assert_eq!(output, vec!["0", "1", "5"]);
    }
}
