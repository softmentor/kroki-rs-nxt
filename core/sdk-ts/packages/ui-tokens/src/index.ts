export const tokens = {
  bgDeep: '#120f2d',
  bgCard: 'rgba(45, 40, 85, 0.42)',
  accentPrimary: '#ff6f61',
  accentSecondary: '#ffb30f',
  textMain: '#f7f7fb',
  textDim: '#b9b9cf',
  gradMain: 'linear-gradient(135deg, #ffb30f, #ff6f61)',
} as const;

export type ThemeMode = 'dark' | 'light';

export const applyTheme = (mode: ThemeMode): void => {
  if (typeof document === 'undefined') {
    return;
  }
  const value = mode === 'light' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', value);
};
