import { expect, test } from 'vitest';
import { pickDeployments, pickScenarios } from '$lib/services/pickConfig';
import type { DeploymentCfg, ScenarioCfg } from '@rainlanguage/orderbook';

const deployments: Record<string, DeploymentCfg> = {
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
          rpcs: ['rpc-url'],
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
        rpcs: ['rpc-url'],
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
          rpcs: ['rpc-url'],
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
        rpcs: ['rpc-url'],
        chainId: 14,
      },
    },
  },
};

const scenarios: Record<string, ScenarioCfg> = {
  'network1.sell': {
    key: 'network1.sell',
    bindings: {},
    deployer: {
      key: 'network1',
      address: '0xabcdef',
      network: {
        key: 'network1',
        rpcs: ['rpc-url'],
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
        rpcs: ['rpc-url'],
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
        rpcs: ['rpc-url'],
        chainId: 14,
      },
    },
  },
};

test('pick deployments', () => {
  const activeNetwork = 14;
  const result = pickDeployments(deployments, scenarios, activeNetwork);
  expect(result).toStrictEqual({
    sell: {
      scenario: 'network1.sell',
      order: 'sell',
    },
    buy: {
      scenario: 'network1.buy',
      order: 'buy',
    },
  });
});

test('pick deployments when empty', () => {
  const activeNetwork = 137;
  const result = pickDeployments(deployments, scenarios, activeNetwork);
  expect(result).toStrictEqual({});
});

test('pick scenarios', () => {
  const activeNetwork = 14;
  const result = pickScenarios(scenarios, activeNetwork);
  expect(result).toStrictEqual({
    'network1.sell': {
      key: 'network1.sell',
      bindings: {},
      deployer: {
        key: 'network1',
        address: '0xabcdef',
        network: {
          key: 'network1',
          rpcs: ['rpc-url'],
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
          rpcs: ['rpc-url'],
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
          rpcs: ['rpc-url'],
          chainId: 14,
        },
      },
    },
  });
});

test('pick scenarios when empty', () => {
  const activeNetwork = 137;
  const result = pickScenarios(scenarios, activeNetwork);
  expect(result).toStrictEqual({});
});
