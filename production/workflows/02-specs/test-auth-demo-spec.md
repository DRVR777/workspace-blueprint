# Auth Demo Spec

## Scope
Build an interactive authentication flow demo showing OAuth 2.0 login with
GitHub. Single page, demonstrates the complete login → token → user profile flow.

## Acceptance Criteria
- [ ] User can click "Login with GitHub" button
- [ ] OAuth flow completes in the demo environment (mocked or real)
- [ ] User profile (name, avatar, public repos count) displays after auth
- [ ] Error state shown if auth fails

## Technical Approach
Web-based demo. React preferred. Statically deployable.

## Dependencies
- AuthFlow component from component-library (if it covers this use case)
- GitHub OAuth app (or mock)

## Done When
All acceptance criteria checked, tested on Chrome latest, deployable to Vercel.
