import { neutral, indigo } from 'tailwindcss/colors';

const config = {
  content: [
    './src/**/*.{html,js,svelte,ts}',
    '../node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}',
  ],

  plugins: [require('flowbite/plugin')],

  darkMode: 'class',

  theme: {
    extend: {
      fontFamily: {
        sans: ['DM Sans', 'sans-serif'],
      },
      colors: {
        primary: { ...indigo },
        gray: { ...neutral },
      },
      zIndex: {
        100: '100',
      },
    },
  },
};

module.exports = config;
