import { pickBy, isNil } from 'lodash';
import type { Config, ConfigSource } from '@rainlanguage/orderbook';

export function pickDeployments(
  mergedConfigSource: ConfigSource | undefined,
  mergedConfig: Config | undefined,
  chainId: number,
) {
  return !isNil(mergedConfigSource) &&
    !isNil(mergedConfigSource?.deployments) &&
    !isNil(mergedConfigSource?.orders)
    ? pickBy(
        mergedConfigSource.deployments,
        (d) => mergedConfig?.scenarios?.[d.scenario].deployer.network.chainId === chainId,
      )
    : {};
}

export function pickScenarios(mergedConfig: Config | undefined, chainId: number) {
  return !isNil(mergedConfig)
    ? pickBy(mergedConfig.scenarios, (d) => d.deployer.network.chainId === chainId)
    : {};
}
