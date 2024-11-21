import type { Config } from 'tailwindcss';
import { neutral, indigo } from 'tailwindcss/colors';

export default {
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		'../../node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}',
		'../../node_modules/@rainlanguage/ui-components/**/*.{html,js,svelte,ts}',
		'../ui-components/**/*.{html,js,svelte,ts}'
	],

	theme: {
		extend: {
			fontFamily: {
				sans: ['DM Sans', 'sans-serif']
			},
			colors: {
				primary: { ...indigo },
				gray: { ...neutral }
			},
			zIndex: {
				100: '100'
			}
		}
	},

	plugins: []
} satisfies Config;
