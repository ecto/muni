# Analysis Workflow

This document describes the step-by-step workflow for analyzing the codebase and creating new skills.

## Phase 1: Discovery

### Step 1: Inventory Components

Scan the repository structure to identify all major components.

**Commands**:
```bash
# List top-level directories
ls -d */

# Count files by component
find [component] -type f -name "*.[ext]" | wc -l

# Identify file types
find . -type f | sed 's/.*\.//' | sort | uniq -c | sort -rn | head -20

# Check documentation
find . -name "README.md" -o -name "CLAUDE.md" -o -name "*.md"
```

**Output**: Component inventory table

| Component | Type | Files | LOC | Languages | Documentation |
|-----------|------|-------|-----|-----------|---------------|
| bvr/firmware | Application | 45 | 8000 | Rust | Yes |
| depot/console | Web UI | 120 | 12000 | TS/React | Yes |
| ... | ... | ... | ... | ... | ... |

### Step 2: Catalog Existing Skills

List all current skills and their coverage areas.

**Commands**:
```bash
# List skills
ls .claude/skills/

# Read skill descriptions
grep "^description:" .claude/skills/*/SKILL.md
```

**Output**: Skill coverage map

| Skill | Component(s) | Type | Focus Areas |
|-------|--------------|------|-------------|
| firmware-review | bvr/firmware | Code Review | Safety, CAN, state machine |
| console-frontend-review | depot/console | Code Review | React, WebSocket, 3D viz |
| mcu-embedded-review | mcu/ | Code Review | Embassy, RP2350, ESP32 |
| documentation-automation | All | Automation | Docs, changelog, README |

### Step 3: Identify Gaps

Compare component inventory with skill coverage to find uncovered areas.

**Analysis**:
- List components WITHOUT dedicated skills
- Group related components (e.g., all depot Rust services)
- Assess complexity (LOC, architectural patterns, criticality)
- Rate priority (High/Medium/Low)

**Output**: Gap analysis

| Component | Complexity | Priority | Suggested Skill |
|-----------|------------|----------|-----------------|
| depot/discovery | Medium (400 LOC) | Medium | depot-services-review |
| depot/dispatch | Medium (600 LOC) | Medium | depot-services-review |
| depot/map-api | Medium (500 LOC) | Medium | depot-services-review |
| depot/mapper | Medium (800 LOC) | Medium | depot-services-review |
| depot/splat-worker | High (Python+GPU) | Low | splat-processing-review |
| paper/ | Low (Typst docs) | Low | typst-documentation |
| web/ | Low (static site) | Low | N/A |
| Deploy scripts | Medium | Medium | deployment-automation |

## Phase 2: Analysis

### Step 4: Deep Dive on High-Priority Components

For each high/medium priority gap, analyze the component in detail.

**Analysis Checklist**:
- [ ] Read main entry point (main.rs, index.ts, etc.)
- [ ] Read 3-5 core modules to identify patterns
- [ ] Read existing documentation (README, comments)
- [ ] Identify technology stack (frameworks, libraries)
- [ ] List common patterns (error handling, async, testing)
- [ ] Identify critical areas (safety, security, performance)
- [ ] Find similar components for comparison

**Example Analysis: depot/discovery**

1. **Entry point**: `src/main.rs` (120 LOC)
   - Tokio async runtime
   - Axum web server on port 4888
   - PostgreSQL connection
   - UDP broadcast listener

2. **Core modules**:
   - `src/db.rs` - PostgreSQL queries
   - `src/routes.rs` - REST API endpoints
   - `src/broadcast.rs` - UDP discovery protocol

3. **Patterns identified**:
   - Error handling: `anyhow::Result`
   - Database: SQLx with migrations
   - API: Axum router with JSON responses
   - Configuration: Environment variables

4. **Technology stack**:
   - Rust 1.83
   - Tokio 1.41 (async runtime)
   - Axum 0.7 (web framework)
   - SQLx 0.8 (database client)
   - PostgreSQL (database)

5. **Similar components**:
   - depot/dispatch (also Axum + PostgreSQL)
   - depot/map-api (also Axum + PostgreSQL)
   - depot/gps-status (similar patterns)

**Conclusion**: Group all depot Rust services into a single `depot-services-review` skill covering shared patterns.

### Step 5: Identify Shared Patterns

For groups of related components, identify shared patterns that should be standardized.

**Pattern Categories**:
1. **Architecture patterns**: Service structure, dependency injection
2. **Error handling**: Error types, context, logging
3. **Database patterns**: Migrations, queries, transactions
4. **API patterns**: Routing, middleware, request/response
5. **Testing patterns**: Unit tests, integration tests, mocks
6. **Deployment patterns**: Docker, configuration, environments

**Example Shared Patterns: Depot Services**

**Database Migrations**:
```sql
-- All services use SQLx migrations in migrations/
-- Pattern: YYYYMMDDHHMMSS_description.sql
-- Applied automatically on startup
```

**Error Handling**:
```rust
// All services use anyhow::Result in binary
use anyhow::{Context, Result};

fn handler() -> Result<Json<Response>> {
    let data = fetch_data()
        .context("Failed to fetch data")?;
    Ok(Json(Response { data }))
}
```

**Docker Build**:
```dockerfile
# All services use multi-stage builds
FROM rust:1.83 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/[service] /usr/local/bin/
CMD ["[service]"]
```

## Phase 3: Design

### Step 6: Design Skill Structure

Based on analysis, design the skill structure.

**Design Template**:

```
skill-name/
├── SKILL.md                    # Main skill file (1500-2500 lines)
│   ├── Frontmatter            # name, description, allowed-tools
│   ├── Overview               # Architecture, tech stack, files
│   ├── Review Checklist       # 3-5 major categories
│   ├── Code Patterns          # 3-5 pattern categories
│   ├── Testing Requirements   # Unit, integration tests
│   ├── References             # Links to supporting files
│   └── Quick Commands         # Build, test, deploy
├── [topic-1].md               # Deep dive file 1 (500-1000 lines)
├── [topic-2].md               # Deep dive file 2 (500-1000 lines)
└── [topic-3].md               # Deep dive file 3 (500-1000 lines)
```

**Example Design: depot-services-review**

```
depot-services-review/
├── SKILL.md
│   ├── Overview
│   │   ├── Services covered (4 services)
│   │   ├── Shared tech stack
│   │   └── Key files reference
│   ├── Shared Patterns Checklist
│   │   ├── Database migrations
│   │   ├── Error handling
│   │   ├── API design (Axum routing)
│   │   └── Docker deployment
│   ├── Service-Specific Reviews
│   │   ├── Discovery service
│   │   ├── Dispatch service
│   │   ├── Map API service
│   │   └── Mapper service
│   ├── Integration Review (service-to-service)
│   ├── Code Patterns (Rust + Axum + SQLx)
│   └── Testing (unit + integration)
├── database-patterns.md
│   ├── SQLx setup
│   ├── Migration patterns
│   ├── Query patterns
│   └── Transaction handling
├── api-design-patterns.md
│   ├── Axum routing conventions
│   ├── Middleware (CORS, auth, logging)
│   ├── Request validation
│   └── Error responses
└── docker-deployment.md
    ├── Multi-stage builds
    ├── Docker Compose setup
    ├── Environment variables
    └── Health checks
```

### Step 7: Draft Content Outline

Create a detailed outline for each file.

**SKILL.md Outline**:
```markdown
# Depot Services Review Skill

## Overview
- Services covered
- Shared technology stack
- Architecture summary

## Shared Patterns Review Checklist

### 1. Database Migrations
- [ ] 5-7 checklist items
- Good/bad examples
- References to database-patterns.md

### 2. Error Handling
- [ ] 5-7 checklist items
- Good/bad examples
- References to error handling patterns

### 3. API Design (Axum)
- [ ] 5-7 checklist items
- Good/bad examples
- References to api-design-patterns.md

### 4. Docker Deployment
- [ ] 5-7 checklist items
- Good/bad examples
- References to docker-deployment.md

## Service-Specific Reviews

### Discovery Service
- Purpose
- Key files
- Specific concerns (UDP broadcast, registration)

### Dispatch Service
- Purpose
- Key files
- Specific concerns (WebSocket, task assignment)

### Map API Service
- Purpose
- Key files
- Specific concerns (tile serving, caching)

### Mapper Service
- Purpose
- Key files
- Specific concerns (map processing, pipelines)

## Code Patterns
- Async/Tokio patterns
- Configuration loading
- Logging with tracing

## Testing Requirements
- Unit test conventions
- Integration test setup
- Docker test environment

## References
- Links to supporting files
- CLAUDE.md reference

## Quick Commands
- Build commands
- Test commands
- Docker commands
```

## Phase 4: User Consultation

### Step 8: Present Analysis to User

Before creating any files, present the analysis and design to the user.

**Presentation Format**:

```
# Skill Coverage Analysis

## Current Coverage
[List existing 4 skills with brief descriptions]

## Gaps Identified
[Table of 6-8 uncovered components with priority ratings]

## Suggested New Skills

### 1. depot-services-review (High Priority)
**Coverage**: 4 Rust services (discovery, dispatch, map-api, mapper)
**Rationale**: [Why this is needed]
**Focus Areas**: [3-5 key focus areas]
**Files**: SKILL.md + 3 supporting files

### 2. deployment-automation (Medium Priority)
[Similar structure]

### 3. splat-processing-review (Low Priority)
[Similar structure]

## Recommendation

I recommend creating **depot-services-review** first because:
1. [Reason 1]
2. [Reason 2]
3. [Reason 3]

Would you like me to:
A) Create the depot-services-review skill as designed above
B) Modify the scope or focus areas
C) Create a different skill first
D) Skip skill creation for now
```

### Step 9: Gather User Feedback

Use `AskUserQuestion` to clarify:

**Questions to Ask**:
1. Which skill(s) should be created?
2. Should any skills be combined or split?
3. What focus areas are most important?
4. What level of detail (comprehensive vs. high-level)?
5. Should any components be excluded?

**Example Questions**:
```markdown
I've identified that depot services (discovery, dispatch, map-api, mapper) would benefit from a shared skill. Should I:

1. **Create one skill for all 4 services** (recommended)
   - Pros: Shared patterns, consistency, easier maintenance
   - Cons: Longer skill file, some service-specific details

2. **Create separate skills for each service**
   - Pros: More focused, service-specific depth
   - Cons: Pattern duplication, harder to maintain

3. **Group only related services** (e.g., dispatch+map-api separate from discovery+mapper)
   - Pros: Balance between shared patterns and specificity
   - Cons: More skills to maintain

Which approach do you prefer?
```

## Phase 5: Creation

### Step 10: Create Skill Files

After user approval, create all skill files.

**Creation Order**:
1. Create skill directory
2. Write SKILL.md with frontmatter
3. Write supporting files
4. Create internal cross-references

**Commands**:
```bash
# Create directory
mkdir -p .claude/skills/[skill-name]

# Create files
# Use Write tool for SKILL.md
# Use Write tool for each supporting file
```

### Step 11: Verify Skill Registration

Test that the skill is recognized by Claude Code.

**Verification**:
1. Check that skill appears in system prompt
2. Test invocation: `/[skill-name]`
3. Verify allowed-tools are appropriate
4. Check cross-references work

### Step 12: Document Creation

Update any meta-documentation:
- Add skill to project README if there's a skills section
- Update CLAUDE.md if skill introduces new conventions
- Create PR if working in a team environment

## Phase 6: Iteration

### Step 13: Gather Feedback

After using the skill:
- Ask user if skill met their needs
- Identify any missing coverage areas
- Note any confusing sections
- Check if examples are helpful

### Step 14: Refine Skill

Based on feedback:
- Add missing checklist items
- Clarify confusing sections
- Add more examples
- Fix errors or outdated information
- Create additional supporting files if needed

## Templates for User Interaction

### Template: Analysis Results

```
I've analyzed the codebase and found:

**Existing Skills (4):**
- firmware-review: BVR firmware (Rust)
- console-frontend-review: React web app
- mcu-embedded-review: Embedded firmware
- documentation-automation: Docs and changelog

**Uncovered Components (6):**

| Component | Complexity | Priority | Lines of Code |
|-----------|------------|----------|---------------|
| depot/discovery | Medium | Medium | 400 |
| depot/dispatch | Medium | High | 600 |
| depot/map-api | Medium | Medium | 500 |
| depot/mapper | Medium | Medium | 800 |
| depot/splat-worker | High | Low | 1200 |
| paper/ | Low | Low | 2000 |

**Suggested Skills:**

1. **depot-services-review** (High Priority)
   - Covers: discovery, dispatch, map-api, mapper
   - Shared patterns: Axum, SQLx, Docker
   - Estimated size: 2000 lines + 3 supporting files

2. **deployment-automation** (Medium Priority)
   - Covers: deploy.sh, Docker Compose, systemd
   - Patterns: Cross-compile, multi-stage builds
   - Estimated size: 1500 lines + 2 supporting files

3. **splat-processing-review** (Low Priority)
   - Covers: Python GPU worker
   - Patterns: CUDA, Gaussian splatting
   - Estimated size: 1200 lines + 2 supporting files

Would you like me to create any of these skills?
```

### Template: Skill Design Proposal

```
I've designed the **[skill-name]** skill. Here's the structure:

**Frontmatter:**
- Name: [skill-name]
- Description: [200-word description]
- Allowed tools: Read, Grep, Glob, Bash([specific commands])

**Content Structure:**

1. **Overview** (200 lines)
   - Architecture summary
   - Technology stack
   - Key files reference

2. **Review Checklist** (800 lines)
   - [Category 1]: [5-7 items]
   - [Category 2]: [5-7 items]
   - [Category 3]: [5-7 items]
   - [Category 4]: [5-7 items]

3. **Code Patterns** (400 lines)
   - [Pattern 1]
   - [Pattern 2]
   - [Pattern 3]

4. **Testing Requirements** (200 lines)
5. **References** (50 lines)
6. **Quick Commands** (50 lines)

**Supporting Files:**
- [file-1].md: [Topic] (600 lines)
- [file-2].md: [Topic] (500 lines)
- [file-3].md: [Topic] (400 lines)

**Total estimated size**: ~3200 lines

Does this structure work for you, or would you like any changes?
```

### Template: Post-Creation Summary

```
I've created the **[skill-name]** skill:

**Files created:**
- .claude/skills/[skill-name]/SKILL.md (1800 lines)
- .claude/skills/[skill-name]/[file-1].md (600 lines)
- .claude/skills/[skill-name]/[file-2].md (500 lines)
- .claude/skills/[skill-name]/[file-3].md (400 lines)

**Coverage:**
- [Component 1]: [Focus areas]
- [Component 2]: [Focus areas]
- [Component 3]: [Focus areas]

**To use this skill:**
```bash
# Invoke skill
/[skill-name]

# Or in conversation
"Review [component] with focus on [area]"
```

Would you like me to test the skill on a sample file?
```
