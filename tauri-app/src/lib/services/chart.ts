import type { ChartData } from '$lib/typeshare/fuzz';
import { invoke } from '@tauri-apps/api';

export async function makeChartData(dotrain: string, settings: string): Promise<ChartData[]> {
  console.log('makeChartData')
  return await invoke("make_charts", {
     dotrain, settings
  });
}