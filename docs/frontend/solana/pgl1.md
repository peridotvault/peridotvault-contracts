# PGL-1 Architecture

PGL-1 is the PeridotVault game licensing program.

Primary responsibilities:

- Initialize global PGL configuration
- Track authorized actors
- Track creator state
- Create game accounts
- Mint and renew license accounts
- Update game publisher and metadata

Core accounts:

- `pgl_config`
- `authorized_actor`
- `creator_state`
- `game`
- `license`

PGL-1 is the authority for license state. Store and registry flows should treat PGL-1 accounts as canonical when proving ownership or purchase eligibility.
