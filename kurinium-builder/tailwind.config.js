/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{svelte,js,ts}'],
  theme: {
    extend: {
      colors: {
        'cyber-red': '#ff003c',
        'cyber-red-light': '#ff4d6a',
        'cyber-red-dark': '#cc0030',
        'cyber-red-darker': '#8b0020',
        'cyber-yellow': '#f9f002',
        'cyber-green': '#8ae66e',
        'cyber-neon': '#39ff14',
        'cyber-blue': '#0c5f74',
        'cyber-orange': '#ff9800',
        'cyber-purple': '#a855f7',
        'cyber-black': '#000000',
        'cyber-black-light': '#1a1a1a',
        'cyber-gray': '#2a2a2a',
        'cyber-white': '#ffffff',
        'cyber-silver': '#c0c0c0',
      },
      fontFamily: {
        'cyber': ['"Barlow"', 'sans-serif'],
        'cyber-title': ['"Oxanium"', 'sans-serif'],
      },
      animation: {
        'glitch': 'glitch 0.9s linear infinite',
        'glitch-slow': 'glitch 3s linear infinite',
        'scan-h': 'scanH 3s linear infinite alternate',
        'scan-v': 'scanV 9s linear infinite alternate',
        'pulse-glow': 'pulseGlow 2s ease-in-out infinite',
        'button-hover': 'buttonHover 0.3s ease-out',
      },
      keyframes: {
        glitch: {
          '0%': { transform: 'skew(-3deg)', marginLeft: '-2px' },
          '10%': { transform: 'skew(3deg)', marginLeft: '0' },
          '11%': { transform: 'skew(0deg)', marginLeft: '2px' },
          '50%': { transform: 'skew(0deg)', marginLeft: '0' },
          '51%': { transform: 'skew(3deg)', marginLeft: '5px' },
          '59%': { transform: 'skew(-3deg)', marginLeft: '5px' },
          '60%': { transform: 'skew(0deg)', marginLeft: '0' },
          '100%': { transform: 'skew(0deg)' },
        },
        scanH: {
          '0%': { top: '-27px' },
          '100%': { top: 'calc(100% + 12px)' },
        },
        scanV: {
          '0%': { left: '0px' },
          '100%': { left: '100%' },
        },
        pulseGlow: {
          '0%, 100%': { boxShadow: '0 0 5px #ff003c, 0 0 10px #ff003c' },
          '50%': { boxShadow: '0 0 10px #ff003c, 0 0 20px #ff003c, 0 0 30px #ff003c' },
        },
        buttonHover: {
          '0%': { transform: 'skew(0deg)' },
          '100%': { transform: 'skew(-2deg)' },
        },
      },
    },
  },
  plugins: [],
}
