use crate::compiler::{
    AotArtifact, AotBuildPlan, AotJob, BackendKind, BuildStage, CompiledFunction,
};

pub(super) fn build_aot_plan<'a>(
    functions: impl Iterator<Item = &'a CompiledFunction>,
) -> AotBuildPlan {
    let mut jobs = Vec::new();

    for function in functions {
        if function.selected_backend != BackendKind::Native {
            continue;
        }

        if let Some(artifact) = &function.artifacts.aot {
            jobs.push(AotJob {
                function: function.name.clone(),
                artifact: AotArtifact {
                    symbol: artifact.symbol.clone(),
                    target_platforms: artifact.target_platforms.clone(),
                    stage: BuildStage::BuildTimeOnly,
                },
            });
        }
    }

    AotBuildPlan {
        stage: BuildStage::BuildTimeOnly,
        jobs,
    }
}
