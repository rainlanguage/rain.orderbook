import { describe, it, expect, vi, beforeEach } from 'vitest';
import { getDeploymentsNetworks } from '../utils/getDeploymentNetworks';
import type { DeploymentCfg } from '@rainlanguage/orderbook';


vi.mock('viem/chains', () => ({
  mainnet: { id: 1, name: 'Ethereum' },
  polygon: { id: 137, name: 'Polygon' },
  arbitrum: { id: 42161, name: 'Arbitrum One' },
}));

describe('getDeploymentsNetworks', () => {
  it('should return undefined if deployments is undefined', () => {
    expect(getDeploymentsNetworks(undefined)).toBeUndefined();
  });

  it('should return undefined if deployments is an empty object', () => {
    expect(getDeploymentsNetworks({})).toBeUndefined();
  });

  it('should correctly map a single deployment with a known chain name', () => {
    const deployments: Record<string, DeploymentCfg> = {
      dep1: {
        scenario: {
          deployer: { network: { chainId: 1, key: 'eth' } },
        },
      } as DeploymentCfg,
    };
    expect(getDeploymentsNetworks(deployments)).toEqual({ 1: 'Ethereum' });
  });

  it('should use network.key if chain name is not found in viem/chains', () => {
    const deployments: Record<string, DeploymentCfg> = {
      dep1: {
        scenario: {
          deployer: { network: { chainId: 9999, key: 'customTestNet' } },
        },
      } as DeploymentCfg,
    };
    expect(getDeploymentsNetworks(deployments)).toEqual({ 9999: 'customTestNet' });
  });

  it('should correctly map multiple unique deployments', () => {
    const deployments: Record<string, DeploymentCfg> = {
      dep1: {
        scenario: {
          deployer: { network: { chainId: 1, key: 'eth' } },
        },
      } as DeploymentCfg,
      dep2: {
        scenario: {
          deployer: { network: { chainId: 137, key: 'matic' } },
        },
      } as DeploymentCfg,
      dep3: {
        scenario: {
          deployer: { network: { chainId: 5, key: 'goerli' } }, // Goerli not in mock, should use key
        },
      } as DeploymentCfg,
    };
    expect(getDeploymentsNetworks(deployments)).toEqual({
      1: 'Ethereum',
      137: 'Polygon',
      5: 'goerli',
    });
  });

  it('should handle multiple deployments on the same chainId, taking the first encountered network name/key', () => {
    const deployments: Record<string, DeploymentCfg> = {
      dep1: {
        scenario: {
          deployer: { network: { chainId: 1, key: 'mainnet-primary' } },
        },
      } as DeploymentCfg, // Should use Ethereum from viem/chains mock
      dep2: {
        scenario: {
          deployer: { network: { chainId: 1, key: 'mainnet-secondary' } },
        },
      } as DeploymentCfg, // This will be ignored for chainId 1 as it's already processed
      dep3: {
        scenario: {
          deployer: { network: { chainId: 137, key: 'polygon-custom-name' } },
        },
      } as DeploymentCfg, // Should use Polygon from viem/chains mock
    };
    expect(getDeploymentsNetworks(deployments)).toEqual({
      1: 'Ethereum',
      137: 'Polygon',
    });
  });

  it('should return undefined if all deployments result in no valid network entries (e.g., only if Object.keys(networks).length is 0)', () => {
    expect(getDeploymentsNetworks({})).toBeUndefined();
  });
}); 