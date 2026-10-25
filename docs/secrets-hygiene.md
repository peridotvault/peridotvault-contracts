# Secrets hygiene (ANT-504)

## Rule

Real environment files are **never committed**. `chains/evm/.env` was
accidentally tracked and is now untracked + git-ignored. Rotation of the
exposed wallet/API key and any history rewrite are owned by the board (human)
under ANT-502 — agents must not attempt either.

## Setup (every clone/worktree, one command)

```sh
git config core.hooksPath .githooks
```

Install gitleaks for the full scan (the hook runs a conservative fallback
scan when gitleaks is absent):

- https://github.com/gitleaks/gitleaks#installing

Verify the hook is active:

```sh
git config core.hooksPath   # must print: .githooks
```

## What the hook does

1. **Fail closed on real env files.** Staging `.env` / `.env.<name>` is
   rejected; only `*.env.example` may be committed.
2. **Content scan.** `gitleaks protect --staged` with the repo-local
   `.gitleaks.toml`, or a fallback pattern scan for private-key/seed
   assignments (values redacted in output — secrets never land in logs).

## If the hook fires

- The commit is blocked; nothing left the machine. Remove the secret from
  the staged files (keep it in your untracked working-tree `.env`) and retry.
- If a secret was **already pushed**, do not rewrite history yourself.
  Escalate to the board (human) — rotation + purge are board-owned (ANT-502).

## Env reference

Copy `chains/evm/.env.example` to `chains/evm/.env` and fill in locally.
The `.example` file documents every variable the EVM scripts/config read,
with placeholder-only values.
