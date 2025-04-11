import { settingsText } from '$lib/stores/settings';
import type { Config, ConfigSource } from '@rainlanguage/orderbook';
import { invoke } from '@tauri-apps/api';
import { get } from 'svelte/store';

export const parseConfigSource = async (text: string): Promise<ConfigSource> =>
  invoke('parse_configstring', { text });

export const mergeDotrainConfigWithSettings = async (dotrain: string): Promise<ConfigSource> =>
  invoke('merge_configstrings', { dotrain, configText: get(settingsText) });

export const convertConfigstringToConfig = async (configString: ConfigSource): Promise<Config> =>
  invoke('convert_configstring_to_config', { configString });
