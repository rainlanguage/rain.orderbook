import { pickBy, isNil } from 'lodash';
import type { Config } from '@rainlanguage/orderbook';

export function pickDeployments(
  mergedConfig: Config | undefined,
  activeNetworkRef: string | undefined,
) {
  return !isNil(mergedConfig) &&
    !isNil(mergedConfig?.dotrainOrder.deployments) &&
    !isNil(mergedConfig?.dotrainOrder.orders)
    ? pickBy(
        mergedConfig.dotrainOrder.deployments,
        (d) =>
          mergedConfig?.dotrainOrder.scenarios?.[d.scenario.key]?.deployer?.network?.key ===
          activeNetworkRef,
      )
    : {};
}

export function pickScenarios(
  mergedConfig: Config | undefined,
  activeNetworkRef: string | undefined,
) {
  return !isNil(mergedConfig)
    ? pickBy(
        mergedConfig.dotrainOrder.scenarios,
        (d) => d?.deployer?.network?.key === activeNetworkRef,
      )
    : {};
}
