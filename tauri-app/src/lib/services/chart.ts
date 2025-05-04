import type { ChartData, DeploymentsDebugDataMap } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';

export const makeChartData = async (dotrain: string): Promise<ChartData> =>
  invoke('make_charts', { dotrain });

export const makeDeploymentsDebugDataMap = async (
  dotrain: string,
  settings: string,
  blockNumbers?: Record<string, number>,
): Promise<DeploymentsDebugDataMap> =>
  invoke('make_deployment_debug', { dotrain, settings, blockNumbers });
