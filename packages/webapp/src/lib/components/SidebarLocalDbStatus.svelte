<script lang="ts">
	import { localDbLatestEntry, localDbStatusIndicator } from '$lib/stores/localDbStatus';
	import { derived } from 'svelte/store';

	const indicatorBadge = derived(localDbStatusIndicator, ($indicator) => {
		const badgeBase =
			$indicator.variant === 'error'
				? 'bg-red-500/15 text-red-400 ring-1 ring-inset ring-red-500/30'
				: $indicator.variant === 'success'
					? 'bg-emerald-500/15 text-emerald-400 ring-1 ring-inset ring-emerald-500/30'
					: $indicator.variant === 'warning'
						? 'bg-amber-500/15 text-amber-500 ring-1 ring-inset ring-amber-500/30'
						: 'bg-zinc-500/10 text-zinc-400 ring-1 ring-inset ring-zinc-500/20';

		const dotClass =
			$indicator.variant === 'error'
				? 'bg-red-400'
				: $indicator.variant === 'success'
					? 'bg-emerald-400'
					: $indicator.variant === 'warning'
						? 'bg-amber-400'
						: 'bg-zinc-400';

		const label =
			$indicator.variant === 'idle'
				? 'Paused'
				: $indicator.variant === 'success'
					? 'Active'
					: $indicator.variant === 'error'
						? 'Error'
						: 'Active';

		return {
			...$indicator,
			badgeClass: badgeBase,
			dotClass,
			displayLabel: label
		};
	});

	const latestEntry = derived(localDbLatestEntry, ($entry) =>
		$entry
			? {
					...$entry,
					formattedTime: new Date($entry.timestamp).toLocaleTimeString([], {
						hour: '2-digit',
						minute: '2-digit',
						second: '2-digit'
					})
				}
			: null
	);
</script>

<div
	class="flex flex-col gap-2 rounded-lg border border-gray-200/70 bg-white/70 p-3 text-xs text-gray-600 shadow-sm backdrop-blur-sm dark:border-gray-700/70 dark:bg-gray-900/50 dark:text-gray-300"
>
	<div class="flex items-center justify-between">
		<div class="flex flex-col">
			<span class="text-[10px] font-semibold uppercase tracking-[0.12em] text-gray-500 dark:text-gray-400"
				>Local Index</span
			>
		</div>
	<span
		class={`inline-flex items-center gap-1 rounded-full px-2 py-[3px] text-[10px] font-semibold uppercase tracking-wide ${$indicatorBadge.badgeClass}`}
	>
		<span class={`size-1.5 rounded-full ${$indicatorBadge.dotClass}`}></span>
		<span class="truncate max-w-[8rem]">{$indicatorBadge.displayLabel}</span>
	</span>
	</div>

	{#if $indicatorBadge.variant === 'error'}
		<p class="rounded-md bg-red-500/10 px-2 py-1 text-[11px] text-red-500 dark:bg-red-500/15 dark:text-red-300">
			Sync loop reported an error. Latest message shown below.
		</p>
	{/if}

	<div>
		{#if $latestEntry}
			<div class="flex gap-2 rounded-md border border-gray-200/80 bg-white/70 px-2 py-[6px] dark:border-gray-700/70 dark:bg-gray-900/60">
				<span class="mt-[1px] shrink-0 text-[10px] font-semibold uppercase tracking-wide text-gray-400 dark:text-gray-500"
					>{$latestEntry.formattedTime}</span
				>
				<p
					title={$latestEntry.message}
					class="line-clamp-3 text-[11px] leading-snug text-gray-600 dark:text-gray-200"
				>
					{$latestEntry.message}
				</p>
			</div>
		{:else}
			<p class="rounded-md bg-gray-100/70 px-2 py-2 text-[11px] text-gray-500 dark:bg-gray-800/60 dark:text-gray-400">
				Status updates will appear here once the local indexer runs.
			</p>
		{/if}
	</div>
</div>
