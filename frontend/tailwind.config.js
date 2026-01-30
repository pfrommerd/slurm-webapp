/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'slurm-dark': '#0f172a',
        'slurm-panel': '#1e293b',
        'slurm-accent': '#38bdf8',
        'slurm-success': '#22c55e',
        'slurm-warning': '#eab308',
        'slurm-error': '#ef4444',
      }
    },
  },
  plugins: [],
}
