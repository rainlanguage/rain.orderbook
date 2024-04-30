import type { ScenariosAuthoringMeta } from '$lib/typeshare/dotrainOrder';
import { invoke } from '@tauri-apps/api';

export const getAuthoringMetas = async (
  dotrain: string,
  settings: string,
): Promise<ScenariosAuthoringMeta> => invoke('get_authoring_metas', { dotrain, settings });
