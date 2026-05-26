import * as deployments from "./deployments/index.js";
import * as solana from "./solana/index.js";

const evm = {
  getEvmContractAddress({ chain, environment, contract }) {
    return `0x_${chain}_${contract}_${environment}`;
  },
};

export { deployments, evm, solana };
