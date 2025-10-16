import { derived, writable } from 'svelte/store';

export type LocalDbStatusLevel = 'idle' | 'info' | 'success' | 'warning' | 'error';

export interface LocalDbStatusEntry {
	id: string;
	message: string;
	timestamp: number;
	level: LocalDbStatusLevel;
}

function classifyMessage(message: string): LocalDbStatusLevel {
	const normalized = message.toLowerCase();

	if (normalized.includes('error') || normalized.includes('fail')) {
		return 'error';
	}

	if (
		normalized.includes('complete') ||
		normalized.includes('synced') ||
		normalized.includes('success')
	) {
		return 'success';
	}

	if (normalized.includes('init') || normalized.includes('start')) {
		return 'info';
	}

	return 'info';
}

const latestEntry = writable<LocalDbStatusEntry | null>(null);
const syncEnabled = writable<boolean>(false);
export const localDbLatestEntry = derived(latestEntry, ($entry) => $entry);

export const localDbStatusIndicator = derived([latestEntry, syncEnabled], ([$entry, $enabled]) => {
	if (!$enabled) {
		return {
			variant: 'idle' as const,
			label: 'Sync paused'
		};
	}

	if (!$entry) {
		return {
			variant: 'info' as const,
			label: 'Waiting for updates...'
		};
	}

	const variant =
		$entry.level === 'error' ? 'error' : $entry.level === 'success' ? 'success' : 'info';

	return {
		variant,
		label: $entry.message
	};
});

export function recordLocalDbStatus(message: string, levelOverride?: LocalDbStatusLevel): void {
	const entry: LocalDbStatusEntry = {
		id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
		message,
		timestamp: Date.now(),
		level: levelOverride ?? classifyMessage(message)
	};

	latestEntry.set(entry);
}

export function setLocalDbSyncEnabled(enabled: boolean): void {
	syncEnabled.set(enabled);
}

export function recordLocalDbError(message: string): void {
	recordLocalDbStatus(message, 'error');
}
