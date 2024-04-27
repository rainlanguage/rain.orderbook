import { expect, test } from 'vitest'
import type { Dictionary } from 'lodash';
import { pickDeployments, pickScenarios } from '$lib/services/pickConfig';
import type { Config, ConfigSource, DeploymentConfigSource, Scenario } from '$lib/typeshare/config';

export const config: Config = {
  "networks": {
    "network1": {
      "name": "network1",
      "rpc": "rpc-url",
      "chain-id": 14,
    },
    "network2": {
      "name": "network2",
      "rpc": "rpc-url",
      "chain-id": 137,
    }
  },
  "subgraphs": {
    "network1": "some-url"
  },
  "metaboards": {
    "network1": "some-url"
  },
  "orderbooks": {
    "network1": {
      "address": "0x123456",
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
      },
      "subgraph": "some-url",
    }
  },
  "deployers": {
    "network1": {
      "address": "0xabcdef",
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
      },
    }
  },
  "tokens": {},
  "orders": {
    "buy": {
      "inputs": [],
      "outputs": [],
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
      },
    },
    "sell": {
      "inputs": [],
      "outputs": [],
      "network": {
        "name": "network1",
        "rpc": "rpc-url",
        "chain-id": 14,
      },
    }
  },
  "scenarios": {
    "network1.sell": {
      "name": "network1.sell",
      "bindings": {},
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    },
    "network1": {
      "name": "network1",
      "bindings": {},
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    },
    "network1.buy": {
      "name": "network1.buy",
      "bindings": {},
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    }
  },
  "charts": {},
  "deployments": {
    "sell": {
      "scenario": {
        "name": "network1.sell",
        "bindings": {},
        "deployer": {
          "address": "0xabcdef",
          "network": {
            "name": "network1",
            "rpc": "rpc-url",
            "chain-id": 14,
          },
        }
      },
      "order": {
        "inputs": [],
        "outputs": [],
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    },
    "buy": {
      "scenario": {
        "name": "network1.buy",
        "bindings": {},
        "deployer": {
          "address": "0xabcdef",
          "network": {
            "name": "network1",
            "rpc": "rpc-url",
            "chain-id": 14,
          },
        }
      },
      "order": {
        "inputs": [],
        "outputs": [],
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    }
  }
}


export const configSource: ConfigSource = {
  "networks": {
    "network1": {
      "rpc": "rpc-url",
      "chain-id": 14,
    },
    "network2": {
      "rpc": "rpc-url",
      "chain-id": 137,
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
    }
  },
  "deployers": {
    "network1": {
      "address": "0xabcdef",
      "network": "network1",
    }
  },
  "orders": {
    "sell": {
      "inputs": [],
      "outputs": [],
    },
    "buy": {
      "inputs": [],
      "outputs": [],
    }
  },
  "scenarios": {
    "network1": {
      "bindings": {},
      "scenarios": {
        "buy": {
          "bindings": {},
        },
        "sell": {
          "bindings": {},
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
  }
}

test('pick deployments', () => {
  const activeNetwork = "network1";
  const result = pickDeployments(configSource, config, activeNetwork);
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
  const result = pickDeployments(configSource, config, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {};

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick scenarios', () => {
  const activeNetwork = "network1";
  const result = pickScenarios(config, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {
    "network1.sell": {
      "name": "network1.sell",
      "bindings": {},
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    },
    "network1.buy": {
      "name": "network1.buy",
      "bindings": {},
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    },
    "network1": {
      "name": "network1",
      "bindings": {},
      "deployer": {
        "address": "0xabcdef",
        "network": {
          "name": "network1",
          "rpc": "rpc-url",
          "chain-id": 14,
        },
      }
    }
  };

  expect(result).toStrictEqual(expectedPickedScenarios);
});

test('pick scenarios when empty', () => {
  const activeNetwork = "network2";
  const result = pickScenarios(config, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {};

  expect(result).toStrictEqual(expectedPickedScenarios);
});