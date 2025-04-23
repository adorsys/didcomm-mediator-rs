# Release Guide

This document provides a step-by-step guide to creating a new release for the **didcomm-mediator-rs** by triggering the workflow.

---

## Prerequisites

Before proceeding, ensure you have the following:

- Write access to the repository.
- A local copy of the repository with the latest `main` branch.
- Git installed on your machine.
- Familiarity with the existing GitHub Actions workflow for the release process.

---

## Release Process

### Step 1: Create a New Branch

1. Pull the latest changes from the `main` branch to ensure you are up to date:
   ```bash
   git checkout main
   git pull origin main
   ```

2. Create a new branch for the release:
    ```bash
    git checkout -b release/vX.Y.Z
    ```
     Replace `vX.Y.Z` with the new version you are releasing, e.g., `v0.1.2`.  

### Step 2: Determine the Next Tag Version

1. Identify the latest tag in the repository:
     ```bash
     git tag --list --sort=-v:refname | head -n 1
     ```
     For example, if the latest tag is `v0.1.1`, the next tag should be `v0.1.2`.

2. Follow semantic versioning to increment the tag appropriately:

* Increment the patch version for bug fixes (e.g., `v0.1.1` → `v0.1.2`).
* Increment the minor version for new features (e.g., `v0.1.1` → `v0.2.0`).
* Increment the major version for breaking changes (e.g., `v0.1.1` → `v1.0.0`).

### Step 3: Create the New Tag

1. Create a new Git tag locally:
     ```bash
     git tag vX.Y.Z
     ```
     Replace vX.Y.Z with the new version, e.g., `v0.1.2`.

2. Push the tag to the remote repository:
     ```bash
     git push origin vX.Y.Z
    ```

### Step 4: Trigger the Release Workflow

* The GitHub Actions workflow is configured to trigger automatically on a tag push (e.g., `v0.1.2`).

* Monitor the progress of the workflow in the Actions tab on GitHub to ensure the release completes successfully.


### Notes on the Workflow

The workflow performs the following tasks automatically:

* Build and Test: The codebase is built and tested to ensure stability.

* Binary Artifact Creation: The release binary is saved as an artifact.

* GitHub Release Creation: A release is created in the repository, including:

    * The release binary as an artifact.

    * A release note template.
