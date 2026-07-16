# Blockchain Roadmap

Agent Passport starts local and offchain. The first Rust API gives frontend developers stable endpoints for player IDs, passports, and proof receipts without pretending that chain infrastructure is live.

## Current Mode: Offchain Devnet

```text
agent identity
-> normalized identity JSON
-> security gate
-> agent passport
-> offchain proof receipt
-> deterministic player ID
```

Current guarantees:

- deterministic hashes;
- deterministic player IDs;
- local proof receipts;
- frontend-ready JSON;
- no private keys;
- no wallet custody;
- no RPC calls;
- no live verification claim.

## Rust API

Run:

```bash
pnpm rust:api
```

Endpoints:

```text
GET  /health
POST /v1/passports/build
POST /v1/passports/verify
POST /v1/players/issue-id
```

The API reports:

```json
{
  "network": "offchain-devnet",
  "liveChain": false
}
```

## Future Chain Adapter

The future adapter should anchor hashes, not secrets:

```text
passportHash + receiptId + playerId
-> chain transaction
-> transaction signature
-> receipt metadata update
```

The chain adapter can be Solana, Base, or another network. It must sit outside the core passport builder.

## Wallet Model

Recommended model:

```text
Agent Passport = identity and proof layer
Arena Wallet = account and spending layer
```

Agent Passport should not create or store private keys. Later Arena infrastructure can link:

- a user-owned wallet;
- an arena-managed smart account;
- an agent-specific wallet address;
- a purchase inventory for skills, trials, items, or access.

## Status Changes

V1 values:

```json
{
  "verificationStatus": "declared",
  "proofStatus": "offchain",
  "network": "offchain-devnet"
}
```

Future anchored values require real transaction evidence:

```json
{
  "proofStatus": "anchored",
  "network": "solana-devnet",
  "transactionSignature": "..."
}
```

Do not emit `anchored`, `verified`, or `mainnet` status until that adapter exists and tests prove it.

