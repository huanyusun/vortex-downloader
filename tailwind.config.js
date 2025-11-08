/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: '#3B82F6',
          hover: '#2563EB',
          light: '#60A5FA',
        },
        success: {
          DEFAULT: '#10B981',
          hover: '#059669',
        },
        error: {
          DEFAULT: '#EF4444',
          hover: '#DC2626',
        },
        warning: {
          DEFAULT: '#F59E0B',
        },
        bg: {
          primary: '#1F2937',
          secondary: '#374151',
          tertiary: '#4B5563',
        },
        text: {
          primary: '#F9FAFB',
          secondary: '#D1D5DB',
          muted: '#9CA3AF',
        },
      },
      fontFamily: {
        heading: ['"SF Pro Display"', 'system-ui', 'sans-serif'],
        body: ['"SF Pro Text"', 'system-ui', 'sans-serif'],
      },
      fontSize: {
        'xs': '0.75rem',
        'sm': '0.875rem',
        'base': '1rem',
        'lg': '1.125rem',
        'xl': '1.25rem',
      },
      lineHeight: {
        tight: '1.25',
        normal: '1.5',
        relaxed: '1.75',
      },
      spacing: {
        '18': '4.5rem',
        '22': '5.5rem',
      },
      borderRadius: {
        'xl': '0.75rem',
        '2xl': '1rem',
      },
    },
  },
  plugins: [],
}
