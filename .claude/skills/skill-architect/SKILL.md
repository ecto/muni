---
name: skill-architect
description: Analyzes the codebase to identify areas that would benefit from specialized skills, reviews existing skills for gaps, and creates new skills with user approval. Use when the user wants to improve their skill coverage, create new code review skills, or ensure all major components have appropriate specialized assistants. Proactively suggests skills for undocumented areas like depot services, build systems, deployment workflows, or testing infrastructure.
allowed-tools: Read, Glob, Grep, Write, Bash(ls), Bash(tree), AskUserQuestion
---

# Skill Architect

This skill analyzes your codebase to identify areas that would benefit from specialized Claude skills, reviews existing skills for gaps, and helps create new skills with your approval.

## Overview

As your codebase grows, different components develop their own patterns, conventions, and gotchas. The Skill Architect helps you maintain comprehensive skill coverage by:

1. **Analyzing** the codebase structure to identify major components
2. **Reviewing** existing skills to find coverage gaps
3. **Suggesting** new skills based on patterns, complexity, and specialization needs
4. **Creating** new skill files with appropriate scope and documentation

## When to Use This Skill

Invoke this skill when you want to:
- Audit existing skill coverage
- Create skills for new components or services
- Improve code review coverage for specific areas
- Standardize patterns across similar components (e.g., all Rust depot services)
- Onboard new team members with specialized skill assistants

## Analysis Process

### 1. Codebase Structure Analysis

The skill examines the repository to identify:
- **Major components**: Top-level directories and their purposes
- **Technology stacks**: Languages, frameworks, build systems
- **Component complexity**: File count, lines of code, architectural patterns
- **Existing documentation**: READMEs, CLAUDE.md, architecture docs

**Key directories to analyze:**
```
muni/
├── bvr/firmware/        # Already covered: firmware-review
├── depot/console/       # Already covered: console-frontend-review
├── depot/discovery/     # UNCOVERED: Rust service
├── depot/dispatch/      # UNCOVERED: Rust service
├── depot/map-api/       # UNCOVERED: Rust service
├── depot/mapper/        # UNCOVERED: Rust service
├── depot/splat-worker/  # UNCOVERED: Python GPU service
├── mcu/                 # Already covered: mcu-embedded-review
├── paper/               # UNCOVERED: Typst documents
└── web/                 # UNCOVERED: Static website
```

### 2. Existing Skill Review

Analyzes current skills to understand:
- **Coverage scope**: What components each skill handles
- **Depth**: How detailed the skill guidance is
- **Patterns**: Common review checklist items, tool usage, references
- **Gaps**: Components or technologies not covered by any skill

**Current skills:**
- `firmware-review`: BVR firmware (Rust + Tokio + CAN)
- `console-frontend-review`: React/TypeScript web app
- `documentation-automation`: Maintains docs, changelogs, READMEs
- `mcu-embedded-review`: RP2350/ESP32-S3 embedded firmware

### 3. Skill Suggestion Algorithm

Suggests new skills based on:

**High Priority (should have dedicated skills):**
- Complex components with unique patterns (>1000 LOC)
- Safety-critical systems
- Technologies requiring specialized knowledge
- Components with frequent changes or reviews
- Areas with many conventions or gotchas

**Medium Priority (nice to have):**
- Services with similar patterns (e.g., all depot Rust services)
- Build/deployment workflows with many steps
- Testing infrastructure with specific requirements
- Integration patterns between components

**Low Priority (not worth dedicated skills):**
- Simple utilities or scripts
- Static content with minimal logic
- Well-documented components with clear patterns
- Areas that rarely change

### 4. Skill Creation Template

When creating new skills, includes:

**Frontmatter:**
- `name`: Kebab-case identifier
- `description`: When to use (triggers), what it covers, key technologies
- `allowed-tools`: Appropriate tools (Read, Grep, Glob, Bash variants, etc.)

**Content Structure:**
```markdown
# [Skill Name]

## Overview
- Component purpose
- Technology stack
- Architecture summary
- Key files reference

## Review Checklist
### 1. [Category Name]
- [ ] Checklist items with clear pass/fail criteria
- Example patterns (good/bad)
- Common pitfalls
- References to detailed docs

## Code Patterns
- Language-specific conventions
- Framework patterns
- Error handling
- Testing requirements

## Quick Commands
- Build, test, deploy commands
- Common development tasks
```

**Supporting Files:**
- Create additional `.md` files for detailed topics (protocols, checklists, patterns)
- Reference external docs in CLAUDE.md or component READMEs
- Keep SKILL.md as the primary entry point

## Example Skill Suggestions

Based on the Muni codebase:

### 1. `depot-services-review` (High Priority)
**Coverage:** All Rust depot services (discovery, dispatch, map-api, mapper, gps-status)
**Rationale:**
- 5 Rust services with similar patterns (Tokio, Axum/Tonic, PostgreSQL)
- Shared concerns: database migrations, API design, error handling, Docker deployment
- Each service 200-800 LOC, moderate complexity
- Active development area

**Focus Areas:**
- Database schema and migration best practices
- REST/gRPC API design patterns
- Docker multi-stage builds
- Service-to-service communication
- Metrics and observability (InfluxDB integration)

### 2. `deployment-automation` (Medium Priority)
**Coverage:** Deploy scripts, Docker Compose, CI/CD workflows
**Rationale:**
- Complex deployment across rovers (aarch64 cross-compile) and depot (x86_64 Docker)
- Multiple environments (development, staging, production)
- Specific patterns: deploy.sh script, Docker Compose profiles, systemd services

**Focus Areas:**
- Cross-compilation setup (cross tool, target config)
- Deploy script conventions (--cli, --restart flags)
- Docker Compose profiles (gpu, rtk)
- Systemd service management
- Environment variable handling

### 3. `typst-documentation` (Low Priority)
**Coverage:** Technical documents in paper/ directory
**Rationale:**
- Specialized format (Typst, not Markdown)
- Specific conventions for datasheets, manuals, pitch decks
- Graphics and brand assets (logos, charts)

**Focus Areas:**
- Typst syntax and patterns
- Document templates (one-pagers, manuals, decks)
- Graphics inclusion and positioning
- PDF generation and build system (Makefile)

### 4. `integration-testing` (Medium Priority)
**Coverage:** End-to-end tests, simulation, mock infrastructure
**Rationale:**
- Complex integration points (rover ↔ depot, CAN bus, WebSocket)
- Need for mocks and simulators
- Testing patterns across Rust, TypeScript, Python

**Focus Areas:**
- Mock CAN bus for firmware testing
- WebSocket protocol testing
- Database fixtures and test data
- Docker test environments
- Rerun recording validation

## Skill Creation Workflow

### Step 1: Gather Context
- Read component source files (main entry points, core modules)
- Read existing documentation (READMEs, CLAUDE.md, architecture docs)
- Identify patterns by examining 3-5 representative files
- Review existing skills to avoid duplication

### Step 2: Define Scope
Ask the user:
- **What components** should this skill cover? (single service vs. multiple related services)
- **What focus areas** are most important? (code review, deployment, testing, architecture)
- **What level of detail**? (comprehensive checklist vs. high-level guidance)
- **What allowed tools**? (Read/Grep/Glob for review, Bash for testing, etc.)

### Step 3: Draft Skill Content
- Create SKILL.md with frontmatter and main content
- Identify 2-4 supporting files for detailed topics
- Include code examples (good/bad patterns)
- Add quick reference commands

### Step 4: Review with User
Present the skill structure:
- Show the frontmatter (name, description, allowed-tools)
- Outline the main sections
- List supporting files
- Ask for feedback before creating files

### Step 5: Create Files
- Write SKILL.md
- Write supporting markdown files
- Verify skill is recognized by Claude Code
- Test skill invocation with a sample review

## Usage Examples

### Example 1: Audit Existing Coverage
```
User: "Run /skill-architect to check what skills we're missing"

Skill analyzes:
1. Lists 4 existing skills and their coverage
2. Identifies 8 major components
3. Finds 4 uncovered areas (depot services, deployment, paper, web)
4. Suggests 3 new skills with priority ratings
5. Asks user which skills to create
```

### Example 2: Create Specific Skill
```
User: "Create a skill for reviewing depot Rust services"

Skill workflow:
1. Reads discovery, dispatch, map-api, mapper source files
2. Identifies common patterns (Axum, PostgreSQL, Docker)
3. Drafts depot-services-review skill with:
   - Database migration checklist
   - API design patterns
   - Docker build best practices
   - Service-specific sections for each service
4. Presents draft to user
5. Creates files after approval
```

### Example 3: Improve Existing Skill
```
User: "The firmware-review skill is missing GPS/RTK patterns"

Skill workflow:
1. Reads firmware-review/SKILL.md
2. Reads bvr/firmware/crates/gps/ source
3. Identifies GPS-specific review items (NMEA parsing, RTK corrections, coordinate transforms)
4. Drafts new section and supporting file (gps-patterns.md)
5. Presents changes to user
6. Updates skill files after approval
```

## Best Practices

### Skill Naming
- Use kebab-case: `depot-services-review`, not `Depot_Services_Review`
- Be specific: `firmware-review` (BVR-specific) not `rust-review` (too broad)
- Use `-review` suffix for code review skills
- Use `-automation` suffix for proactive task skills

### Description Guidelines
- Start with what the skill reviews/handles
- Include key technologies in the first sentence
- List specific use cases (when to invoke)
- Mention 3-5 key focus areas
- Keep under 500 characters

### Allowed Tools
**Read-only skills** (code review, analysis):
- `Read, Grep, Glob` - Essential for reading code
- `Bash(cargo:test)` - Run tests to verify
- `Bash(cargo:check)` - Type checking
- `Bash(npm:run:build)` - Build verification

**Write-enabled skills** (automation, fixes):
- Add `Write, Edit` for code modifications
- Add `Bash(git:*)` for git operations
- Use sparingly - most skills should be read-only

### Content Organization
**SKILL.md** (1000-2000 lines):
- High-level overview
- Review checklists with examples
- Common patterns (good/bad)
- Quick command reference

**Supporting files** (500-1000 lines each):
- Deep dives on specific topics
- Protocol specifications
- Architecture diagrams (as text)
- Extended examples

### Checklist Design
- Use `[ ]` checkboxes for clear pass/fail items
- Include code examples after each item
- Explain WHY the pattern matters (safety, performance, maintainability)
- Provide both good and bad examples
- Link to detailed supporting files

### Example Quality
```markdown
**Good Example Structure:**

❌ BAD:
\`\`\`rust
// Missing error handling
let result = do_something();
\`\`\`

✅ GOOD:
\`\`\`rust
// Proper error handling with context
let result = do_something()
    .context("Failed to do something")?;
\`\`\`

**Why:** Context-aware errors improve debuggability and help trace failures in production.
```

## Skill Maintenance

Skills should be updated when:
- **New patterns emerge**: Add to checklist with examples
- **Technologies change**: Update tool versions, API changes
- **Coverage gaps found**: Add new sections or supporting files
- **User feedback**: Clarify confusing items, add missing edge cases

**Update process:**
1. Identify what needs to change
2. Read current skill content
3. Draft changes (new sections, updated examples, fixed errors)
4. Present diff to user
5. Update files after approval

## Anti-Patterns to Avoid

**Don't create skills for:**
- Single files or functions (too narrow)
- Language-generic reviews (too broad)
- Rarely-changed code (wasted effort)
- Well-documented trivial code (adds no value)

**Don't include in skills:**
- Complete source code listings (link instead)
- Duplicate information from CLAUDE.md (reference instead)
- Subjective style preferences (focus on correctness and safety)
- Framework-specific trivia (focus on project patterns)

**Don't make skills that:**
- Require constant updates (use stable patterns)
- Overlap significantly with existing skills (consolidate instead)
- Demand expert knowledge beyond the codebase (link to external docs)

## Quick Reference

### Analysis Commands
```bash
# Find all major components
ls -d */ | head -20

# Count files by component
find depot/ -name "*.rs" | wc -l
find depot/console -name "*.tsx" | wc -l

# Identify languages
find . -name "*.rs" -o -name "*.ts" -o -name "*.py" | \
  sed 's/.*\.//' | sort | uniq -c

# List existing skills
ls .claude/skills/
```

### Skill Testing
```bash
# Verify skill is recognized (check system prompt includes it)
# Invoke skill manually:
# /skill-architect

# Test on sample component
# E.g., "Review depot/discovery with the new depot-services-review skill"
```

## References

- [CLAUDE.md](../../../CLAUDE.md) - Project-wide conventions
- [firmware-review/SKILL.md](../firmware-review/SKILL.md) - Example of comprehensive review skill
- [documentation-automation/SKILL.md](../documentation-automation/SKILL.md) - Example of automation skill
