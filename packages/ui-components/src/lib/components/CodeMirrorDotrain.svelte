<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { RawRainlangExtension } from 'codemirror-rainlang';
	import { openLintPanel } from '@codemirror/lint';

	export let value: string;
	export let disabled = false;
	export let styles = {};
	export let rainlangExtension: RawRainlangExtension;
	export let codeMirrorTheme;
</script>

<CodeMirror
	bind:value
	extensions={[rainlangExtension]}
	theme={codeMirrorTheme}
	readonly={disabled}
	useTab={true}
	tabSize={2}
	styles={{
		'&': {
			width: '100%'
		},
		...styles
	}}
	on:ready={(e) => {
		openLintPanel(e.detail);
	}}
/>

<style global>
	:global(.ͼ1 .cm-panel.cm-panel-lint ul [aria-selected]) {
		background-color: inherit;
	}
</style>
