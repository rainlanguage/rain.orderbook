import { pickBy, isNil } from 'lodash';
import type { Config, ConfigSource } from '@rainlanguage/orderbook/js_api';

export function pickDeployments(
  mergedConfigSource: ConfigSource | undefined,
  mergedConfig: Config | undefined,
  activeNetworkRef: string | undefined,
) {
  return !isNil(mergedConfigSource) &&
    !isNil(mergedConfigSource?.deployments) &&
    !isNil(mergedConfigSource?.orders)
    ? pickBy(
        mergedConfigSource.deployments,
        (d) => mergedConfig?.scenarios?.[d.scenario]?.deployer?.network?.key === activeNetworkRef,
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
