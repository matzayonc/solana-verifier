use swiftness::{
    funvec::{FUNVEC_QUERIES, FunVec},
    types::{Felt, StarkCommitment},
};
use swiftness_air::{
    Transcript, domains::StarkDomains, layout::recursive::global_values::InteractionElements,
};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Intermediate {
    pub verify: VerifyIntermediate,
}

unsafe impl bytemuck::Zeroable for Intermediate {}
unsafe impl bytemuck::Pod for Intermediate {}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VerifyIntermediate {
    pub n_original_columns: u32,
    pub n_interaction_columns: u32,
    pub stark_domains: StarkDomains,
    pub transcript: Transcript,
    pub stark_commitment: StarkCommitment<InteractionElements>,
    pub queries: FunVec<Felt, FUNVEC_QUERIES>,
}
