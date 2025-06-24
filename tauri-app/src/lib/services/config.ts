import { settingsText } from '$lib/stores/settings';
import type { Config, ConfigSource, NewConfig } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';
import { get } from 'svelte/store';

export const parseConfig = async (text: string, validate = false): Promise<NewConfig> =>
  invoke('parse_new_configstring', { text, validate });

export const mergeDotrainConfigWithSettings = async (dotrain: string): Promise<ConfigSource> =>
  invoke('merge_configstrings', { dotrain, configText: get(settingsText) });

export const convertConfigstringToConfig = async (configString: ConfigSource): Promise<Config> =>
  invoke('convert_configstring_to_config', { configString });
