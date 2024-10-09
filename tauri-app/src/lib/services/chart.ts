import type { ChartData, DeploymentDebugData } from '$lib/typeshare/config';
import { invoke } from '@tauri-apps/api';

export const makeChartData = async (dotrain: string, settings: string): Promise<ChartData> =>
  invoke('make_charts', { dotrain, settings });

export const makeDeploymentDebugData = async (
  dotrain: string,
  settings: string,
  blockNumber?: number,
): Promise<DeploymentDebugData> =>
  invoke('make_deployment_debug', { dotrain, settings, blockNumber });
