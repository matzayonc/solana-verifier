use swiftness::funvec::FunVec;
use swiftness::types::Felt;
use swiftness::types::StarkProof;
use swiftness_air::layout::LayoutTrait;
use swiftness_air::layout::recursive_with_poseidon::Layout;
use swiftness_air::public_memory::PublicInput;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

pub struct VerifyOutputTask<'a> {
    pub public_input: &'a PublicInput,
    pub output: &'a mut FunVec<Felt, 1024>,
    pub program_hash: &'a mut Felt,
}

impl Task for VerifyOutputTask<'_> {
    fn execute(&mut self) {
        let (program_hash, output) = Layout::verify_public_input(self.public_input).unwrap();

        *self.program_hash = program_hash;
        self.output.move_to(output);
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> VerifyOutputTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        _cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        VerifyOutputTask {
            public_input: &proof.public_input,
            output: &mut intermediate.verify_output.output,
            program_hash: &mut intermediate.verify_output.program_hash,
        }
    }
}
