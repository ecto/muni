---
name: ensemble
description: Gather perspectives from multiple AI models (Claude, Gemini, Codex) on complex problems. Launches parallel analysis agents and synthesizes their findings to identify consensus, disagreements, and blind spots.
allowed-tools: Task, Read, Glob, Grep, AskUserQuestion
user-invocable: true
---

# Multi-Model Ensemble Analysis

Orchestrate multiple AI models to analyze complex problems from different perspectives. This skill launches parallel agents to gather diverse viewpoints, then synthesizes findings into actionable recommendations.

## When to Use

- Architectural decisions with significant trade-offs
- Complex debugging where root cause is unclear
- Design reviews where you want diverse perspectives
- Technology selection decisions
- Code review for critical or safety-sensitive changes
- Any problem where multiple viewpoints add value

## Workflow

### Phase 1: Problem Identification

Extract the core problem from the conversation:

1. **Identify the decision/challenge** - What specific question needs analysis?
2. **Gather context** - Read CLAUDE.md, relevant source files, and conversation history
3. **Note constraints** - Time, resources, compatibility requirements, etc.

Summarize in 2-3 clear sentences.

### Phase 2: Create Unified Prompt

Create ONE prompt that will be sent to all models. The prompt must be:

- **Self-contained** - Include all necessary context directly (don't rely on agents having access to files)
- **Unbiased** - Do NOT pre-bias external models with your opinions
- **Structured** - Clear sections for problem, context, constraints, and what you want analyzed

**Prompt Template:**
```
## Problem
[2-3 sentence description of the decision/challenge]

## Context
[Relevant technical details, code snippets, architecture info]
[Inject content from CLAUDE.md or relevant files directly]

## Constraints
[Time, compatibility, performance, safety requirements]

## Analysis Requested
Please analyze this problem and provide:
1. Your recommended approach with rationale
2. Key risks or concerns
3. Alternative approaches considered
4. Implementation considerations
```

### Phase 3: Parallel Model Execution

Launch ALL THREE Task tools in a SINGLE response so they run in parallel:

**1. Claude Critic (Sonnet)**
```
Task tool with subagent_type: "general-purpose"
model: "sonnet"
prompt: [Your unified prompt + "Focus on critical analysis, edge cases, and potential failure modes."]
```

**2. Gemini Analysis**
```
Task tool with subagent_type: "Bash"
prompt: "Run: gemini '[Your unified prompt]' and return the full response"
```

**3. Codex Analysis**
```
Task tool with subagent_type: "Bash"
prompt: "Run: codex '[Your unified prompt]' and return the full response"
```

**Important:**
- All three must be launched in a SINGLE message with multiple tool calls
- Inject context directly into prompts - don't assume agents can read files
- Keep prompts identical for fair comparison

### Phase 4: Synthesis

After all models respond, synthesize findings:

**1. Consensus Points**
- Where do all models agree?
- These are likely strong recommendations

**2. Disagreements**
- Where do models diverge?
- Analyze WHY they disagree
- Your role: decide which perspective is stronger given your full context

**3. Blind Spots**
- What did one model catch that others missed?
- These often surface valuable edge cases

**4. Final Recommendation**
- Integrate findings into a clear recommendation
- Acknowledge trade-offs
- Be specific about next steps

## Error Handling

Models may fail due to:
- **Authentication** - Gemini/Codex may not be configured
- **Timeouts** - Long prompts may exceed limits
- **Unavailability** - External services may be down

When errors occur:
1. Document which models succeeded/failed
2. Continue analysis with available responses
3. Note if a failed model might have provided unique perspective

## Example

**User asks:** "Should we use WebSockets or SSE for real-time updates in the console?"

**Phase 1 - Problem:**
"Deciding between WebSockets and Server-Sent Events for streaming rover telemetry to the depot console. Need bidirectional communication for teleop commands but also want simplicity for status updates."

**Phase 2 - Prompt:**
```
## Problem
Choosing between WebSockets and SSE for real-time communication in a robotics fleet management console. Need to stream telemetry from rovers and send teleop commands.

## Context
- React frontend with Zustand state management
- Rust backend services using Tokio
- Current: WebSocket for teleop, considering SSE for status updates
- ~10 rovers max per depot initially
- Telemetry at 10-30Hz, commands at 50Hz

## Constraints
- Must work through corporate proxies
- Need reconnection handling
- Mobile browser support required
- Latency-sensitive for teleop

## Analysis Requested
1. Recommended approach with rationale
2. Key risks or concerns
3. Alternative approaches
4. Implementation considerations
```

**Phase 3 - Launch all three agents in parallel**

**Phase 4 - Synthesis:**
"All models agree WebSockets are necessary for teleop commands. Gemini suggested a hybrid approach: WebSocket for teleop, SSE for status monitoring. Codex raised proxy traversal concerns that weren't in the other responses. Recommendation: Keep WebSocket for all real-time communication for consistency, but implement proper reconnection handling as all models flagged this."

## Best Practices

- **Be specific** - Vague problems get vague answers
- **Include code** - When relevant, include actual code snippets in prompts
- **State constraints clearly** - Models can't guess your requirements
- **Don't over-synthesize** - Preserve disagreements when they represent genuine trade-offs
- **Act on findings** - The value is in the decision, not the analysis

## Limitations

- External models (Gemini, Codex) require CLI tools to be installed and authenticated
- Very long prompts may need to be trimmed
- Models may have different knowledge cutoffs
- Cost: Running three models is 3x the compute
