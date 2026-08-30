/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      // Backed by the CSS variables in assets/styles/theme.css. The
      // `<alpha-value>` placeholder is what lets `text-fg/40` and friends
      // keep working against a themed token.
      colors: {
        bg: 'rgb(var(--velo-bg) / <alpha-value>)',
        chrome: 'rgb(var(--velo-chrome) / <alpha-value>)',
        surface: 'rgb(var(--velo-surface) / <alpha-value>)',
        inset: 'rgb(var(--velo-inset) / <alpha-value>)',
        fg: 'rgb(var(--velo-fg) / <alpha-value>)',
        accent: 'rgb(var(--velo-accent) / <alpha-value>)',
        success: 'rgb(var(--velo-success) / <alpha-value>)',
        warning: 'rgb(var(--velo-warning) / <alpha-value>)',
        danger: 'rgb(var(--velo-danger) / <alpha-value>)',
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          '"Segoe UI"',
          'Roboto',
          'Helvetica',
          'Arial',
          'sans-serif',
        ],
      },
    },
  },
  plugins: [],
}
