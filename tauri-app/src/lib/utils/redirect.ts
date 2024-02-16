import { goto } from '$app/navigation';
import { allRequiredSettingsValid } from '$lib/stores/settings';
import { get } from 'svelte/store';

export function redirectIfSettingsNotDefined() {
  if(!get(allRequiredSettingsValid)) goto('/settings');
}