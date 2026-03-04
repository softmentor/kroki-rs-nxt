import { LitElement, css, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { panelStyles } from './base-styles.js';

@customElement('kroki-status-bar')
export class KrokiStatusBar extends LitElement {
  @property({ type: String }) status = 'idle';
  @property({ type: Number }) durationMs = 0;

  static override styles = [
    panelStyles,
    css`
      .bar {
        align-items: center;
        display: flex;
        justify-content: space-between;
        padding: 8px 12px;
      }

      .meta {
        color: var(--text-dim);
        font-size: 12px;
      }
    `,
  ];

  override render() {
    return html`
      <div class="panel bar">
        <span>Status: ${this.status}</span>
        <span class="meta">${this.durationMs > 0 ? `${this.durationMs} ms` : '-'}</span>
      </div>
    `;
  }
}
