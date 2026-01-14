# Skill Templates

This document provides templates for creating different types of skills.

## Template 1: Code Review Skill

Use this template for skills that review code for correctness, safety, and adherence to patterns.

```markdown
---
name: [component-name]-review
description: Reviews [Language] code for [Component Name] with focus on [key area 1], [key area 2], and [key area 3]. Use when reviewing [component] changes, debugging [common issues], validating [critical patterns], or evaluating [architecture decisions]. Covers [specific pattern 1], [specific pattern 2], and [specific pattern 3].
allowed-tools: Read, Grep, Glob, Bash(build-command), Bash(test-command)
---

# [Component Name] Review Skill

This skill provides comprehensive code review for [Component Name], focusing on [primary focus areas].

## Overview

[Component description - what it does, why it exists]

**Key Architecture:**
- **Platform**: [OS/runtime environment]
- **Language**: [Primary language and version]
- **Framework**: [Key frameworks or libraries]
- **Communication**: [Protocols, APIs, interfaces]
- **Location**: [Repository path]

**Critical Files:**
- [File 1]: [Path and purpose]
- [File 2]: [Path and purpose]
- [File 3]: [Path and purpose]

## Review Checklist

### 1. [Critical Category Name]

**Purpose**: [Why this matters - safety, correctness, performance]

**Location**: [Where to find this in the codebase]

**Key Points to Review:**
- [ ] [Specific check 1]
- [ ] [Specific check 2]
- [ ] [Specific check 3]
- [ ] [Specific check 4]

**Example Pattern:**
\`\`\`[language]
// Good: [What makes this good]
[good code example]
\`\`\`

**Red Flags:**
- [Anti-pattern 1]
- [Anti-pattern 2]
- [Anti-pattern 3]

**See**: [reference-file.md](reference-file.md) for detailed [topic].

### 2. [Second Category Name]

[Repeat structure above]

## Code Pattern Review

### 1. [Pattern Category]

**Convention**: [What the convention is]

**Key Points to Review:**
- [ ] [Check 1]
- [ ] [Check 2]

**Example Pattern:**
\`\`\`[language]
// Good pattern
[code]

// Bad pattern
[code]
\`\`\`

## Testing Requirements

### 1. Unit Tests

**Convention**: [Where tests go, naming conventions]

**Key Points to Review:**
- [ ] [Test requirement 1]
- [ ] [Test requirement 2]

**Example Pattern:**
\`\`\`[language]
#[test]
fn test_[behavior]() {
    // Arrange
    // Act
    // Assert
}
\`\`\`

## References

- [reference-file-1.md](reference-file-1.md) - [Topic]
- [reference-file-2.md](reference-file-2.md) - [Topic]
- [CLAUDE.md](../../../CLAUDE.md) - Project-wide conventions

## Quick Review Commands

\`\`\`bash
# Build
[build command]

# Test
[test command]

# Lint
[lint command]
\`\`\`
```

## Template 2: Automation Skill

Use this template for skills that proactively maintain or generate content.

```markdown
---
name: [task-name]-automation
description: Automatically [action] including [task 1], [task 2], and [task 3]. Use proactively after [trigger event 1], [trigger event 2], or [trigger event 3]. Updates [artifact 1], adds [artifact 2], generates [artifact 3], and ensures [artifact 4] are up-to-date. Covers [scope 1], [scope 2], and [scope 3].
allowed-tools: Read, Grep, Glob, Write, Edit, Bash(git:status), Bash(git:diff), Bash(git:log)
---

# [Task Name] Automation Skill

This skill automatically [task description] across the project.

## Overview

[Description of what gets automated and why]

**Automation Scope:**
- **[Artifact 1]**: [What gets maintained]
- **[Artifact 2]**: [What gets generated]
- **[Artifact 3]**: [What gets updated]

**Trigger Events:**
1. After implementing features
2. After fixing bugs
3. After making [specific changes]
4. When [condition is met]

## Automation Tasks

### 1. [Task Name]

**When**: [Trigger conditions]

**What**: [What gets updated]

**How**:
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Format**: [Format specification]

**Example**:
\`\`\`
[example output]
\`\`\`

### 2. [Task Name]

[Repeat structure]

## Patterns and Conventions

### [Pattern Category]

**Format**: [Format specification]

**Rules**:
- [Rule 1]
- [Rule 2]

**Example**:
\`\`\`
[example]
\`\`\`

## Decision Tree

Use this to determine what automation to perform:

\`\`\`
Did the user implement a feature?
├─ Yes → Update [Artifact 1] with [Type 1] entry
└─ No → Check next condition

Did the user fix a bug?
├─ Yes → Update [Artifact 1] with [Type 2] entry
└─ No → Check next condition
\`\`\`

## Quality Checks

Before completing automation:
- [ ] [Check 1]
- [ ] [Check 2]
- [ ] [Check 3]

## References

- [pattern-file.md](pattern-file.md) - [Topic]
- [CLAUDE.md](../../../CLAUDE.md) - Project conventions
```

## Template 3: Multi-Component Service Skill

Use this template for skills covering multiple related services with shared patterns.

```markdown
---
name: [service-category]-review
description: Reviews [Language] code for [N] [service category] services ([service 1], [service 2], [service 3]) with focus on [shared pattern 1], [shared pattern 2], and [shared pattern 3]. Use when reviewing service changes, debugging [common issue type], validating [shared architecture], or evaluating [cross-service concerns]. Covers [pattern 1], [pattern 2], and [pattern 3] across all services.
allowed-tools: Read, Grep, Glob, Bash(build-command), Bash(test-command), Bash(docker:*)
---

# [Service Category] Review Skill

This skill provides code review for [N] related [Language] services in the [category] layer.

## Overview

[Description of service category and shared architecture]

**Services Covered:**
1. **[Service 1]** ([path]) - [Purpose]
2. **[Service 2]** ([path]) - [Purpose]
3. **[Service 3]** ([path]) - [Purpose]

**Shared Technology Stack:**
- **Language**: [Language + version]
- **Framework**: [Web framework]
- **Database**: [Database system]
- **Deployment**: [Container/orchestration]
- **Communication**: [Protocols]

## Shared Patterns Review

### 1. [Shared Pattern Category]

**Purpose**: [Why this pattern is used across services]

**Key Points to Review:**
- [ ] [Check 1 - applies to all services]
- [ ] [Check 2 - applies to all services]

**Example Pattern:**
\`\`\`[language]
// Good: Standard pattern used across services
[code]
\`\`\`

**See**: [shared-pattern.md](shared-pattern.md)

### 2. [Second Shared Pattern]

[Repeat structure]

## Service-Specific Reviews

### [Service 1 Name]

**Location**: [path]

**Purpose**: [What this service does]

**Specific Concerns:**
- [ ] [Service 1 specific check 1]
- [ ] [Service 1 specific check 2]

**Key Files:**
- [file 1]: [purpose]
- [file 2]: [purpose]

### [Service 2 Name]

[Repeat structure]

## Integration Review

### Service-to-Service Communication

**Key Points to Review:**
- [ ] [Integration check 1]
- [ ] [Integration check 2]

**Example Pattern:**
\`\`\`[language]
// Service A calling Service B
[code]
\`\`\`

## Deployment Review

### Docker Configuration

**Key Points to Review:**
- [ ] [Docker check 1]
- [ ] [Docker check 2]

**Example Pattern:**
\`\`\`dockerfile
# Multi-stage build pattern
[Dockerfile example]
\`\`\`

## References

- [shared-patterns.md](shared-patterns.md) - Patterns used across services
- [service-1-specifics.md](service-1-specifics.md) - Service 1 details
- [CLAUDE.md](../../../CLAUDE.md) - Project conventions

## Quick Commands

\`\`\`bash
# Build all services
[command]

# Test specific service
[command]

# Deploy with Docker Compose
[command]
\`\`\`
```

## Template 4: Deployment/Infrastructure Skill

Use this template for skills covering deployment, CI/CD, and infrastructure.

```markdown
---
name: [deployment-area]-automation
description: Reviews and automates [deployment aspect] including [task 1], [task 2], and [task 3]. Use when [trigger 1], [trigger 2], or [trigger 3]. Covers [scope 1] setup, [scope 2] configuration, [scope 3] workflows, and [scope 4] patterns.
allowed-tools: Read, Grep, Glob, Bash(docker:*), Bash(deploy-script), Bash(systemctl:status)
---

# [Deployment Area] Automation Skill

This skill handles [deployment aspect] across [scope].

## Overview

[Description of deployment infrastructure and processes]

**Deployment Scope:**
- **[Target 1]**: [What gets deployed where]
- **[Target 2]**: [What gets deployed where]
- **[Target 3]**: [What gets deployed where]

**Key Technologies:**
- **[Tool 1]**: [Purpose]
- **[Tool 2]**: [Purpose]
- **[Tool 3]**: [Purpose]

## Deployment Workflows

### 1. [Workflow Name]

**Purpose**: [What this workflow accomplishes]

**Trigger**: [When this workflow runs]

**Steps**:
1. [Step 1]
2. [Step 2]
3. [Step 3]

**Key Points to Review:**
- [ ] [Check 1]
- [ ] [Check 2]

**Example**:
\`\`\`bash
# Deployment command
[command]
\`\`\`

### 2. [Second Workflow]

[Repeat structure]

## Configuration Review

### [Configuration Category]

**Key Points to Review:**
- [ ] [Check 1]
- [ ] [Check 2]

**Example Configuration:**
\`\`\`[format]
[config example]
\`\`\`

## Environment Management

### [Environment Name]

**Purpose**: [Environment purpose]

**Configuration**:
- [Config 1]: [Value/pattern]
- [Config 2]: [Value/pattern]

**Deployment Process**:
1. [Step 1]
2. [Step 2]

## Troubleshooting

### Common Issues

**Issue**: [Problem description]
**Cause**: [Root cause]
**Solution**: [How to fix]
**Prevention**: [How to avoid]

## References

- [deployment-patterns.md](deployment-patterns.md) - Standard patterns
- [environment-configs.md](environment-configs.md) - Environment setup
- [CLAUDE.md](../../../CLAUDE.md) - Project conventions

## Quick Commands

\`\`\`bash
# Deploy
[command]

# Check status
[command]

# Rollback
[command]
\`\`\`
```

## Supporting File Template

Create supporting files for deep dives on specific topics.

```markdown
# [Topic Name]

Detailed information about [topic] for the [skill-name] skill.

## Overview

[Brief description of what this file covers]

## [Section 1]

[Detailed content]

### [Subsection]

[Content with code examples]

\`\`\`[language]
// Example
[code]
\`\`\`

## [Section 2]

[More detailed content]

## Reference Tables

| Item | Value | Notes |
|------|-------|-------|
| [Item 1] | [Value] | [Notes] |
| [Item 2] | [Value] | [Notes] |

## Diagrams

\`\`\`
[ASCII diagram]
\`\`\`

## Examples

### Example 1: [Scenario]

**Context**: [Setup]

**Code**:
\`\`\`[language]
[code]
\`\`\`

**Explanation**: [Why this works]

## See Also

- [Other reference file]
- [External documentation]
```

## Checklist for New Skills

Before creating a new skill, verify:

**Scope:**
- [ ] Skill covers a well-defined component or pattern
- [ ] Scope is neither too narrow (single file) nor too broad (entire language)
- [ ] Component has sufficient complexity to warrant a skill (>500 LOC or critical path)

**Content:**
- [ ] Frontmatter includes name, description, allowed-tools
- [ ] Description explains when to invoke (triggers)
- [ ] SKILL.md includes Overview, Review Checklist, Patterns, References
- [ ] 2-4 supporting files for deep dives
- [ ] Code examples for good/bad patterns
- [ ] Quick command reference

**Quality:**
- [ ] Checklist items are specific and actionable
- [ ] Examples include explanations (why it's good/bad)
- [ ] Red flags clearly identify anti-patterns
- [ ] References link to supporting files and CLAUDE.md

**Non-overlap:**
- [ ] Doesn't duplicate existing skill coverage
- [ ] If overlap exists, skills are consolidated or differentiated by scope

**Maintainability:**
- [ ] Based on stable patterns (not rapidly changing APIs)
- [ ] References project-specific conventions, not generic style guides
- [ ] Includes versioning info for tools/frameworks if relevant
