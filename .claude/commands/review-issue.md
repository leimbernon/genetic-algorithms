Analyze GitHub issue #$ARGUMENTS and produce an implementation plan.

Steps:
1. Fetch the issue details: `GITHUB_TOKEN= gh issue view $ARGUMENTS --json title,body,labels,milestone`
2. Read the issue body, acceptance criteria, and dependencies
3. Identify all files that need to be created or modified by searching the codebase
4. Check if the issue has the `breaking-change` label — if so, evaluate the implementation options listed in the issue and recommend the least-breaking approach
5. Check dependencies: are prerequisite issues completed? `GITHUB_TOKEN= gh issue view <dep> --json state`
6. Identify the correct milestone branch for this issue
7. Produce a detailed implementation plan with:
   - Files to create/modify (with paths)
   - Trait/struct/enum changes needed
   - Test strategy
   - Estimated complexity (S/M/L)
   - Any risks or open questions

Follow the branching strategy: feature branches from milestone branches, never from main.
