<script lang="ts" generics="T">
	import { ButtonLoading, IconError } from '@rainlanguage/ui-components';
	import { RawRainlangExtension } from 'codemirror-rainlang';
	import {
		IconInfo,
		IconLedger,
		IconSuccess,
		IconTelegram,
		IconWalletConnect,
		IconWarning,
		IconExternalLink,
		CodeMirrorDotrain,
		darkCodeMirrorTheme
	} from '@rainlanguage/ui-components';
	let isLoading: boolean = true;
	let rainlangText: string = 'Hello there';
	const rainlangExtension = new RawRainlangExtension({
		hover: async () => null,
		completion: async () => null,
		diagnostics: async () => []
	});
</script>

<div>
	<ButtonLoading color="primary" class="w-full px-2 py-1" size="lg" pill loading={isLoading}
		>Loading button</ButtonLoading
	>
	<IconError />
	<IconExternalLink svgClass="!h-2 !w-2 mr-1" />
	<IconInfo />
	<IconLedger />
	<IconSuccess />
	<IconTelegram dark />
	<IconWalletConnect />
	<IconWarning />
	<CodeMirrorDotrain
		codeMirrorTheme={darkCodeMirrorTheme}
		{rainlangText}
		disabled={false}
		styles={{ '&': { minHeight: '400px' } }}
		{rainlangExtension}
	/>
</div>
