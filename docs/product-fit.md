# Product Fit And Launch Readiness

This document gives a direct answer about who should use Vogon Runtime, what it
does better than a one-off script, where established alternatives are stronger,
and how the project should be launched. The comparison was reviewed on
2026-07-30 against the linked official product documentation.

## The Definite Answer

Vogon is ready to launch as an **open-source beta for supervised CI release
gates**. It is not ready to be sold as an enterprise control plane, a production
agent orchestrator, or a replacement for deterministic security and policy
tools.

The narrow launch position is:

> A local, auditable AI release gate for Git changes.

The best initial users are platform, release, DevOps, and AI platform engineers
at software companies that:

- already use pull request CI;
- have recurring changes that need contextual review across several files;
- are permitted to send the selected diff to an approved model provider;
- want a local replay artifact instead of depending on a hosted evaluation UI;
- will keep a human reviewer accountable for the final release decision.

The initial buyer or internal champion is a staff-level platform engineer or AI
platform lead. The daily user is the engineer who owns CI policy and reviews
blocked releases.

Vogon should not initially target teams that need a general agent framework,
production tracing, prompt benchmarking across datasets, automated red teaming,
SSO or RBAC, or cryptographically signed compliance evidence.

## The Company Workflow

The concrete launch example is a pull request that changes a payments service
deployment configuration:

1. Existing tests, static analysis, policy checks, and secret scanning run
   first.
2. Vogon collects the trusted-base Git diff directly from the checked-out
   repository.
3. A reviewed workflow asks an approved NVIDIA model to identify deployment
   risks, then requires a strict final JSON `GO` or `NO_GO` decision.
4. The provider and model allowlists are checked before the first request.
5. A `NO_GO` decision writes replay evidence and fails the CI job.
6. The job renders a self-contained HTML report and uploads both artifacts for
   the responsible human reviewer.
7. A later verification run can compare exact cached output or rerun the model
   and compare workflow structure.

The authorized acceptance run against the synthetic example blocked a candidate
that allowed zero healthy instances, disabled health checks, removed the request
timeout, and disabled rollback. The report preserved four grounded reasons and
four required remediations:

![Vogon release gate report showing a NO_GO decision and grounded deployment risks](images/company-release-gate-report.png)

The full runnable setup is in the
[company release gate example](../examples/company-release-gate/README.md).
This is evidence that the integration works end to end. It is not evidence of a
measured model catch rate across real production incidents.

## The Problem It Solves

Teams can build this gate with shell, `curl`, and `jq`. Vogon does not make an
otherwise impossible workflow possible. It packages the repeated, failure-prone
parts into a reviewed format:

| Repeated problem | Typical one-off implementation | Vogon behavior |
| --- | --- | --- |
| Feed repository changes to a model | CI-specific Git and quoting script | Native working-tree or trusted-base Git diff input |
| Reuse the same review policy | Prompt embedded in shell or application code | Checked-in TOML workflow with strict named inputs |
| Prevent accidental provider drift | Environment convention | Provider and model allowlists checked before egress |
| Turn model text into a CI result | Ad hoc `jq` expression | Fail-closed JSON Pointer decision policy |
| Explain a blocked run later | Console log or provider dashboard | Local replay plus offline HTML evidence report |
| Detect workflow drift | Manual comparison | Exact and output-insensitive structural verification |
| Avoid paying twice during review | Custom cache plumbing | Bounded provider-output cache |

The value is operational consistency and reviewability, not better model
intelligence. A poor prompt or unsuitable model remains poor when run through
Vogon.

## Alternatives

### Promptfoo

[Promptfoo](https://www.promptfoo.dev/docs/intro/) is the closest practical
alternative. It is an open-source local CLI and library with declarative tests,
caching, metrics, broad provider support, a web viewer, and red teaming. Its
[CI/CD integration](https://www.promptfoo.dev/docs/integrations/ci-cd/) can emit
JSON, HTML, and JUnit reports and enforce quality gates.

Choose Promptfoo when the primary job is prompt, model, RAG, or security
evaluation across test cases. It is broader and more mature for evaluation and
red teaming than Vogon.

Choose Vogon only when the primary job is an ordered Git-change decision gate
and the team values a small Rust binary, strict decision evidence, execution
policy binding, and replay verification. Promptfoo can be customized to cover
much of this workflow, so Vogon's advantage is a narrower default, not exclusive
capability.

### LangSmith

[LangSmith Evaluation](https://docs.langchain.com/langsmith/evaluation)
supports datasets, offline experiments, online evaluators, human review, and
production feedback loops. Its
[observability model](https://docs.langchain.com/langsmith/observability-concepts)
records and analyzes application traces.

Choose LangSmith for a shared platform that connects development evaluation,
production tracing, datasets, and team analysis. It is substantially stronger
than Vogon for application observability and collaborative evaluation. Choose
Vogon when a local CI artifact and no mandatory hosted control plane are more
important than those platform capabilities.

### Braintrust

[Braintrust Evaluation](https://www.braintrust.dev/docs/evaluate) supports
playgrounds, immutable experiments, CI evaluation, datasets, scorers, and
online scoring. Its [observability tools](https://www.braintrust.dev/docs/observe)
turn production requests into searchable traces.

Choose Braintrust to measure AI application quality over time and connect
production traces back to evaluation datasets. It is substantially stronger
than Vogon for experiments, collaboration, and production monitoring. Vogon is
the smaller option for a repository-local release decision.

### LangGraph

[LangGraph](https://docs.langchain.com/oss/python/langgraph/overview) is a
low-level runtime for long-running, stateful agents with durable execution,
persistence, streaming, and human-in-the-loop control.

Choose LangGraph to build the actual agent or stateful application. Vogon's
linear workflows, CI decision policies, and replay evidence are not a
replacement for agent state, branching, tool calls, or pause-and-resume
execution.

### Temporal

[Temporal](https://temporal.io/) is a general durable execution platform with
persistent workflow state, retries, task queues, signals, and timers.

Choose Temporal for production business workflows that must survive process and
infrastructure failures. Vogon does not provide distributed scheduling,
long-running state recovery, or operational workflow visibility. Using Temporal
for a two-step pull request review would usually add unnecessary infrastructure;
using Vogon for a payment or order workflow would be the wrong architecture.

### A Small CI Script

Choose a shell, Python, or TypeScript script when there is one repository, one
provider, one prompt, and no recurring replay or policy requirement. That is the
lowest-complexity solution.

Adopt Vogon only after the script starts duplicating input collection, provider
restrictions, strict parsing, cache handling, replay comparison, and evidence
rendering. Replacing a clear 30-line script with a new dependency is not a win.

## Safety Boundary

The current product must be deployed with these conditions:

- Run deterministic tests, policy-as-code, static analysis, and secret scanning
  before Vogon. A model decision is a second opinion, not the only hard gate.
- Review the workflow prompt and pin the Vogon release, provider, and model in
  CI policy.
- Send a diff only when the selected provider is approved for its contents.
  Vogon does not currently filter diff paths or detect secrets inside a diff.
- Treat caches, replay files, reports, and CI artifacts as potentially
  sensitive. Configure the CI system's retention and access accordingly.
- Require human review for blocked and allowed high-impact releases.
- Do not describe replay hashes as signatures or attestations. They detect
  accidental or untrusted-file modification during local validation but do not
  prove who created an artifact.

[GitHub Actions artifacts](https://docs.github.com/en/actions/concepts/workflows-and-actions/workflow-artifacts)
can retain the replay and report after a job. Their retention and access
controls remain the repository owner's responsibility.

## Launch Verdict

| Question | Current answer |
| --- | --- |
| Can a new user run it in a separate Git repository? | Yes, covered by an automated external-repository acceptance test. |
| Does a real provider path work? | Yes, demonstrated with an authorized NVIDIA run. |
| Can it fail CI on a strict decision? | Yes, while preserving replay evidence. |
| Can a reviewer inspect the result without a service? | Yes, through the offline HTML report. |
| Are offline verification and project tests reproducible? | Yes, with deterministic adapters, cache reuse, and exact or structural modes. |
| Is the evidence cryptographically attributable? | No. |
| Does Vogon minimize or classify diff data before egress? | No. |
| Does it provide enterprise identity, policy administration, or retention? | No. |
| Does it measure model quality over datasets or production traffic? | No. |
| Is it a durable production agent runtime? | No. |

The honest release label is **open-source beta**, not enterprise GA. The beta is
useful for a supervised release-gate workflow today. The product is not
"finished" in the broader sense implied by its runtime name.

The next evidence needed is adoption evidence, not more generic runtime surface:
three independent teams should install the released binary, add the gate to a
real repository, and report whether it found actionable issues without creating
unacceptable noise, latency, cost, or data-governance risk. Until then, the
project has technical acceptance evidence but no validated product-market fit.
