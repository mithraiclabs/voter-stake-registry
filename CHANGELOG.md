# Voter Stake Registry Changelog

## v0.2.4 - 2022-5-4 - not on mainnet

### Program
- Upgrade Anchor to v0.24.2

## v0.2.3 - 2022-4-29 - not on mainnet

### Program
- Use spl-governance 2.2.1 as dependency, instead of a specific commit.

### Typescript Client
- Upgrade the Anchor dependency to v0.24.2

## v0.2.2 - skipped

## v0.2.1 - 2022-4-3 - mainnet deploy slot 129520307

### Program
- Increase the maximum number of lockup periods to 200 * 365 to allow for 200-year cliff and
  constant lockups.
- Add a function to compute the guaranteed locked vote power bonus. This is unused by the
  program itself, but helpful for programs that want to provide benefits based on a user's
  lockup amount and time.

### Other
- Add cli tool to decode voter accounts.
- Update dependencies.


## v0.2.0 - 2022-2-14 - mainnet deploy slot 121129331

- First release.
- Available on devnet at VotEn9AWwTFtJPJSMV5F9jsMY6QwWM5qn3XP9PATGW7
- In use by the Mango DAO on mainnet at the same address.
