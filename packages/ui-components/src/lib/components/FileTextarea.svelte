<script lang="ts">
	import { ButtonLoading } from '@rainlanguage/ui-components';

	export let textFile;
</script>

<div class="flex w-full flex-col">
	<div class="flex items-end gap-x-4">
		<div class="flex-grow">
			<slot name="alert" />
		</div>
		<div class="flex flex-col items-end justify-end gap-2">
			<div class="flex gap-x-2">
				{#if $textFile.path}
					<ButtonLoading loading={$textFile.isSaving} color="green" on:click={textFile.saveFile}
						>Save</ButtonLoading
					>
				{/if}
				<ButtonLoading loading={$textFile.isSavingAs} color="green" on:click={textFile.saveFileAs}
					>Save As</ButtonLoading
				>
				<ButtonLoading loading={$textFile.isLoading} color="blue" on:click={textFile.loadFile}
					>Load</ButtonLoading
				>
			</div>

			{#if $textFile.path}
				<div
					class="flex w-full justify-end overflow-hidden overflow-ellipsis text-xs text-gray-500 dark:text-gray-400"
				>
					{$textFile.path}
				</div>
			{/if}
		</div>
	</div>
</div>

<div class="my-4 overflow-hidden rounded-lg border dark:border-none">
	<slot name="textarea" />
</div>

<div class="my-4">
	<slot name="additionalFields" />
</div>

<div class="flex justify-end">
	<slot name="submit" />
</div>
