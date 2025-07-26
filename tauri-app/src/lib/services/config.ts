import { settingsText } from '$lib/stores/settings';
import { globalDotrainFile } from '$lib/storesGeneric/textFileStore';
import type { DeploymentCfg, ScenarioCfg } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';
import { get } from 'svelte/store';

export const checkSettingsErrors = (text: string[]): Promise<void> =>
  invoke('check_settings_errors', { text });

export const checkDotrainWithSettingsErrors = (
  dotrain: string,
  settings: string[],
): Promise<void> =>
  invoke('check_dotrain_with_settings_errors', {
    dotrain,
    settings,
  });

export const getDeployments = (): Promise<Record<string, DeploymentCfg>> => {
  if (!get(globalDotrainFile).text) {
    return Promise.resolve({});
  }
  return invoke('get_deployments', {
    dotrain: get(globalDotrainFile).text,
    settings: get(settingsText),
  });
};

export const getScenarios = (): Promise<Record<string, ScenarioCfg>> => {
  if (!get(globalDotrainFile).text) {
    return Promise.resolve({});
  }
  return invoke('get_scenarios', {
    dotrain: get(globalDotrainFile).text,
    settings: get(settingsText),
  });
};
