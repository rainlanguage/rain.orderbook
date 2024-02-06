<script lang="ts">
	import { lightTheme } from '../utils/codeMirrorThemes';
	import CodeMirror from 'svelte-codemirror-editor';
	import { parseDotrain } from '../utils/parseDotrain';
	import { RainlangExtension, type LanguageServicesConfig, RainDocument, type Problem, MetaStore } from 'codemirror-rainlang';
	import { rpcUrl } from '$lib/stores/settings';

	export let value: string;

	const callback = async(dotrain: RainDocument): Promise<Problem[]> => parseDotrain(dotrain, $rpcUrl, 5000);
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
	theme={lightTheme}
	useTab={true}
	tabSize={2}
	styles={{
	"&": {
			width: "100%",
			minHeight: "400px"
	},
}}
/>
