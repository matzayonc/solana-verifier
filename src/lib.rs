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

pub const PROGRAM_ID: &str = "ANH87aBZFKHhB3aLAndnp8cJd8QNL58buSeLCtVb1ukj";

#[repr(u8)]
#[derive(Serialize, Deserialize)]
pub enum Entrypoint<'a> {
    PublishFragment { offset: usize, data: &'a [u8] },
    Schedule,
    VerifyProof,
}

#[derive(Clone, Copy, Default, Zeroable, Pod, Debug, PartialEq)]
#[repr(C)]
pub struct ProofAccount {
    pub proof: StarkProof,                 // The proof to verify.
    pub cache: Cache,                      // Inner-task data.
    pub intermediate: Intermediate, // Values calculated while proving, and used for subsequent tasks.
    pub schedule: Schedule<RawTask, 1000>, // Tasks remaining to be executed.
}

#[derive(Debug, Clone, Copy, Default, Zeroable, Pod, PartialEq)]
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
pub fn process_instruction(
    instruction: Entrypoint<'_>,
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
            if stage == VerificationStage::Verified {
                return Err(ProgramError::Custom(32));
            }

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
            let task_name = format!("{:?}", task);
            msg!("Executing task: {}", task_name);

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

    // pub fn read_proof() -> ProofAccount {
    //     let small_json = include_str!("../resources/saya.json");
    //     let stark_proof = parse(small_json).unwrap();
    //     let proof = stark_proof.transform_to();

    //     ProofAccount {
    //         proof,
    //         ..Default::default()
    //     }
    // }

    pub fn read_proof_from_file() -> Vec<u8> {
        let account_data = include_bytes!("../resources/proof.bin").to_vec();
        let account = bytemuck::from_bytes::<ProofAccount>(&account_data);
        bytemuck::bytes_of(account).to_vec()
    }

    #[ignore]
    #[test]
    pub fn prepare_account() {
        let small_json = include_str!("../resources/saya.json");
        let stark_proof = parse(small_json).unwrap();
        let proof = stark_proof.transform_to();

        let proof_account = ProofAccount {
            proof,
            ..Default::default()
        };
        let account_data = bytemuck::bytes_of(&proof_account);

        let account_data_path = "resources/proof.bin";

        std::fs::write(account_data_path, account_data).unwrap();
        let account_data = std::fs::read(account_data_path).unwrap();
        let read_proof_account = bytemuck::from_bytes::<ProofAccount>(&account_data);

        assert_eq!(&proof_account, read_proof_account);
    }

    #[test]
    fn test_deserialize_proof() {
        let account_data = &mut read_proof_from_file()[..];

        let mut stage = VerificationStage::Publish;

        stage = process_instruction(Entrypoint::Schedule, account_data, stage).unwrap();
        let mut c = 0;

        while stage != VerificationStage::Verified {
            stage = process_instruction(Entrypoint::VerifyProof, account_data, stage).unwrap();
            c += 1;
        }

        assert_eq!(c, 8);

        let ProofAccount { intermediate, .. } = bytemuck::from_bytes::<ProofAccount>(account_data);

        assert_eq!(
            intermediate.program_hash().to_string(),
            "2600195635685626119055100741094371725887213141003183770434823435664529167464"
        );
        assert_eq!(
            format!("{:?}", intermediate.output()),
            "[0x1, 0x4, 0x193641eb151b0f41674641089952e60bc3aded26e3cf42793655c562b8c3aa0, 0x5ab580b04e3532b6b18f81cfa654a05e29dd8e2352d88df1e765a84072db07, 0xb2c58e4eec9b5a8f0c5ba4d15ae59c8ac8a8d96fca443dd591296ba3391aaf]"
        );
    }
}
