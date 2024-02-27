<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { codeMirrorTheme } from '$lib/stores/darkMode';
	import { yaml } from '@codemirror/lang-yaml';

	export let value: string;
	export let disabled = false;
	export let styles = {};
</script>

<CodeMirror
	bind:value
	lang={yaml()}
	theme={$codeMirrorTheme}
	readonly={disabled}
	useTab={true}
	tabSize={2}
	styles={{
		"&": {
			width: "100%",
		},
		...styles
	}}
/>