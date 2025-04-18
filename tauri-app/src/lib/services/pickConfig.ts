import { pickBy, isNil } from 'lodash';
import type { Config } from '@rainlanguage/orderbook';

export function pickDeployments(
  mergedConfig: Config | undefined,
  activeNetworkRef: string | undefined,
) {
  return !isNil(mergedConfig) && !isNil(mergedConfig?.deployments) && !isNil(mergedConfig?.orders)
    ? pickBy(
        mergedConfig.deployments,
        (d) =>
          mergedConfig?.scenarios?.[d.scenario.key]?.deployer?.network?.key === activeNetworkRef,
      )
    : {};
}

export function pickScenarios(
  mergedConfig: Config | undefined,
  activeNetworkRef: string | undefined,
) {
  return !isNil(mergedConfig)
    ? pickBy(mergedConfig.scenarios, (d) => d?.deployer?.network?.key === activeNetworkRef)
    : {};
}
