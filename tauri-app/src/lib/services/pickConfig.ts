import { pickBy } from 'lodash';
import type { DeploymentCfg, ScenarioCfg } from '@rainlanguage/orderbook';

export function pickDeployments(
  deployments: Record<string, DeploymentCfg>,
  scenarios: Record<string, ScenarioCfg>,
  chainId: number,
) {
  const filtered = pickBy(
    deployments,
    (d) => scenarios[d.scenario.key].deployer.network.chainId === chainId,
  );
  const result: Record<string, { scenario: string; order: string }> = {};

  for (const [key, deployment] of Object.entries(filtered)) {
    result[key] = {
      scenario: deployment.scenario.key,
      order: deployment.order.key,
    };
  }

  return result;
}

export function pickScenarios(scenarios: Record<string, ScenarioCfg>, chainId: number) {
  return pickBy(scenarios, (d) => d.deployer.network.chainId === chainId);
}
