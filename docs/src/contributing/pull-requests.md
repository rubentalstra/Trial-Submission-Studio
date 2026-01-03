# Pull Requests

Guidelines for submitting pull requests to Trial Submission Studio.

## Before Creating a PR

### Complete Your Changes

- [ ] Code compiles: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] Lints pass: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt`

### Commit Guidelines

#### Conventional Commits

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

#### Types

| Type       | Description             |
|------------|-------------------------|
| `feat`     | New feature             |
| `fix`      | Bug fix                 |
| `docs`     | Documentation only      |
| `test`     | Adding/updating tests   |
| `refactor` | Code refactoring        |
| `perf`     | Performance improvement |
| `chore`    | Maintenance tasks       |

#### Examples

```bash
git commit -m "feat(validate): add CT validation for SEX variable"
git commit -m "fix(xpt): handle missing values correctly"
git commit -m "docs: update installation instructions"
git commit -m "test(map): add property tests for similarity"
git commit -m "refactor(ingest): simplify schema detection"
```

### Keep PRs Focused

- One feature or fix per PR
- Small, reviewable changes
- Don't mix refactoring with features

## Creating a PR

### Push Your Branch

```bash
git push origin feature/my-feature
```

### Open PR on GitHub

1. Go to your fork on GitHub
2. Click "Pull Request"
3. Select your branch
4. Fill in the template

### PR Title

Use same format as commits:

```
feat(validate): add USUBJID cross-domain validation
fix(xpt): correct numeric precision for large values
docs: add API documentation for tss-map
```

### PR Description Template

```markdown
## Summary

Brief description of what this PR does.

## Changes

- Added X
- Fixed Y
- Updated Z

## Testing

How was this tested?

- [ ] Unit tests added
- [ ] Manual testing performed
- [ ] Tested on: macOS / Windows / Linux

## Related Issues

Fixes #123
Related to #456

## Checklist

- [ ] Code compiles without warnings
- [ ] Tests pass
- [ ] Clippy passes
- [ ] Code is formatted
- [ ] Documentation updated (if needed)
```

## Review Process

### What Reviewers Look For

1. **Correctness** - Does it work?
2. **Tests** - Are changes tested?
3. **Style** - Follows coding standards?
4. **Performance** - Any concerns?
5. **Documentation** - Updated if needed?

### Responding to Feedback

1. Address all comments
2. Push additional commits
3. Mark conversations resolved
4. Request re-review when ready

### Acceptable Responses

- Fix the issue
- Explain why it's correct
- Discuss alternative approaches
- Agree to follow up in separate PR

## After Merge

### Clean Up

```bash
# Switch to main
git checkout main

# Update from upstream
git pull upstream main

# Delete local branch
git branch -d feature/my-feature

# Delete remote branch (optional, GitHub can auto-delete)
git push origin --delete feature/my-feature
```

### Update Fork

```bash
git push origin main
```

## PR Types

### Feature PRs

- Reference the issue or discussion
- Include tests
- Update documentation if user-facing

### Bug Fix PRs

- Reference the bug issue
- Include regression test
- Explain root cause if complex

### Documentation PRs

- No code changes required
- Preview locally: `mdbook serve`
- Check links work

### Refactoring PRs

- No behavior changes
- All existing tests must pass
- Add tests if coverage was low

## Tips for Good PRs

### Make Review Easy

- Write clear descriptions
- Add comments on complex code
- Break large changes into steps

### Be Patient

- Reviews take time
- Don't ping repeatedly
- Provide more context if asked

### Learn from Feedback

- Feedback improves code quality
- Ask questions if unclear
- Apply learnings to future PRs

## Automated Checks

### CI Pipeline

Every PR runs:

1. **Build** - Compilation check
2. **Test** - All tests
3. **Lint** - Clippy
4. **Format** - rustfmt

### Required Checks

All checks must pass before merge.

### Fixing Failed Checks

```bash
# If tests fail
cargo test

# If clippy fails
cargo clippy -- -D warnings

# If format fails
cargo fmt
```

## Emergency Fixes

For critical bugs:

1. Create PR with `hotfix/` prefix
2. Note urgency in description
3. Request expedited review

## Questions?

- Ask in PR comments
- Open a Discussion
- Reference documentation

## Next Steps

- [Getting Started](getting-started.md) - Contribution overview
- [Coding Standards](coding-standards.md) - Style guide
- [Testing](testing.md) - Testing guide
