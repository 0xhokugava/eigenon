# Issue Triage

This document describes how maintainers classify and prepare Eigenon issues.
Its purpose is to keep the tracker actionable and make contribution labels
consistent.

## Initial Review

For each new issue:

1. Check for an existing issue or pull request covering the same work.
2. Confirm that the report contains enough context to reproduce or evaluate it.
3. Apply one primary type label.
4. Apply any relevant area or contribution labels.
5. Record unresolved design questions before implementation begins.
6. Link related issues and pull requests.

An assignee should indicate active ownership. Do not assign an issue only to
signal interest or long-term responsibility.

## Type Labels

* `bug`: Existing behavior is incorrect or inconsistent with documentation.
* `documentation`: Documentation is missing, unclear or outdated.
* `enhancement`: A new capability or user-facing improvement.
* `question`: More information or maintainer direction is required.
* `refactor`: Internal restructuring intended to preserve behavior.
* `testing`: Test coverage, validation infrastructure or test reliability.

Use `math` when an issue depends on a mathematical convention, derivation or
correctness argument. Use `invitation` for an explicit request for community
feedback rather than an implementation task.

## Contribution Labels

### `good first issue`

Apply this label only when all of the following are true:

* the task is small and self-contained
* expected behavior and acceptance criteria are explicit
* the relevant files or modules are identified
* no unresolved architecture or product decision blocks implementation
* the result can be verified locally with a documented command
* no assignee or active pull request already owns the task

A good first issue should help a contributor learn one part of the project. It
should not require broad repository knowledge, performance research or an API
redesign.

### `help wanted`

Use this label when the scope is accepted and outside contributions are
welcome, but the work may require more context or experience than a first
issue.

## Issue Lifecycle

* Link a pull request from the issue and keep the issue open until the change is
  merged or a maintainer decides not to proceed.
* Close duplicate issues with a link to the canonical issue.
* Use `invalid` when the reported behavior cannot be reproduced or does not
  describe an Eigenon problem.
* Use `wontfix` when the issue is understood but intentionally outside the
  project scope.
* Remove `good first issue` and `help wanted` when work is assigned or an active
  pull request exists.

## Preparing a Good First Issue

The issue body should include:

1. A short problem statement.
2. Relevant file paths or functions.
3. A bounded list of changes.
4. Acceptance criteria.
5. Exact validation commands.
6. Any qubit-ordering or numerical conventions involved.
