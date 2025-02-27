use swiftness::{
    funvec::{FUNVEC_QUERIES, FunVec},
    types::{Felt, StarkCommitment},
};
use swiftness_air::{Transcript, domains::StarkDomains};

use crate::verify::{stark_commit::StarkCommitIntermediate, stark_verify::StarkVerifyIntermediate};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct Intermediate {
    pub verify: VerifyIntermediate,
    pub verify_output: VerifyOutputIntermediate,
    pub stark_commit: StarkCommitIntermediate,
    pub stark_verify: StarkVerifyIntermediate,
}

impl Intermediate {
    pub fn output(&self) -> &[Felt] {
        self.verify_output.output.as_slice()
    }

    pub fn program_hash(&self) -> Felt {
        self.verify_output.program_hash
    }
}

unsafe impl bytemuck::Zeroable for Intermediate {}
unsafe impl bytemuck::Pod for Intermediate {}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct VerifyIntermediate {
    pub n_original_columns: u32,
    pub n_interaction_columns: u32,
    pub stark_domains: StarkDomains,
    pub transcript: Transcript,
    pub stark_commitment: StarkCommitment,
    pub queries: FunVec<Felt, FUNVEC_QUERIES>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct VerifyOutputIntermediate {
    pub output: FunVec<Felt, 1024>,
    pub program_hash: Felt,
}
