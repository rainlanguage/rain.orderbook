import type { DeploymentCfg } from '@rainlanguage/orderbook';
import * as chains from 'viem/chains';

/**
 * Gathers key/value pairs of deployments chain ids
 * against their network name to be used for debug modal.
 * @param deployments - Record of deployments.
 * @returns Record of chainIds to network names, or undefined if no deployments.
 */
export function getDeploymentsNetworks(
  deployments: Record<string, DeploymentCfg> | undefined,
): Record<number, string> | undefined {
  if (deployments) {
    const networks: Record<number, string> = {};
    for (const key in deployments) {
      const chainId = deployments[key].scenario.deployer.network.chainId;
      if (!networks[chainId]) {
        const networkKey =
          Object.values(chains).find((v) => v.id === chainId)?.name ??
          deployments[key].scenario.deployer.network.key;
        networks[chainId] = networkKey;
      }
    }
    if (!Object.keys(networks).length) return undefined;
    else return networks;
  }
  return undefined;
} 