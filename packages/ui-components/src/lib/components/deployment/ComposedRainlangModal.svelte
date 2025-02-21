<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';
	import { lightCodeMirrorTheme } from '../../utils/codeMirrorThemes';
	import { Button, Modal } from 'flowbite-svelte';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;

	let rainlangText: string | null = null;
	let open = false;

	async function generateRainlang() {
		const rainlang = await gui.getComposedRainlang();
		if (rainlang) {
			rainlangText = rainlang;
			open = true;
		}
	}
</script>

<Button color="light" size="lg" on:click={generateRainlang}>Show Rainlang</Button>

<Modal size="xl" class="bg-opacity-90  backdrop-blur-sm" bind:open data-testid="modal">
	<div data-testid="modal-content">
		<h3 class="mb-2 text-2xl font-semibold text-gray-900 dark:text-white">Generated Rainlang</h3>
		<CodeMirror
			value={rainlangText}
			extensions={[RainlangLR]}
			theme={lightCodeMirrorTheme}
			readonly={true}
			styles={{
				'&': {
					height: '70vh',
					width: '100%'
				}
			}}
		/>
	</div>
</Modal>
