# Git Workflow - Git Flow

**Last Updated:** 2025-01-24

---

## Branch Structure

This repository uses **git-flow** branching model with the following structure:

### Main Branches

- **`main`** - Production-ready code
  - All releases are tagged here
  - Only accepts merges from `release/` or `hotfix/` branches
  - Protected: Direct commits not allowed

- **`develop`** - Integration branch for next release
  - All feature development integrates here
  - Base branch for feature branches
  - Tracks latest development state

### Supporting Branches

- **`feature/*`** - New features and enhancements
  - Branch from: `develop`
  - Merge back to: `develop`
  - Naming: `feature/description`

- **`release/*`** - Prepare for production release
  - Branch from: `develop`
  - Merge back to: `main` and `develop`
  - Naming: `release/version`

- **`hotfix/*`** - Critical production fixes
  - Branch from: `main`
  - Merge back to: `main` and `develop`
  - Naming: `hotfix/description`

- **`support/*`** - Long-term support branches
  - Branch from: specific tag on `main`
  - Naming: `support/version`

---

## Git Flow Configuration

```
Production branch:     main
Development branch:    develop
Feature prefix:        feature/
Release prefix:        release/
Hotfix prefix:         hotfix/
Support prefix:        support/
Version tag prefix:    (none)
```

---

## Common Workflows

### Starting a New Feature

```bash
# Create and switch to feature branch
git flow feature start my-feature

# Work on feature...
git add .
git commit -m "feat: implement my feature"

# Finish feature (merges to develop and deletes feature branch)
git flow feature finish my-feature

# Push develop
git push origin develop
```

### Creating a Release

```bash
# Start release branch
git flow release start 1.0.0

# Bump version, update CHANGELOG, final adjustments...
git commit -am "chore: prepare release 1.0.0"

# Finish release (merges to main and develop, creates tag)
git flow release finish 1.0.0

# Push all branches and tags
git push origin main develop --tags
```

### Hotfix for Production

```bash
# Start hotfix from main
git flow hotfix start fix-critical-bug

# Apply fix...
git commit -am "fix: critical bug description"

# Finish hotfix (merges to main and develop, creates tag)
git flow hotfix finish fix-critical-bug

# Push all branches and tags
git push origin main develop --tags
```

---

## Daily Development Workflow

### For Feature Development

1. **Start your day:**
   ```bash
   git checkout develop
   git pull origin develop
   ```

2. **Create feature branch:**
   ```bash
   git flow feature start sprint-1-4-gpui-bootstrap
   ```

3. **Work and commit regularly:**
   ```bash
   git add .
   git commit -m "feat: add GPUI initialization"
   ```

4. **Keep feature updated with develop:**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout feature/sprint-1-4-gpui-bootstrap
   git merge develop
   ```

5. **Finish feature:**
   ```bash
   git flow feature finish sprint-1-4-gpui-bootstrap
   git push origin develop
   ```

### For Quick Fixes on Develop

```bash
git checkout develop
git pull origin develop
# Make changes...
git add .
git commit -m "fix: description"
git push origin develop
```

---

## Sprint Workflow Mapping

### Phase 1 Example (Current State)

- **Sprint 1.1-1.3:** ✅ Complete (committed to `main`)
- **Sprint 1.4:** GPUI Bootstrap (next)
  - Create: `git flow feature start sprint-1-4-gpui-bootstrap`
  - Work on implementation
  - Finish: `git flow feature finish sprint-1-4-gpui-bootstrap`
  - Push to develop

- **Sprint 1.5:** Workspace Tabs
  - Create: `git flow feature start sprint-1-5-workspace-tabs`
  - Implement
  - Finish and merge to develop

### Phase 1 Complete (Release)

When all Phase 1 sprints are complete:

```bash
# Create release branch
git flow release start phase-1-complete

# Update documentation, version numbers
git commit -am "chore: prepare Phase 1 release"

# Finish release
git flow release finish phase-1-complete

# Push everything
git push origin main develop --tags
```

This tags the Phase 1 checkpoint on `main`.

---

## Branch Protection (Recommended GitHub Settings)

### `main` Branch Protection

- ✅ Require pull request reviews
- ✅ Require status checks to pass
- ✅ Require branches to be up to date
- ✅ Include administrators
- ✅ Restrict who can push (only release/hotfix merges)

### `develop` Branch Protection

- ✅ Require status checks to pass
- ⚠️ Allow direct commits (for small fixes)
- ✅ Require feature branches for major work

---

## Git Flow Commands Reference

### Feature Commands
```bash
git flow feature start <name>      # Create feature branch
git flow feature finish <name>     # Merge feature to develop
git flow feature publish <name>    # Push feature to remote
git flow feature pull origin <name> # Pull feature from remote
```

### Release Commands
```bash
git flow release start <version>   # Create release branch
git flow release finish <version>  # Merge to main and develop, create tag
git flow release publish <version> # Push release branch to remote
```

### Hotfix Commands
```bash
git flow hotfix start <name>       # Create hotfix from main
git flow hotfix finish <name>      # Merge to main and develop, create tag
```

### Support Commands
```bash
git flow support start <version> <base> # Create support branch from tag
```

---

## Version Tagging Strategy

### Semantic Versioning (SemVer)

Format: `MAJOR.MINOR.PATCH`

- **MAJOR:** Incompatible API changes
- **MINOR:** Add functionality (backwards-compatible)
- **PATCH:** Bug fixes (backwards-compatible)

### Phase-Based Tags

- Phase milestones: `phase-1-complete`, `phase-2-complete`, etc.
- MVP milestone: `mvp-1.0.0`
- Post-MVP: `1.1.0`, `1.2.0`, etc.

---

## Integration with MASTER-PLAN.md

### Sprint Workflow

1. Pick sprint from MASTER-PLAN.md
2. Create feature branch: `git flow feature start sprint-X-Y-name`
3. Follow sprint checklist
4. Commit regularly with conventional commits
5. Finish feature: `git flow feature finish sprint-X-Y-name`
6. Update MASTER-PLAN.md status
7. Commit status update to develop

### Phase Completion

1. Verify all sprints complete on develop
2. Create release branch: `git flow release start phase-X-complete`
3. Update documentation
4. Finish release
5. Tag automatically created on main

---

## Troubleshooting

### If you accidentally commit to main
```bash
# Move commits to develop
git checkout main
git reset --hard origin/main
git checkout develop
git cherry-pick <commit-hash>
```

### If feature branch conflicts with develop
```bash
git checkout feature/my-feature
git merge develop
# Resolve conflicts
git add .
git commit
```

### Abort feature/release/hotfix
```bash
git flow feature delete my-feature
# or
git branch -D feature/my-feature
```

---

## References

- [Git Flow Cheatsheet](https://danielkummer.github.io/git-flow-cheatsheet/)
- [Original Git Flow Blog Post](https://nvie.com/posts/a-successful-git-branching-model/)
- [Semantic Versioning](https://semver.org/)

---

**Status:** ✅ Git Flow Initialized
**Main Branch:** `main`
**Develop Branch:** `develop`
**Current Branch:** `develop`
