<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RainlangLR } from 'codemirror-rainlang';
	import { lightCodeMirrorTheme } from '../../utils/codeMirrorThemes';
	import { Button, Modal } from 'flowbite-svelte';

	export let composeRainlang: () => Promise<string | undefined>;
	export let codeMirrorStyles = {};

	let rainlangText: string | null = null;
	let open = false;

	async function generateRainlang() {
		const rainlang = await composeRainlang();
		if (rainlang) {
			rainlangText = rainlang;
			open = true;
		}
	}
</script>

<Button size="lg" on:click={generateRainlang}>Show Rainlang</Button>

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
