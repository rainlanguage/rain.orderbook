import { expect, test } from 'vitest';
import type { Dictionary } from 'lodash';
import { pickDeployments, pickScenarios } from '$lib/services/pickConfig';
import type { Config, DeploymentCfg, ScenarioCfg } from '@rainlanguage/orderbook';

export const config: Config = {
  networks: {
    network1: {
      key: 'network1',
      rpc: 'rpc-url',
      chainId: 14,
    },
    network2: {
      key: 'network2',
      rpc: 'rpc-url',
      chainId: 137,
    },
  },
  subgraphs: {
    network1: {
      key: 'some-key',
      url: 'some-url',
    },
  },
  metaboards: {
    network1: 'some-url',
  },
  orderbooks: {
    network1: {
      key: 'network1',
      address: '0x123456',
      network: {
        key: 'network1',
        rpc: 'rpc-url',
        chainId: 14,
      },
      subgraph: {
        key: 'some-key',
        url: 'some-url',
      },
    },
  },
  deployers: {
    network1: {
      key: 'network1',
      address: '0xabcdef',
      network: {
        key: 'network1',
        rpc: 'rpc-url',
        chainId: 14,
      },
    },
  },
  tokens: {},
  orders: {
    buy: {
      key: 'buy',
      inputs: [],
      outputs: [],
      network: {
        key: 'network1',
        rpc: 'rpc-url',
        chainId: 14,
      },
    },
    sell: {
      key: 'sell',
      inputs: [],
      outputs: [],
      network: {
        key: 'network1',
        rpc: 'rpc-url',
        chainId: 14,
      },
    },
  },
  scenarios: {
    'network1.sell': {
      key: 'network1.sell',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
    network1: {
      key: 'network1',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
    'network1.buy': {
      key: 'network1.buy',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
  },
  charts: {},
  deployments: {
    sell: {
      key: 'sell',
      scenario: {
        key: 'network1.sell',
        bindings: {},
        deployer: {
          key: 'network1',
          address: '0xabcdef',
          network: {
            key: 'network1',
            rpc: 'rpc-url',
            chainId: 14,
          },
        },
      },
      order: {
        key: 'sell',
        inputs: [],
        outputs: [],
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
    buy: {
      key: 'buy',
      scenario: {
        key: 'network1.buy',
        bindings: {},
        deployer: {
          key: 'network1',
          address: '0xabcdef',
          network: {
            key: 'network1',
            rpc: 'rpc-url',
            chainId: 14,
          },
        },
      },
      order: {
        key: 'buy',
        inputs: [],
        outputs: [],
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
  },
  accounts: {
    name_one: 'address_one',
    name_two: 'address_two',
  },
};

test('pick deployments', () => {
  const activeNetwork = 'network1';
  const result = pickDeployments(config, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentCfg> = {
    sell: {
      key: 'sell',
      scenario: {
        key: 'network1.sell',
        bindings: {},
        deployer: {
          key: 'network1',
          address: '0xabcdef',
          network: {
            key: 'network1',
            rpc: 'rpc-url',
            chainId: 14,
          },
        },
      },
      order: {
        key: 'sell',
        inputs: [],
        outputs: [],
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
    buy: {
      key: 'buy',
      scenario: {
        key: 'network1.buy',
        bindings: {},
        deployer: {
          key: 'network1',
          address: '0xabcdef',
          network: {
            key: 'network1',
            rpc: 'rpc-url',
            chainId: 14,
          },
        },
      },
      order: {
        key: 'buy',
        inputs: [],
        outputs: [],
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
  };

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick deployments when empty', () => {
  const activeNetwork = 'network2';
  const result = pickDeployments(config, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentCfg> = {};

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick scenarios', () => {
  const activeNetwork = 'network1';
  const result = pickScenarios(config, activeNetwork);
  const expectedPickedScenarios: Dictionary<ScenarioCfg> = {
    'network1.sell': {
      key: 'network1.sell',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
    'network1.buy': {
      key: 'network1.buy',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
    network1: {
      key: 'network1',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          chainId: 14,
        },
      },
    },
  };

  expect(result).toStrictEqual(expectedPickedScenarios);
});

test('pick scenarios when empty', () => {
  const activeNetwork = 'network2';
  const result = pickScenarios(config, activeNetwork);
  const expectedPickedScenarios: Dictionary<ScenarioCfg> = {};

  expect(result).toStrictEqual(expectedPickedScenarios);
});
