use swiftness::{
    funvec::{FUNVEC_QUERIES, FunVec},
    types::{Felt, StarkCommitment},
};
use swiftness_air::{
    Transcript, domains::StarkDomains, layout::recursive::global_values::InteractionElements,
};

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct Intermediate {
    verify: VerifyIntermediate,
}

unsafe impl bytemuck::Zeroable for Intermediate {}
unsafe impl bytemuck::Pod for Intermediate {}

#[derive(Clone, Copy, Default)]
#[repr(C)]
struct VerifyIntermediate {
    n_original_columns: u32,
    n_interaction_columns: u32,
    stark_domains: StarkDomains,
    transcript: Transcript,
    stark_commitment: StarkCommitment<InteractionElements>,
    queries: FunVec<Felt, FUNVEC_QUERIES>,
}
