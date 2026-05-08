# Account Fetchers

Account fetchers should read typed program accounts using the deployed program ID and IDL.

Expected conventions:

- Accept a connection or provider from the caller
- Accept environment-specific program resolution
- Return typed account data
- Avoid global clients or hidden RPC configuration

Useful fetcher groups:

- PGL-1 config, game, creator state, license
- Registry config, payment token, registry game, publish grant
- Store config, payment option, purchase receipt

Fetchers should be thin wrappers over the IDL account layout and should not enforce business policy beyond decoding and basic validation.
