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
# Create release branch from develop
git flow release start phase-1-complete

# Update documentation, version numbers
git commit -am "chore: prepare Phase 1 release"

# Push release branch to create PR
git push origin release/phase-1-complete

# Create PR to main using GitHub CLI
gh pr create --base main --head release/phase-1-complete \
  --title "Release: Phase 1 Complete" \
  --body "Phase 1 checkpoint release"

# After PR is reviewed and merged:
# 1. GitHub merges release branch to main
# 2. Manually merge back to develop:
git checkout develop
git pull origin develop
git merge main
git push origin develop

# Tag the release on main
git checkout main
git pull origin main
git tag phase-1-complete
git push origin --tags

# Delete release branch
git branch -d release/phase-1-complete
git push origin --delete release/phase-1-complete
```

**Note:** Since `main` is protected, we cannot use `git flow release finish` to push directly. Instead, we create a PR for the release branch.

---

## Branch Protection (Configured)

### `main` Branch Protection ✅ ACTIVE

**Status:** Configured and enforced via GitHub API

Protection rules active:
- ✅ **Require pull request reviews** (0 approvals required for solo dev)
- ✅ **Dismiss stale reviews** when new commits pushed
- ✅ **Enforce for administrators** (no exceptions)
- ✅ **Block force pushes** (prevent history rewriting)
- ✅ **Block deletions** (cannot delete main branch)
- ❌ **No required status checks** (add CI/CD when ready)

**Result:** All changes to `main` must go through pull requests. Direct commits are blocked for everyone, including admins.

**To merge to main:**
1. Create PR from `develop` or `release/*` branch
2. Review and approve (or use "Squash and merge")
3. PR merged automatically updates `main`

### `develop` Branch Protection

**Status:** Not protected (allows direct commits)

Recommended practices:
- ⚠️ Allow direct commits for small fixes and documentation
- ✅ Use feature branches for major work
- ✅ Keep develop in working state (all commits should build)

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

### Current Strategy: Pre-1.0 Development

**Version 1.0.0 reserved for when app is determined to be truly usable.**

Format: `0.MINOR.PATCH` (pre-release development)

**Phase-to-Version Mapping:**
- `0.1.0` - Phase 1 complete (Foundation & Workspace) ← Current
- `0.2.0` - Phase 2 complete (Zed Terminal Integration)
- `0.3.0` - Phase 3 complete (File/Folder Browser)
- `0.4.0` - Phase 4 complete (Markdown Viewer - MVP features done)
- `0.5.0` - Phase 5 complete (Markdown Editor)
- `1.0.0` - **Usability validated** (when determined ready by maintainer)

**Patch versions (0.x.y):**
- Use for hotfixes and minor improvements between phases
- Example: `0.1.1` for bug fixes during Phase 1

**Git Tags:**
- Version tags: `v0.1.0`, `v0.2.0`, etc. (match Cargo.toml version)
- Phase milestone tags: `phase-1-complete`, `phase-2-complete` (descriptive)
- Both tag types can coexist

**Semantic Versioning after 1.0:**
- `MAJOR.MINOR.PATCH` format (standard SemVer)
- **MAJOR:** Breaking changes
- **MINOR:** New features (backwards-compatible)
- **PATCH:** Bug fixes (backwards-compatible)

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
