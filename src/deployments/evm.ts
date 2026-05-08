import type { EvmChain, PeridotEnvironment, EvmDeployment } from "../core";

const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";
const TODO_VERSION = "TODO";
const INTERFACE_VERSION = "0.1.0";

export const EVM_DEPLOYMENTS: Record<
  EvmChain,
  Record<PeridotEnvironment, EvmDeployment>
> = {
  ethereum: {
    local: {
      chain: "ethereum",
      environment: "local",
      chainId: 31337,
      contracts: {
        pgc1: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        }
      }
    },
    staging: {
      chain: "ethereum",
      environment: "staging",
      chainId: 11155111,
      contracts: {
        pgc1: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        }
      }
    },
    testnet: {
      chain: "ethereum",
      environment: "testnet",
      chainId: 11155111,
      contracts: {
        pgc1: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        }
      }
    },
    mainnet: {
      chain: "ethereum",
      environment: "mainnet",
      chainId: 1,
      contracts: {
        pgc1: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        }
      }
    }
  },
  base: {
    local: {
      chain: "base",
      environment: "local",
      chainId: 31337,
      contracts: {
        pgc1: {
          address: "0x5fbdb2315678afecb367f032d93f642f64180aa3",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: "0xe7f1725e7734ce288f8367e1bb143e90bb3f0512",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: "0xe55a693527d8CD166a9b814BfFdAA5Adb65DB5aB",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: "0x9fe46736679d2d9a65f0992f2272de9f3c7fa6e0",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        }
      }
    },
    staging: {
      chain: "base",
      environment: "staging",
      chainId: 84532,
      contracts: {
        pgc1: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        }
      }
    },
    testnet: {
      chain: "base",
      environment: "testnet",
      chainId: 84532,
      contracts: {
        pgc1: {
          address: "0xdacc68d4b4f26da689f63466bccdf935567839d0",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: "0x2091278674cec58296f4e7b868c2c84f5942abad",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: "0xe55a693527d8CD166a9b814BfFdAA5Adb65DB5aB",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: "0x21c1deffc44dbe3d742caa704f94f2772ffb6567",
          version: "0.1.0",
          interfaceVersion: INTERFACE_VERSION
        }
      }
    },
    mainnet: {
      chain: "base",
      environment: "mainnet",
      chainId: 8453,
      contracts: {
        pgc1: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        registry: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        treasuryRouter: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        },
        pgc1Factory: {
          address: ZERO_ADDRESS,
          version: TODO_VERSION,
          interfaceVersion: INTERFACE_VERSION
        }
      }
    }
  }
};

export function getEvmDeployment(params: {
  chain: EvmChain;
  environment: PeridotEnvironment;
}): EvmDeployment {
  return EVM_DEPLOYMENTS[params.chain][params.environment];
}
