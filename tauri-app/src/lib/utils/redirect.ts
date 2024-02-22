import { goto } from '$app/navigation';
import { hasRequiredSettings } from '$lib/stores/settings';

export async function redirectIfMissingRequiredSettings() {
  const hasRequiredSettingsVal = await hasRequiredSettings.load();

  if(!hasRequiredSettingsVal) goto('/settings');
}