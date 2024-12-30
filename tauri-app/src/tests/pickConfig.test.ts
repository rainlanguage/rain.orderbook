import { expect, test } from 'vitest';
import type { Dictionary } from 'lodash';
import { pickDeployments, pickScenarios } from '$lib/services/pickConfig';
import type { Config, ConfigSource, DeploymentConfigSource, Scenario } from '$lib/typeshare/config';

export const config: Config = {
  networks: {
    network1: {
      key: 'network1',
      rpc: 'rpc-url',
      'chain-id': 14,
    },
    network2: {
      key: 'network2',
      rpc: 'rpc-url',
      'chain-id': 137,
    },
  },
  subgraphs: {
    network1: 'some-url',
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
        'chain-id': 14,
      },
      subgraph: 'some-url',
    },
  },
  deployers: {
    network1: {
      key: 'network1',
      address: '0xabcdef',
      network: {
        key: 'network1',
        rpc: 'rpc-url',
        'chain-id': 14,
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
        'chain-id': 14,
      },
    },
    sell: {
      key: 'sell',
      inputs: [],
      outputs: [],
      network: {
        key: 'network1',
        rpc: 'rpc-url',
        'chain-id': 14,
      },
    },
  },
  scenarios: {
    'network1.sell': {
      name: 'network1.sell',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          'chain-id': 14,
        },
      },
    },
    network1: {
      name: 'network1',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          'chain-id': 14,
        },
      },
    },
    'network1.buy': {
      name: 'network1.buy',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          'chain-id': 14,
        },
      },
    },
  },
  charts: {},
  deployments: {
    sell: {
      scenario: {
        name: 'network1.sell',
        bindings: {},
        deployer: {
          key: 'network1',
          address: '0xabcdef',
          network: {
            key: 'network1',
            rpc: 'rpc-url',
            'chain-id': 14,
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
          'chain-id': 14,
        },
      },
    },
    buy: {
      scenario: {
        name: 'network1.buy',
        bindings: {},
        deployer: {
          key: 'network1',
          address: '0xabcdef',
          network: {
            key: 'network1',
            rpc: 'rpc-url',
            'chain-id': 14,
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
          'chain-id': 14,
        },
      },
    },
  },
  accounts: {
    name_one: 'address_one',
    name_two: 'address_two',
  },
};

export const configSource: ConfigSource = {
  networks: {
    network1: {
      rpc: 'rpc-url',
      'chain-id': 14,
    },
    network2: {
      rpc: 'rpc-url',
      'chain-id': 137,
    },
  },
  subgraphs: {
    network1: 'some-url',
  },
  orderbooks: {
    network1: {
      address: '0x123456',
      network: 'network1',
      subgraph: 'network1',
    },
  },
  deployers: {
    network1: {
      address: '0xabcdef',
      network: 'network1',
    },
  },
  orders: {
    sell: {
      inputs: [],
      outputs: [],
    },
    buy: {
      inputs: [],
      outputs: [],
    },
  },
  scenarios: {
    network1: {
      bindings: {},
      scenarios: {
        buy: {
          bindings: {},
        },
        sell: {
          bindings: {},
        },
      },
    },
  },
  charts: {},
  deployments: {
    buy: {
      scenario: 'network1.buy',
      order: 'buy',
    },
    sell: {
      scenario: 'network1.sell',
      order: 'sell',
    },
  },
  accounts: {
    name_one: 'address_one',
    name_two: 'address_two',
  },
};

test('pick deployments', () => {
  const activeNetwork = 'network1';
  const result = pickDeployments(configSource, config, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {
    sell: {
      scenario: 'network1.sell',
      order: 'sell',
    },
    buy: {
      scenario: 'network1.buy',
      order: 'buy',
    },
  };

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick deployments when empty', () => {
  const activeNetwork = 'network2';
  const result = pickDeployments(configSource, config, activeNetwork);
  const expectedPickedDeployments: Dictionary<DeploymentConfigSource> = {};

  expect(result).toStrictEqual(expectedPickedDeployments);
});

test('pick scenarios', () => {
  const activeNetwork = 'network1';
  const result = pickScenarios(config, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {
    'network1.sell': {
      name: 'network1.sell',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          'chain-id': 14,
        },
      },
    },
    'network1.buy': {
      name: 'network1.buy',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          'chain-id': 14,
        },
      },
    },
    network1: {
      name: 'network1',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpc: 'rpc-url',
          'chain-id': 14,
        },
      },
    },
  };

  expect(result).toStrictEqual(expectedPickedScenarios);
});

test('pick scenarios when empty', () => {
  const activeNetwork = 'network2';
  const result = pickScenarios(config, activeNetwork);
  const expectedPickedScenarios: Dictionary<Scenario> = {};

  expect(result).toStrictEqual(expectedPickedScenarios);
});
