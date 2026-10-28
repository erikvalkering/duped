# Copilot Instructions for duped-android

## Commit and Tag Policy

- For every small change or fix, always create a separate commit.
- After each commit, create a new tag in the format `v0.0.xx` (no leading zeros, e.g., v0.0.82, not v0.0.082), incrementing from the latest tag.
- Push both the commit and the tag to the remote repository immediately.
- This ensures that each change triggers a new APK build in GitHub Actions.

## General Guidance

- Keep commits focused and atomic (one logical change per commit).
- Update this file if the workflow changes.

---
Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
