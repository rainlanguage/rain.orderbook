import type { ChartData } from '$lib/typeshare/config';
import { invoke } from '@tauri-apps/api';

export const makeChartData = async (dotrain: string, settings: string): Promise<ChartData[]> => invoke("make_charts", { dotrain, settings });