import type { LayoutLoad } from './$types';

export const load: LayoutLoad = async ({ fetch }) => {
	try {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/strategies/dev/registry'
		);
		const files = await response.text();

		const _files = files
			.split('\n')
			.filter((line: string) => line.trim())
			.map((line: string) => {
				const [name, url] = line.split(' ');
				return { name, url };
			});

		return {
			files: _files
		};
	} catch {
		return {
			files: []
		};
	}
};
