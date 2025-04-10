import type { ChartData, DeploymentDebugData } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';

export const makeChartData = async (dotrain: string): Promise<ChartData> =>
  invoke('make_charts', { dotrain });

export const makeDeploymentDebugData = async (
  dotrain: string,
  blockNumber?: number,
): Promise<DeploymentDebugData> => invoke('make_deployment_debug', { dotrain, blockNumber });
