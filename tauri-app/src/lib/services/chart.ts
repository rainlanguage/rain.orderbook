import type { ChartData } from '@rainlanguage/orderbook/js_api';
import { invoke } from '@tauri-apps/api';

export const makeChartData = async (dotrain: string, settings: string): Promise<ChartData> =>
  invoke('make_charts', { dotrain, settings });
