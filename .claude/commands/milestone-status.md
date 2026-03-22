Show the current status of all GitHub milestones and their issues.

Steps:
1. Fetch all milestones: `GITHUB_TOKEN= gh api repos/:owner/:repo/milestones --jq '.[] | "\(.number)|\(.title)|\(.open_issues)|\(.closed_issues)"'`
2. For each milestone, fetch its issues: `GITHUB_TOKEN= gh issue list --milestone "<name>" --state all --json number,title,state,labels --jq '.[] | "\(.state) #\(.number): \(.title)"'`
3. Present a summary table per milestone showing:
   - Total issues (open/closed)
   - Progress percentage
   - List of open issues grouped by label (breaking-change, architecture, performance, observability, enhancement)
   - Dependencies between issues (look for "Depends on #" in issue bodies)
4. Highlight any blocked issues (dependencies not yet closed)
5. Suggest which issues are ready to work on next (no open dependencies)
