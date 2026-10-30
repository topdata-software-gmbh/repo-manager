import preset from '@topdata-software-gmbh/design-tokens/tailwind-preset';

/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  presets: [preset],
  theme: { extend: {} },
  plugins: [],
};
