# Transaction Builders

Transaction builders should compose one or more instruction builders into a transaction-ready object.

Rules:

- Do not sign
- Do not send
- Do not confirm
- Do not hide required accounts
- Return enough metadata for the caller to inspect the transaction

Example shape:

```ts
const result = await buildBuyGameTransaction({
  environment: "testnet",
  buyer,
  game,
  paymentMint
});
```

The caller should still own wallet interaction, signing, submission, confirmation, and retries.
