---
name: commit
description: Commit staged and unstaged changes in logical chunks using conventional commit messages. Cleans up debug statements, runs lints/tests/builds after committing, and fixes any issues that arise.
allowed-tools: Bash, Read, Grep, Glob, Edit
user-invocable: true
---

# Commit Skill

Commit the current git diff in logical, well-organized chunks using conventional commit messages.

## Workflow

### 1. Analyze Changes

First, understand what has changed:

```bash
git status
git diff
git diff --staged
```

### 2. Clean Up Debug Statements

Before committing, scan for and handle debug statements:

- **Remove** any `console.log`, `dbg!`, `println!` debug statements that were added during development
- **Convert** any debug statements that have enduring utility to use the appropriate logger:
  - TypeScript: Use the project's logging utility
  - Rust: Use `tracing` macros (`info!`, `debug!`, `warn!`, `error!`)

### 3. Ask for Clarification

If it's unclear what should be committed or how changes should be grouped:

- Ask the user for clarification on commit scope
- Ask about which changes belong together logically
- Ask about commit message wording if the intent is ambiguous

### 4. Commit in Logical Chunks

Group related changes into separate commits:

- Each commit should represent a single logical change
- Don't mix unrelated changes in one commit
- Order commits so the history tells a coherent story

### 5. Use Conventional Commit Messages

Format: `<type>(<scope>): <description>`

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `refactor`: Code restructuring (no behavior change)
- `test`: Add/update tests
- `chore`: Tooling, dependencies, build config
- `style`: Code style (formatting, no logic change)
- `perf`: Performance improvement

**Examples**:
- `feat(teleop): add gamepad vibration feedback`
- `fix(can): handle malformed VESC status frames`
- `refactor(control): extract rate limiter to separate module`
- `docs(readme): add deployment instructions`

### 6. Verify After Committing

After creating commits, run all verification:

```bash
# For Rust projects
cargo check
cargo clippy
cargo test

# For TypeScript projects
npm run lint
npm run build
npm test
```

### 7. Fix Issues

If lints, checks, tests, or builds fail:

1. Fix the issues
2. Amend the relevant commit or create a fixup commit
3. Re-run verification until everything passes

## Best Practices

- Keep commit messages concise but descriptive (50 chars for subject line)
- Reference issue numbers when applicable: `fix(auth): resolve login timeout (#42)`
- Use imperative mood: "Add feature" not "Added feature"
- Include Co-Authored-By trailer for pair programming or AI assistance
