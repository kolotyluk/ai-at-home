# AI@home: Distributed, Policy-Driven Edge Compute for Hybrid AI Execution

**Purpose**: NotebookLM source document for generating an Audio Overview briefing about AI@home
**Target audience**: Technical professionals interested in distributed AI infrastructure
**Customization prompt for NotebookLM**: "Explain AI@home to a technical audience familiar with cloud computing and home labs. Use the electric vehicle analogy. Keep it practical — what it is, why it matters, how it works, and what's next."
**Source**: Eric Kolotyluk, AI@home white paper (2026-05-01, 83 pages)

---

## The Problem: Your Computing Power Is Being Wasted

Here is a paradox that affects millions of technical professionals: you may own more computing power than most people — a home server, a NAS with idle GPUs, a workstation with a powerful graphics card — yet every time you ask an AI a question, your prompt travels to a data center hundreds of miles away. Your local hardware sits idle while you pay for someone else's electricity.

And if you decide to run AI locally instead? You lose the cloud's advantages — access to large models, broad knowledge, and the context that makes AI useful.

This is not a temporary inconvenience. It is a structural problem in how AI infrastructure is currently designed. Today's AI systems are built like internal combustion vehicles — they depend on centralized distribution of compute, the way cars depend on centralized distribution of gasoline. You cannot fuel up at home.

AI@home proposes a different model. Think of it as the electric vehicle equivalent for AI: an architecture where you can "charge at home" — using your own hardware for AI workloads while still connecting to the broader cloud when you need it.

---

## What AI@home Actually Is

AI@home is a hybrid architectural model that enables locally deployed computers — home servers, NAS systems, workstations, and personal machines — to participate directly in AI workload execution alongside cloud resources.

The core premise is straightforward: AI workloads should execute where they make the most sense, based on constraints like privacy, cost, latency, and available compute. AI@home provides the coordination and execution layer to make this possible.

It is built on three key elements:

1. **A control plane** responsible for policy-driven coordination, scheduling, and resource selection
2. **A set of execution nodes** capable of running containerized AI workloads across heterogeneous environments
3. **An open, vendor-neutral API** enabling interoperability between local, organizational, and cloud systems

Critically, AI@home is intentionally scoped to execution and coordination only. It does not define how AI agents should behave, what they should know, or how their quality should be governed. Those concerns belong to a separate governance layer. AI@home handles the road; governance frameworks handle the driving.

---

## The Architecture: How It Works

### Nodes and Coordinators

The system has two primary components. **Nodes** are the execution units — your home server, your workstation, your NAS. Each node provides compute resources and enforces local policy constraints. You decide what workloads your hardware will accept, what data it can access, and how much of your resources to contribute.

**Coordinators** orchestrate workloads across available nodes. They handle capability matching (does this node have the GPU power for this task?), job scheduling (which node should run this workload?), workload distribution, and result aggregation. Initially cloud-hosted, coordinators can later be self-hosted for fully decentralized deployments.

### Control Plane vs Data Plane

AI@home separates lightweight coordination logic from heavy data operations. The **control plane** handles node registration, capability reporting, workload scheduling, policy enforcement, and audit — all lightweight operations using gRPC. The **data plane** handles model distribution, input ingestion, intermediate artifacts like embeddings, and output delivery — bandwidth-intensive operations that use optimized transfer mechanisms.

This separation, borrowed from network infrastructure design, allows each plane to be optimized independently.

### The Trust Spectrum

Not everyone wants the same level of openness. AI@home supports a continuum of trust configurations:

- **Fully Permissive Compute Donor**: Your hardware runs external AI workloads in sandboxed containers with minimal restrictions. Maximum contribution, simplest setup. Think of it as the SETI@home model — you donate spare compute cycles.

- **Managed Hybrid Node**: Your hardware participates under defined constraints. You restrict which workload types are allowed, enforce resource limits, and control what data leaves your machine. You get the benefits of distributed execution with guardrails.

- **Privacy-First Local Node**: Your hardware operates primarily as a local execution environment. Sensitive data never leaves your trust boundary. Cloud interaction is limited to cases you explicitly permit. Maximum privacy, minimum external exposure.

You choose where you sit on this spectrum, and you can change your position at any time.

---

## Why This Matters: Five Key Benefits

### 1. Use What You Already Own

Substantial compute capacity sits idle in home labs, NAS systems, and workstations worldwide. AI@home lets you put that hardware to work for AI tasks instead of letting it idle while you pay cloud providers for the same capability.

### 2. Keep Sensitive Data Local

When your AI workload runs on your own hardware, your data never leaves your network. For professionals working with confidential information — legal documents, medical records, financial data — this is not a convenience feature. It is a requirement.

### 3. Reduce Latency and Cost

Local execution eliminates round-trip latency to distant data centers. For preprocessing, embedding generation, and small model inference, local execution is faster and free (you already own the hardware and electricity).

### 4. No Vendor Lock-In

AI@home is designed as an open, interoperable architecture using standardized APIs and protocol-based communication. Nodes and coordinators from different vendors can interoperate. You are not locked into any single cloud provider, hardware vendor, or AI model.

### 5. Start Small, Grow Incrementally

You do not need to build a data center. Start with a single node running a TrueNAS container. Add capabilities over time. The architecture is designed for incremental adoption — each stage is independently useful.

---

## Node Capability Tiers: What Hardware Can Do What

AI@home defines four capability tiers based on hardware:

**Tier 1 — Minimal Edge Node**: Basic CPU, less than 16GB RAM. Capable of preprocessing tasks — data preparation, filtering, embedding retrieval. The entry-level participation tier. A Raspberry Pi or basic NAS qualifies.

**Tier 2 — Standard Edge Node**: Dedicated GPU, 32-64GB RAM. Capable of small model inference — running compact language models, image classification, audio transcription. An Apple Mac Studio or mid-range workstation qualifies.

**Tier 3 — Advanced Edge Node**: Multi-GPU setup, 128GB+ RAM. Capable of medium model inference and fine-tuning. A serious home lab with NVIDIA A4000 or RTX 4090 GPUs qualifies.

**Tier 4 — Maxed-Out Edge / Micro Data Center**: High-end GPU cluster, 256GB+ RAM. Capable of large model inference and training tasks. Enterprise-grade home lab or small organizational infrastructure.

The white paper provides detailed reference implementations for two configurations: a CUDA-TrueNAS setup (NVIDIA GPU + TrueNAS storage, targeting home lab enthusiasts) and a Metal-Apple Studio setup (Apple Silicon + Metal API, targeting creative professionals).

---

## The Workload Lifecycle: From Request to Result

When an AI workload enters the system, it follows a structured lifecycle:

1. **Submit**: A user or agent submits a workload request with requirements (model type, compute needs, data constraints, latency tolerance)

2. **Match**: The coordinator evaluates available nodes against the workload's requirements — does any node have the right GPU, enough memory, and compatible policies?

3. **Schedule**: The coordinator assigns the workload to the best-matching node based on capability, current load, and policy constraints

4. **Execute**: The node runs the workload in a containerized environment, isolated from other processes and from the node's own data

5. **Return**: Results are aggregated and delivered back through the coordination pathway, with full traceability

Three workload types are supported: **inference** (running a model to generate output), **preprocessing** (data preparation, embedding generation, retrieval), and **training** (parameter updates, fine-tuning).

---

## The SETI@home and Bitcoin Mining Analogies

AI@home draws explicit parallels to three precedents for distributed computing:

**SETI@home** (1999): Over 5 million volunteers contributed idle compute cycles to search for extraterrestrial intelligence. Demonstrated that large-scale coordination of edge resources is feasible and that people will voluntarily participate.

**Folding@home** (2000): Distributed protein folding simulations across millions of personal computers. Contributed directly to COVID-19 research. Demonstrated that distributed compute can produce scientifically valuable results.

**Bitcoin Mining** (2009): Introduced proof-of-work incentive structures for compute contribution. Demonstrated that economic incentives can sustain large-scale distributed compute networks.

AI@home builds on these precedents but extends them to AI workloads — more complex, more heterogeneous, and requiring policy-driven coordination rather than simple task distribution.

---

## The Roadmap: Four Versions

AI@home follows an evolutionary deployment approach. Each version is independently useful — you do not need to wait for V4 to get value from V1.

**V1 — Permissive Node (Walking Skeleton)**: Single node, containerized execution, minimal policy. Can someone set up a container on TrueNAS, connect an AI agent to it, run basic AI work, and get results? This is the proof of concept.

**V2 — Hybrid Node**: Local and cloud execution working together. Basic policy enforcement. The node can run some workloads locally and route others to the cloud based on simple rules.

**V3 — Policy-Rich Node**: Full constraint enforcement, advanced workload routing, formal audit trails. The system enforces complex policies about what runs where, what data can leave, and what results are acceptable.

**V4 — Distributed Ecosystem (Optional)**: Multi-node coordination, federated deployment, marketplace dynamics. Multiple nodes cooperate as a network. This is the SETI@home-scale vision — but it is explicitly optional. V1-V3 are valuable without it.

---

## The Governance Question: What AI@home Does NOT Do

AI@home is deliberately silent on several critical questions:

- How should AI agents behave?
- How should their outputs be validated?
- How should knowledge persist across sessions?
- How should agent behavior be tested and verified?

These are governance questions, not infrastructure questions. The paper makes a clear architectural argument: execution infrastructure and governance infrastructure must be separate layers.

Without governance, distributed AI execution risks amplifying failure modes rather than containing them. Hallucinations propagate faster across more nodes. Inconsistent behavior becomes harder to detect at scale. Untraceable outputs multiply across heterogeneous environments.

The paper's framing: "If AI@home is the infrastructure of an AI society, then solutions such as AGET provide the governance systems that define, constrain, and validate how its agents behave."

This is not a limitation of AI@home — it is a deliberate design decision. By staying focused on execution infrastructure and leaving governance to specialized frameworks, AI@home avoids the complexity trap of trying to solve everything in one system.

---

## Open Questions and Challenges

The paper honestly identifies several unresolved challenges:

**Training Feasibility**: Can distributed edge nodes meaningfully participate in model training, or is training fundamentally a centralized activity? The compute, memory, and bandwidth requirements for training are orders of magnitude higher than inference.

**Energy Implications**: What happens when thousands of home GPUs start running AI workloads? Residential power consumption could become a regulatory and environmental concern.

**Regulatory Uncertainty**: Data residency laws vary by jurisdiction. When a workload routes from a Canadian node to an American cloud, which privacy rules apply? The legal framework for compute donation is undefined.

**Security**: Malicious nodes could return poisoned results, exfiltrate input data, or consume resources without producing useful output. The trust model must evolve alongside the deployment model.

**Economic Viability**: Will enough people participate voluntarily, or does the system need formal incentive structures? The SETI@home model worked for altruistic science; AI workloads may require economic incentives.

---

## The Bottom Line

AI@home proposes that the current model of AI deployment — everything in the cloud, nothing at the edge — is not the only option and not the best option. Substantial compute capacity exists at the edge. Privacy-sensitive workloads should not be forced to the cloud. And the coordination technology to make hybrid execution work is within reach.

The architecture is practical, not theoretical. It builds on established technology (containers, gRPC, REST APIs) and targets existing communities (home lab operators, TrueNAS users) with existing hardware.

Whether AI@home becomes the SETI@home of artificial intelligence depends on whether the execution-governance separation holds, whether the incentive structures emerge, and whether the early adopter community finds enough value to sustain participation through the inevitable rough edges of V1.

The paper is an invitation to find out.

---

## How to Use This Document with NotebookLM

1. Upload this document to NotebookLM as the primary source
2. Optionally also upload the AI@home white paper PDF for deeper context
3. Use the customization prompt: "Explain AI@home to a technical audience familiar with cloud computing and home labs. Use the electric vehicle analogy. Keep it practical — what it is, why it matters, how it works, and what's next."
4. Generate an Audio Overview

The resulting audio briefing should cover: the centralized compute problem, the EV analogy, the hybrid architecture, trust spectrum, node tiers, workload lifecycle, SETI@home parallels, the 4-version roadmap, the governance separation, and open challenges — in approximately 12-18 minutes of conversational audio.
