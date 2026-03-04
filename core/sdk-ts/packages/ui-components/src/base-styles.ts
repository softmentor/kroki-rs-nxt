import { css } from 'lit';

export const panelStyles = css`
  :host {
    display: block;
    color: var(--text-main);
    font-family: 'Inter', 'Segoe UI', sans-serif;
  }

  .panel {
    backdrop-filter: blur(10px) saturate(160%);
    background: var(--bg-card);
    border: 1px solid var(--glass-border);
    border-radius: 16px;
    box-shadow: var(--card-shadow);
  }
`;
