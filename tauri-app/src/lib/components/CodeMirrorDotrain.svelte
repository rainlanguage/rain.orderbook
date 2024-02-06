<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { parseDotrain } from '../utils/parseDotrain';
	import { RainlangExtension, type LanguageServicesConfig, RainDocument, type Problem, MetaStore } from 'codemirror-rainlang';
	import { codeMirrorTheme } from '$lib/stores/darkMode';

	export let value: string;
	export let disabled = false;

	const callback = async(dotrain: RainDocument): Promise<Problem[]> => parseDotrain(dotrain);
	const metaStore = new MetaStore(false);
	const config: LanguageServicesConfig = {
		hover: true,
		completion: true,
		callback,
		metaStore
	};
	const rainlangExtension = new RainlangExtension(config);
</script>

<CodeMirror
	bind:value
	extensions={[rainlangExtension]}
	theme={$codeMirrorTheme}
	readonly={disabled}
	useTab={true}
	tabSize={2}
	styles={{
		"&": {
			width: "100%",
			minHeight: "400px",
		},
	}}
/>