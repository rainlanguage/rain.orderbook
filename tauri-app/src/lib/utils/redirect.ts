import { goto } from '$app/navigation';
import { isSettingsDefined } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function redirectIfSettingsNotDefined() {
  if(!get(isSettingsDefined)) goto('/settings');
}