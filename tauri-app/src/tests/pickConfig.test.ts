import { expect, test } from 'vitest'
import type { Dictionary } from 'lodash';
import { pickDeployments, pickScenarios } from '$lib/services/pickConfig';
import type { Config, ConfigSource, DeploymentConfigSource, Scenario } from '$lib/typeshare/config';

export const mockedConfig: Config = {
  "networks": {
    "network1": {
      "name": "network1",
      "rpc": "rpc-url",
      "chain-id": 14,
      "label": "some-label",
      "network-id": 14,
      "currency": "A"
    },
    "network2": {
      "name": "network2",
      "rpc": "rpc-url",
      "chain-id": 137,
      "label": "some-label",
      "network-id": 137,
      "currency": "B"
    }
  },
  "subgraphs": {
    "network1": "some-url"
  },
  "orderbooks": {
    "network1": {
      "address": "0x123456",
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
        "label": "some-label",
        "network-id": 14,
        "currency": "A"
      },
      "subgraph": "some-url",
      "label": "some-label"
    }
  },
  "tokens": {
    "wflr": {
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
        "label": "some-label",
        "network-id": 14,
        "currency": "A"
      },
      "address": "0x123abcd",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    },
    "weth": {
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
        "label": "some-label",
        "network-id": 14,
        "currency": "A"
      },
      "address": "0x9876efab",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    }
  },
  "deployers": {
    "network1": {
      "address": "0xabcdef",
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
        "label": "some-label",
        "network-id": 14,
        "currency": "A"
      },
      "label": "some-label"
    }
  },
  "orders": {
    "buy": {
      "inputs": [
        {
          "token": {
            "network": {
              "name": "network1",
              "rpc": "rpc-url",
              "chain-id": 14,
              "label": "some-label",
              "network-id": 14,
              "currency": "A"
            },
            "address": "0x123abcd",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0x1"
        }
      ],
      "outputs": [
        {
          "token": {
            "network": {
              "name": "network1",
              "rpc": "rpc-url",
              "chain-id": 14,
              "label": "some-label",
              "network-id": 14,
              "currency": "A"
            },
            "address": "0x9876efab",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0x1"
        }
      ],
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
        "label": "some-label",
        "network-id": 14,
        "currency": "A"
      },
      "deployer": undefined,
      "orderbook": undefined
    },
    "sell": {
      "inputs": [
        {
          "token": {
            "network": {
              "name": "network1",
              "rpc": "rpc-url",
              "chain-id": 14,
              "label": "some-label",
              "network-id": 14,
              "currency": "A"
            },
            "address": "0x9876efab",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0x1"
        }
      ],
      "outputs": [
        {
          "token": {
            "network": {
              "name": "network1",
              "rpc": "rpc-url",
              "chain-id": 14,
              "label": "some-label",
              "network-id": 14,
              "currency": "A"
            },
            "address": "0x123abcd",
            "decimals": 18,
            "label": undefined,
            "symbol": undefined
          },
          "vault-id": "0x1"
        }
      ],
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
        "label": "some-label",
        "network-id": 14,
        "currency": "A"
      },
      "deployer": undefined,
      "orderbook": undefined
    }
  },
  "scenarios": {
    "network1.sell": {
      "name": "network1.sell",
      "bindings": {},
      "runs": 1,
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "label": "some-label"
      }
    },
    "network1": {
      "name": "network1",
      "bindings": {},
      "runs": undefined,
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "label": "some-label"
      }
    },
    "network1.buy": {
      "name": "network1.buy",
      "bindings": {},
      "runs": undefined,
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "label": "some-label"
      }
    }
  },
  "charts": {},
  "deployments": {
    "sell": {
      "scenario": {
        "name": "network1.sell",
        "bindings": {},
        "runs": 1,
        "deployer": {
          "address": "0xabcdef",
          "network": {
            "name": "network1",
            "rpc": "rpc-url",
            "chain-id": 14,
            "label": "some-label",
            "network-id": 14,
            "currency": "A"
          },
          "label": "some-label"
        }
      },
      "order": {
        "inputs": [
          {
            "token": {
              "network": {
                "name": "network1",
                "rpc": "rpc-url",
                "chain-id": 14,
                "label": "some-label",
                "network-id": 14,
                "currency": "A"
              },
              "address": "0x9876efab",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0x1"
          }
        ],
        "outputs": [
          {
            "token": {
              "network": {
                "name": "network1",
                "rpc": "rpc-url",
                "chain-id": 14,
                "label": "some-label",
                "network-id": 14,
                "currency": "A"
              },
              "address": "0x123abcd",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0x1"
          }
        ],
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "deployer": undefined,
        "orderbook": undefined
      }
    },
    "buy": {
      "scenario": {
        "name": "network1.buy",
        "bindings": {},
        "runs": undefined,
        "deployer": {
          "address": "0xabcdef",
          "network": {
            "name": "network1",
            "rpc": "rpc-url",
            "chain-id": 14,
            "label": "some-label",
            "network-id": 14,
            "currency": "A"
          },
          "label": "some-label"
        }
      },
      "order": {
        "inputs": [
          {
            "token": {
              "network": {
                "name": "network1",
                "rpc": "rpc-url",
                "chain-id": 14,
                "label": "some-label",
                "network-id": 14,
                "currency": "A"
              },
              "address": "0x123abcd",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0x1"
          }
        ],
        "outputs": [
          {
            "token": {
              "network": {
                "name": "network1",
                "rpc": "rpc-url",
                "chain-id": 14,
                "label": "some-label",
                "network-id": 14,
                "currency": "A"
              },
              "address": "0x9876efab",
              "decimals": 18,
              "label": undefined,
              "symbol": undefined
            },
            "vault-id": "0x1"
          }
        ],
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "deployer": undefined,
        "orderbook": undefined
      }
    }
  },
  "sentry": undefined
}


export const mockedConfigSource: ConfigSource = {
  "networks": {
    "network1": {
      "rpc": "rpc-url",
      "chain-id": 14,
      "label": "some-label",
      "network-id": 14,
      "currency": "A"
    },
    "network2": {
      "rpc": "rpc-url",
      "chain-id": 137,
      "label": "some-label",
      "network-id": 137,
      "currency": "B"
    }
  },
  "subgraphs": {
    "network1": "some-url"
  },
  "orderbooks": {
    "network1": {
      "address": "0x123456",
      "network": "network1",
      "subgraph": "network1",
      "label": "some-label"
    }
  },
  "tokens": {
    "wflr": {
      "network": "network1",
      "address": "0x123abcd",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    },
    "weth": {
      "network": "network1",
      "address": "0x9876efab",
      "decimals": 18,
      "label": undefined,
      "symbol": undefined
    }
  },
  "deployers": {
    "network1": {
      "address": "0xabcdef",
      "network": "network1",
      "label": "some-label"
    }
  },
  "orders": {
    "sell": {
      "inputs": [
        {
          "token": "weth",
          "vault-id": 0x1n
        }
      ],
      "outputs": [
        {
          "token": "wflr",
          "vault-id": 0x1n
        }
      ],
      "deployer": undefined,
      "orderbook": undefined
    },
    "buy": {
      "inputs": [
        {
          "token": "wflr",
          "vault-id": 0x1n
        }
      ],
      "outputs": [
        {
          "token": "weth",
          "vault-id": 0x1n
        }
      ],
      "deployer": undefined,
      "orderbook": undefined
    }
  },
  "scenarios": {
    "network1": {
      "bindings": {},
      "runs": undefined,
      "deployer": undefined,
      "scenarios": {
        "buy": {
          "bindings": {},
          "runs": undefined,
          "deployer": undefined,
          "scenarios": undefined
        },
        "sell": {
          "bindings": {},
          "runs": 1,
          "deployer": undefined,
          "scenarios": undefined
        }
      }
    }
  },
  "charts": {},
  "deployments": {
    "buy": {
      "scenario": "network1.buy",
      "order": "buy"
    },
    "sell": {
      "scenario": "network1.sell",
      "order": "sell"
    }
  },
  "sentry": undefined
}

test('pick deployments', () => {
  const activeNetwork = "network1";
  const result = pickDeployments(mockedConfigSource, mockedConfig, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {
    "sell": {
      "scenario": "network1.sell",
      "order": "sell"
    },
    "buy": {
      "scenario": "network1.buy",
      "order": "buy"
    }
  };

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick deployments when empty', () => {
  const activeNetwork = "network2";
  const result = pickDeployments(mockedConfigSource, mockedConfig, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {};

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick scenarios', () => {
  const activeNetwork = "network1";
  const result = pickScenarios(mockedConfig, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {
    "network1.sell": {
      "name": "network1.sell",
      "bindings": {},
      "runs": 1,
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "label": "some-label"
      }
    },
    "network1.buy": {
      "name": "network1.buy",
      "bindings": {},
      "runs": undefined,
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "label": "some-label"
      }
    },
    "network1": {
      "name": "network1",
      "bindings": {},
      "runs": undefined,
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
          "label": "some-label",
          "network-id": 14,
          "currency": "A"
        },
        "label": "some-label"
      }
    }
  };

  expect(result).toStrictEqual(expectedPickedScenarios);
});

test('pick scenarios when empty', () => {
  const activeNetwork = "network2";
  const result = pickScenarios(mockedConfig, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {};

  expect(result).toStrictEqual(expectedPickedScenarios);
});