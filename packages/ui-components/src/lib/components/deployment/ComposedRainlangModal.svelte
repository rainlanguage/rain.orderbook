<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';
	import { lightCodeMirrorTheme } from '../../utils/codeMirrorThemes';
	import { Button, Modal } from 'flowbite-svelte';
	import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';

	export let gui: DotrainOrderGui;
	export let codeMirrorStyles = {};

	let rainlangText: string | null = null;
	let open = false;

	async function generateRainlang() {
		if (!gui) return;
		const rainlang = await gui.getComposedRainlang();
		if (rainlang) {
			rainlangText = rainlang;
			open = true;
		}
	}
</script>

<Button color="light" size="lg" on:click={generateRainlang}>Show Rainlang</Button>

<Modal size="xl" class="bg-opacity-90 backdrop-blur-sm" bind:open data-testid="modal">
	<div data-testid="modal-content">
		<CodeMirror
			value={rainlangText}
			extensions={[RainlangLR]}
			theme={lightCodeMirrorTheme}
			readonly={true}
			styles={{
				'&': {
					height: '70vh',
					width: '70vw'
				},
				...codeMirrorStyles
			}}
		/>
	</div>
</Modal>
