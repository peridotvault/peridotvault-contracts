export class PeridotContractsError extends Error {
  readonly code: string;

  constructor(message: string, code = "PERIDOT_CONTRACTS_ERROR") {
    super(message);
    this.name = "PeridotContractsError";
    this.code = code;
  }
}
