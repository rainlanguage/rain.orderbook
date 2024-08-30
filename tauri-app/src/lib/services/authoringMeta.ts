import type { ScenarioWords } from '$lib/typeshare/authoringMeta';
import { invoke } from '@tauri-apps/api';

export const getAuthoringMetaV2ForScenarios = async (
  dotrain: string,
  settings?: string,
): Promise<ScenarioWords[]> => invoke('get_authoring_meta_v2_for_scenarios', { dotrain, settings });
