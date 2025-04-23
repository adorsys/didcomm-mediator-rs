# Release Guide

This document provides a step-by-step guide to creating a new release for the **didcomm-mediator-rs** project.

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

