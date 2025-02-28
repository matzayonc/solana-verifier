use swiftness::swiftness_fri::fri::fri_commit;
use swiftness::types::StarkProof;

use crate::Cache;
use crate::intermediate::Intermediate;
use crate::task::Task;
use crate::task::Tasks;

use super::StarkCommitTask;

pub struct StarkCommitFriTask<'a> {
    parent: StarkCommitTask<'a>,
}

impl Task for StarkCommitFriTask<'_> {
    // stark_commit() - last part
    fn execute(&mut self) -> Vec<Tasks> {
        let StarkCommitTask {
            intermediate,
            transcript,
            config,
            unsent_commitment,
            ..
        } = &mut self.parent;
        fri_commit(
            &mut intermediate.fri_commitment,
            transcript,
            &unsent_commitment.fri,
            &config.fri,
        );

        self.children()
    }

    fn children(&self) -> Vec<Tasks> {
        vec![]
    }
}

impl<'a> StarkCommitFriTask<'a> {
    pub fn view(
        proof: &'a mut StarkProof,
        cache: &'a mut Cache,
        intermediate: &'a mut Intermediate,
    ) -> Self {
        StarkCommitFriTask {
            parent: StarkCommitTask::view(proof, cache, intermediate),
        }
    }
}
