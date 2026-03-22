Start working on a GitHub issue by setting up the correct branch.

Usage: /start-issue <issue-number>

Steps:
1. Fetch issue details: `GITHUB_TOKEN= gh issue view $ARGUMENTS --json title,milestone,labels`
2. Determine the milestone name from the issue
3. Check if the milestone branch exists: `git branch -a | grep milestone/`
4. If milestone branch does NOT exist:
   - `git checkout main && git pull origin main`
   - Create it: `git checkout -b milestone/<milestone-name-kebab-case>`
   - Push it: `GITHUB_TOKEN= git push -u origin milestone/<milestone-name-kebab-case>`
5. Checkout the milestone branch and pull latest: `git checkout milestone/<name> && git pull`
6. Create the feature branch: `git checkout -b feat/$ARGUMENTS-<short-description>`
7. Confirm the branch is ready and show the issue acceptance criteria

IMPORTANT: Never create feature branches from main. Always from the milestone branch.
