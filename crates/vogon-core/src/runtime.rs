use crate::{
    CURRENT_REPLAY_SCHEMA_VERSION, RedactionSet, ReplayMismatch, Result, RunCache, RunReport,
    RuntimeEvent, RuntimeMetadata, Step, StepResult, VerificationReport, Workflow, stable_hash,
};

/// Adapter trait implemented by model providers.
pub trait ModelAdapter {
    /// Completes one workflow step from the step metadata and assembled input.
    fn complete(&self, step: &Step, input: &str) -> Result<String>;

    /// Returns non-secret adapter configuration that scopes runtime cache keys.
    ///
    /// Adapters that can target different providers, models, endpoints, or
    /// behavior-affecting options should override this value. The runtime hashes
    /// this identity before using it as cache key material, but implementations
    /// must still avoid including credentials or other secrets.
    fn cache_identity(&self) -> String {
        std::any::type_name::<Self>().to_owned()
    }

    /// Returns non-secret runtime metadata recorded in replay reports.
    ///
    /// Adapter implementations should override this with structured provider,
    /// model, adapter version, and runtime parameter details whenever those
    /// values are known.
    fn runtime_metadata(&self) -> RuntimeMetadata {
        RuntimeMetadata::new(
            "custom",
            std::any::type_name::<Self>(),
            "unknown",
            self.cache_identity(),
        )
    }
}

#[derive(Debug, Clone)]
/// Workflow runtime backed by a model adapter.
pub struct Runtime<A> {
    adapter: A,
}

impl<A> Runtime<A>
where
    A: ModelAdapter,
{
    /// Creates a runtime from a model adapter.
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    /// Runs a workflow without cache, redactions, or event observation.
    pub fn run(&self, workflow: &Workflow) -> Result<RunReport> {
        self.run_uncached_with_redactions_and_observer(workflow, &RedactionSet::empty(), |_| {})
    }

    /// Runs a workflow and emits runtime events to an observer.
    pub fn run_with_observer<F>(&self, workflow: &Workflow, mut observer: F) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.run_uncached_with_redactions_and_observer(
            workflow,
            &RedactionSet::empty(),
            &mut observer,
        )
    }

    /// Runs a workflow with a cache scoped by adapter identity and step input hash.
    pub fn run_with_cache(&self, workflow: &Workflow, cache: &mut RunCache) -> Result<RunReport> {
        self.run_with_cache_redactions_and_observer(workflow, cache, &RedactionSet::empty(), |_| {})
    }

    /// Runs a workflow with cache and event observation.
    pub fn run_with_cache_and_observer<F>(
        &self,
        workflow: &Workflow,
        cache: &mut RunCache,
        mut observer: F,
    ) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.run_with_cache_redactions_and_observer(
            workflow,
            cache,
            &RedactionSet::empty(),
            &mut observer,
        )
    }

    /// Runs a workflow and redacts matching literals from recorded outputs.
    pub fn run_with_redactions(
        &self,
        workflow: &Workflow,
        redactions: &RedactionSet,
    ) -> Result<RunReport> {
        self.run_uncached_with_redactions_and_observer(workflow, redactions, |_| {})
    }

    /// Runs a workflow with both cache and output redactions.
    pub fn run_with_cache_and_redactions(
        &self,
        workflow: &Workflow,
        cache: &mut RunCache,
        redactions: &RedactionSet,
    ) -> Result<RunReport> {
        self.run_with_cache_redactions_and_observer(workflow, cache, redactions, |_| {})
    }

    /// Runs a workflow with redactions and event observation.
    pub fn run_with_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        redactions: &RedactionSet,
        observer: F,
    ) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.run_uncached_with_redactions_and_observer(workflow, redactions, observer)
    }

    /// Runs a workflow with cache, redactions, and event observation.
    pub fn run_with_cache_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        cache: &mut RunCache,
        redactions: &RedactionSet,
        observer: F,
    ) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.run_internal(workflow, redactions, Some(cache), observer)
    }

    fn run_uncached_with_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        redactions: &RedactionSet,
        observer: F,
    ) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.run_internal(workflow, redactions, None, observer)
    }

    fn run_internal<F>(
        &self,
        workflow: &Workflow,
        redactions: &RedactionSet,
        mut cache: Option<&mut RunCache>,
        mut observer: F,
    ) -> Result<RunReport>
    where
        F: FnMut(RuntimeEvent),
    {
        workflow.validate()?;

        let mut previous_output = String::new();
        let mut steps = Vec::with_capacity(workflow.steps().len());

        for step in workflow.steps() {
            observer(RuntimeEvent::StepStarted {
                step_id: step.id().clone(),
            });

            let input = step_input(step, &previous_output);
            let input_hash = stable_hash(&input);
            let cache_key = self.cache_key(&input_hash);
            let output = match cache.as_deref_mut().and_then(|cache| {
                cache
                    .get_output(&cache_key)
                    .map(std::borrow::ToOwned::to_owned)
            }) {
                Some(output) => output,
                None => {
                    let output = self.adapter.complete(step, &input)?;
                    if let Some(cache) = cache.as_deref_mut() {
                        cache.insert_output(cache_key, output.clone());
                    }
                    output
                }
            };
            let redacted_output = redactions.redact(&output);

            steps.push(StepResult {
                step_id: step.id().clone(),
                input_hash,
                output_hash: stable_hash(&redacted_output),
                output: redacted_output,
            });

            previous_output = output;

            observer(RuntimeEvent::StepFinished {
                step_id: step.id().clone(),
            });
        }

        let run_hash_material = steps
            .iter()
            .map(|step| {
                format!(
                    "{}:{}:{}",
                    step.step_id.as_str(),
                    step.input_hash,
                    step.output_hash
                )
            })
            .collect::<Vec<_>>()
            .join("|");

        Ok(RunReport {
            schema_version: CURRENT_REPLAY_SCHEMA_VERSION,
            workflow_name: workflow.name().to_owned(),
            runtime: self.adapter.runtime_metadata(),
            run_hash: stable_hash(run_hash_material),
            steps,
        })
    }

    /// Verifies a workflow against an expected replay without cache or redactions.
    pub fn verify(&self, workflow: &Workflow, expected: &RunReport) -> Result<VerificationReport> {
        self.verify_uncached_with_redactions_and_observer(
            workflow,
            expected,
            &RedactionSet::empty(),
            |_| {},
        )
    }

    /// Verifies a workflow and emits runtime events to an observer.
    pub fn verify_with_observer<F>(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        mut observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.verify_uncached_with_redactions_and_observer(
            workflow,
            expected,
            &RedactionSet::empty(),
            &mut observer,
        )
    }

    /// Verifies a workflow against a replay while applying output redactions.
    pub fn verify_with_redactions(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        redactions: &RedactionSet,
    ) -> Result<VerificationReport> {
        self.verify_uncached_with_redactions_and_observer(workflow, expected, redactions, |_| {})
    }

    /// Verifies a workflow using a cache scoped by adapter identity and step input hash.
    pub fn verify_with_cache(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        cache: &mut RunCache,
    ) -> Result<VerificationReport> {
        self.verify_with_cache_redactions_and_observer(
            workflow,
            expected,
            cache,
            &RedactionSet::empty(),
            |_| {},
        )
    }

    /// Verifies a workflow using both cache and output redactions.
    pub fn verify_with_cache_and_redactions(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        cache: &mut RunCache,
        redactions: &RedactionSet,
    ) -> Result<VerificationReport> {
        self.verify_with_cache_redactions_and_observer(
            workflow,
            expected,
            cache,
            redactions,
            |_| {},
        )
    }

    /// Verifies a workflow with redactions and event observation.
    pub fn verify_with_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        redactions: &RedactionSet,
        observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        self.verify_uncached_with_redactions_and_observer(workflow, expected, redactions, observer)
    }

    /// Verifies a workflow with cache, redactions, and event observation.
    pub fn verify_with_cache_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        cache: &mut RunCache,
        redactions: &RedactionSet,
        mut observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        let actual = self.run_with_cache_redactions_and_observer(
            workflow,
            cache,
            redactions,
            &mut observer,
        )?;
        self.compare_reports(expected, actual, observer)
    }

    fn verify_uncached_with_redactions_and_observer<F>(
        &self,
        workflow: &Workflow,
        expected: &RunReport,
        redactions: &RedactionSet,
        mut observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        let actual =
            self.run_uncached_with_redactions_and_observer(workflow, redactions, &mut observer)?;
        self.compare_reports(expected, actual, observer)
    }

    fn compare_reports<F>(
        &self,
        expected: &RunReport,
        actual: RunReport,
        mut observer: F,
    ) -> Result<VerificationReport>
    where
        F: FnMut(RuntimeEvent),
    {
        let mut mismatches = Vec::new();

        if expected.workflow_name != actual.workflow_name {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::WorkflowName {
                    expected: expected.workflow_name.clone(),
                    actual: actual.workflow_name.clone(),
                },
                &mut observer,
            );
        }

        if expected.run_hash != actual.run_hash {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::RunHash {
                    expected: expected.run_hash.clone(),
                    actual: actual.run_hash.clone(),
                },
                &mut observer,
            );
        }

        if expected.schema_version == CURRENT_REPLAY_SCHEMA_VERSION
            && expected.runtime != actual.runtime
        {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::RuntimeMetadata {
                    expected: Box::new(expected.runtime.clone()),
                    actual: Box::new(actual.runtime.clone()),
                },
                &mut observer,
            );
        }

        if expected.steps.len() != actual.steps.len() {
            push_mismatch(
                &mut mismatches,
                ReplayMismatch::StepCount {
                    expected: expected.steps.len(),
                    actual: actual.steps.len(),
                },
                &mut observer,
            );
        }

        for (index, (expected_step, actual_step)) in
            expected.steps.iter().zip(actual.steps.iter()).enumerate()
        {
            if expected_step.step_id != actual_step.step_id {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepId {
                        index,
                        expected: expected_step.step_id.clone(),
                        actual: actual_step.step_id.clone(),
                    },
                    &mut observer,
                );
            }

            if expected_step.input_hash != actual_step.input_hash {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepInputHash {
                        step_id: actual_step.step_id.clone(),
                        expected: expected_step.input_hash.clone(),
                        actual: actual_step.input_hash.clone(),
                    },
                    &mut observer,
                );
            }

            if expected_step.output_hash != actual_step.output_hash {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepOutputHash {
                        step_id: actual_step.step_id.clone(),
                        expected: expected_step.output_hash.clone(),
                        actual: actual_step.output_hash.clone(),
                    },
                    &mut observer,
                );
            }

            if expected_step.output != actual_step.output {
                push_mismatch(
                    &mut mismatches,
                    ReplayMismatch::StepOutput {
                        step_id: actual_step.step_id.clone(),
                        expected: expected_step.output.clone(),
                        actual: actual_step.output.clone(),
                    },
                    &mut observer,
                );
            }
        }

        Ok(VerificationReport {
            workflow_name: actual.workflow_name,
            mismatches,
        })
    }

    fn cache_key(&self, input_hash: &str) -> String {
        stable_hash(format!(
            "adapter={}\ninput_hash={input_hash}",
            self.adapter.cache_identity()
        ))
    }
}

fn push_mismatch<F>(
    mismatches: &mut Vec<ReplayMismatch>,
    mismatch: ReplayMismatch,
    observer: &mut F,
) where
    F: FnMut(RuntimeEvent),
{
    observer(RuntimeEvent::ReplayMismatch {
        step_id: mismatch.step_id().cloned(),
    });
    mismatches.push(mismatch);
}

fn step_input(step: &Step, previous_output: &str) -> String {
    if previous_output.is_empty() {
        step.prompt().to_owned()
    } else {
        format!("{}\n\nPrevious output:\n{}", step.prompt(), previous_output)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RedactionRule, RedactionSet, ReplayMismatch, Result, RunCache, RuntimeEvent, Step, StepId,
        Workflow, stable_hash,
    };

    use std::{cell::Cell, rc::Rc};

    use super::{ModelAdapter, Runtime};

    #[derive(Debug, Clone)]
    struct TestModel;

    impl ModelAdapter for TestModel {
        fn complete(&self, step: &Step, input: &str) -> Result<String> {
            Ok(format!("{}:{input}", step.id().as_str()))
        }
    }

    #[derive(Debug, Clone)]
    struct SecretModel;

    impl ModelAdapter for SecretModel {
        fn complete(&self, _step: &Step, _input: &str) -> Result<String> {
            Ok("token=sk-test-123".to_owned())
        }
    }

    #[derive(Debug, Clone)]
    struct CountingModel {
        calls: Rc<Cell<usize>>,
    }

    impl CountingModel {
        fn new(calls: Rc<Cell<usize>>) -> Self {
            Self { calls }
        }
    }

    impl ModelAdapter for CountingModel {
        fn complete(&self, step: &Step, input: &str) -> Result<String> {
            self.calls.set(self.calls.get() + 1);
            Ok(format!("{}:{input}", step.id().as_str()))
        }
    }

    #[derive(Debug, Clone)]
    struct NamespacedModel {
        namespace: &'static str,
        output: &'static str,
        calls: Rc<Cell<usize>>,
    }

    impl NamespacedModel {
        fn new(namespace: &'static str, output: &'static str, calls: Rc<Cell<usize>>) -> Self {
            Self {
                namespace,
                output,
                calls,
            }
        }
    }

    impl ModelAdapter for NamespacedModel {
        fn complete(&self, _step: &Step, _input: &str) -> Result<String> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.output.to_owned())
        }

        fn cache_identity(&self) -> String {
            self.namespace.to_owned()
        }
    }

    #[test]
    fn runtime_produces_a_report_for_ordered_steps() {
        let workflow = Workflow::new(
            "demo",
            vec![
                Step::new(StepId::new("first").unwrap(), "hello"),
                Step::new(StepId::new("second").unwrap(), "world"),
            ],
        )
        .unwrap();

        let report = Runtime::new(TestModel).run(&workflow).unwrap();

        assert_eq!(report.workflow_name, "demo");
        assert_eq!(report.steps.len(), 2);
        assert_ne!(report.run_hash, "");
    }

    #[test]
    fn run_with_observer_emits_step_events() {
        let workflow = Workflow::new(
            "demo",
            vec![
                Step::new(StepId::new("first").unwrap(), "hello"),
                Step::new(StepId::new("second").unwrap(), "world"),
            ],
        )
        .unwrap();
        let mut events = Vec::new();

        Runtime::new(TestModel)
            .run_with_observer(&workflow, |event| events.push(event))
            .unwrap();

        assert_eq!(
            events,
            vec![
                RuntimeEvent::StepStarted {
                    step_id: StepId::new("first").unwrap()
                },
                RuntimeEvent::StepFinished {
                    step_id: StepId::new("first").unwrap()
                },
                RuntimeEvent::StepStarted {
                    step_id: StepId::new("second").unwrap()
                },
                RuntimeEvent::StepFinished {
                    step_id: StepId::new("second").unwrap()
                },
            ]
        );
    }

    #[test]
    fn verify_accepts_matching_replay() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let replay = runtime.run(&workflow).unwrap();

        let verification = runtime.verify(&workflow, &replay).unwrap();

        assert!(verification.is_match());
    }

    #[test]
    fn verify_reports_replay_mismatch() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let mut replay = runtime.run(&workflow).unwrap();
        replay.steps[0].output = "changed".to_owned();

        let verification = runtime.verify(&workflow, &replay).unwrap();

        assert!(!verification.is_match());
    }

    #[test]
    fn verify_reports_runtime_metadata_mismatch_for_current_replays() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let mut replay = runtime.run(&workflow).unwrap();
        replay.runtime.provider = "other-provider".to_owned();

        let verification = runtime.verify(&workflow, &replay).unwrap();

        assert_eq!(
            verification.mismatches,
            vec![ReplayMismatch::RuntimeMetadata {
                expected: Box::new(replay.runtime),
                actual: Box::new(runtime.run(&workflow).unwrap().runtime),
            }]
        );
    }

    #[test]
    fn verify_with_observer_emits_replay_mismatch_events() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(TestModel);
        let mut replay = runtime.run(&workflow).unwrap();
        replay.steps[0].output = "changed".to_owned();
        let mut events = Vec::new();

        runtime
            .verify_with_observer(&workflow, &replay, |event| events.push(event))
            .unwrap();

        assert!(events.contains(&RuntimeEvent::ReplayMismatch {
            step_id: Some(StepId::new("first").unwrap())
        }));
    }

    #[test]
    fn run_with_redactions_scrubs_step_outputs() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]).unwrap();

        let report = Runtime::new(SecretModel)
            .run_with_redactions(&workflow, &redactions)
            .unwrap();

        assert_eq!(report.steps[0].output, "token=[REDACTED:api_key]");
        assert_eq!(
            report.steps[0].output_hash,
            stable_hash("token=[REDACTED:api_key]")
        );
    }

    #[test]
    fn verify_with_redactions_accepts_redacted_replay() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let runtime = Runtime::new(SecretModel);
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]).unwrap();
        let replay = runtime.run_with_redactions(&workflow, &redactions).unwrap();

        let verification = runtime
            .verify_with_redactions(&workflow, &replay, &redactions)
            .unwrap();

        assert!(verification.is_match());
    }

    #[test]
    fn run_with_cache_reuses_outputs_by_input_hash() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let calls = Rc::new(Cell::new(0));
        let runtime = Runtime::new(CountingModel::new(Rc::clone(&calls)));
        let mut cache = RunCache::new();

        let first = runtime.run_with_cache(&workflow, &mut cache).unwrap();
        let second = runtime.run_with_cache(&workflow, &mut cache).unwrap();

        assert_eq!(first, second);
        assert_eq!(calls.get(), 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn run_with_cache_scopes_entries_by_adapter_identity() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let first_calls = Rc::new(Cell::new(0));
        let second_calls = Rc::new(Cell::new(0));
        let first_runtime = Runtime::new(NamespacedModel::new(
            "provider=a;model=one",
            "first output",
            Rc::clone(&first_calls),
        ));
        let second_runtime = Runtime::new(NamespacedModel::new(
            "provider=b;model=two",
            "second output",
            Rc::clone(&second_calls),
        ));
        let mut cache = RunCache::new();

        let first = first_runtime.run_with_cache(&workflow, &mut cache).unwrap();
        let second = second_runtime
            .run_with_cache(&workflow, &mut cache)
            .unwrap();

        assert_eq!(first.steps[0].output, "first output");
        assert_eq!(second.steps[0].output, "second output");
        assert_eq!(first_calls.get(), 1);
        assert_eq!(second_calls.get(), 1);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn run_with_cache_applies_current_redactions_to_cached_outputs() {
        let workflow = Workflow::new(
            "demo",
            vec![Step::new(StepId::new("first").unwrap(), "hello")],
        )
        .unwrap();
        let redactions =
            RedactionSet::new(vec![RedactionRule::new("api_key", "sk-test-123").unwrap()]).unwrap();
        let mut cache = RunCache::new();
        let runtime = Runtime::new(SecretModel);

        let unredacted = runtime.run_with_cache(&workflow, &mut cache).unwrap();
        let redacted = runtime
            .run_with_cache_and_redactions(&workflow, &mut cache, &redactions)
            .unwrap();

        assert_eq!(unredacted.steps[0].output, "token=sk-test-123");
        assert_eq!(redacted.steps[0].output, "token=[REDACTED:api_key]");
        assert_eq!(cache.len(), 1);
    }
}
