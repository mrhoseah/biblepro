import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        surface: {
          950: '#07070E',
          900: '#0C0C17',
          800: '#11111F',
          700: '#171728',
          600: '#1E1E34',
          500: '#262640',
        },
        bdr: {
          DEFAULT: '#27274A',
          subtle: '#1C1C35',
          strong: '#38386A',
        },
        accent: {
          DEFAULT: '#C8A050',
          dim: '#A07838',
          bright: '#E0B860',
          glow: 'rgba(200,160,80,0.15)',
        },
        ink: {
          DEFAULT: '#E2DFEE',
          muted: '#8E8CAA',
          faint: '#535270',
        },
        live:   '#3FC870',
        danger: '#C04444',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Consolas', 'monospace'],
        serif: ['Georgia', 'serif'],
      },
      fontSize: {
        '2xs': ['10px', '14px'],
        xs: ['11px', '16px'],
        sm: ['12px', '18px'],
      },
      boxShadow: {
        glow:    '0 0 24px rgba(200,160,80,0.18)',
        panel:   '0 4px 24px rgba(0,0,0,0.5)',
        card:    '0 2px 12px rgba(0,0,0,0.4)',
      },
    },
  },
  plugins: [],
} satisfies Config;
