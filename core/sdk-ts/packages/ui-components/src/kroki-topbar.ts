import { LitElement, css, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { panelStyles } from './base-styles.js';

@customElement('kroki-topbar')
export class KrokiTopbar extends LitElement {
  @property({ type: String }) mode: 'dark' | 'light' = 'dark';

  static override styles = [
    panelStyles,
    css`
      .bar {
        align-items: center;
        display: flex;
        gap: 12px;
        justify-content: space-between;
        padding: 10px 14px;
      }

      .title {
        font-size: 14px;
        font-weight: 600;
        letter-spacing: 0.08em;
        text-transform: uppercase;
      }

      button {
        background: var(--grad-main);
        border: 0;
        border-radius: 999px;
        color: #150d2f;
        cursor: pointer;
        font-weight: 600;
        padding: 8px 14px;
      }
    `,
  ];

  override render() {
    return html`
      <div class="panel bar">
        <span class="title">kroki-rs-nxt playground</span>
        <button @click=${this.toggleMode}>Theme: ${this.mode}</button>
      </div>
    `;
  }

  private toggleMode(): void {
    this.mode = this.mode === 'dark' ? 'light' : 'dark';
    this.dispatchEvent(
      new CustomEvent('theme-toggle', {
        detail: { mode: this.mode },
        bubbles: true,
        composed: true,
      }),
    );
  }
}
