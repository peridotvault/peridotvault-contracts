# Registry Architecture

The Registry program coordinates game registration and publishing policy.

Primary responsibilities:

- Initialize registry configuration
- Manage accepted payment tokens
- Create and update publish grants
- Create games through PGL-1 and register them
- Track registry game status

Core accounts:

- `registry_config`
- `accepted_payment_token`
- `registry_game`
- `publish_grant`

The Registry is the policy layer between publishers, PGL-1, and the Game Store.
