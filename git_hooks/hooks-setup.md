# Git Hooks Setup Guide

## Introduction
This guide explains how to ensure that Git hooks, like the `pre-commit` hook, work correctly in your local environment. 

In our repository, we use a pre-commit hook to automatically regenerate the `index.md` file for the development logs whenever changes are made to the `doc/dev-log` directory. We alao add a formatter using ruff to automatically format the Python code before commiting.

## Steps to Enable the Pre-Commit Hook

### 1. Make the Pre-Commit Hook Executable
Git hooks are stored as scripts in the `.git/hooks` directory, but they need to have executable permissions to run. 

If the pre-commit hook is not running, it's likely that the file is not marked as executable.

Run the following command from the root of your repository to make the `pre-commit` hook executable:

```bash
chmod +x .git/hooks/pre-commit
```

### 2. Ensure the Path to the Hook Script is Correct
In case the pre-commit hook references external scripts, ensure that the relative path to these scripts is correct.

For our project, the `pre-commit` hook runs the following script to regenerate the index for development logs:

```sh
./scripts/dev-log/generate_index.py
```

Make sure that this path is correct relative to the root of the repository. If needed, you can modify the hook script to use the full path like so:

```sh
REPO_ROOT=$(git rev-parse --show-toplevel)
INDEX_SCRIPT="$REPO_ROOT/scripts/dev-log/generate_index.py"
```

### 3. Test the Hook
To verify that the `pre-commit` hook is functioning as expected, you can test it by committing changes to any file in the `doc/dev-log` directory. The pre-commit hook should run and regenerate the `index.md` file before completing the commit.

### 4. Check Git Status
After the hook runs, ensure that the changes to `index.md` are added to the commit automatically. You can verify this by running:

```bash
git status
```

If the `index.md` file is not staged for commit, it may be necessary to manually stage it using the following command inside the hook script:

```bash
git add doc/dev-log/index.md
```

## Conclusion
By ensuring the pre-commit hook is executable and correctly configured, you can automate common tasks like regenerating the development log index during the commit process. This helps streamline your workflow and maintain consistency in the repository.
