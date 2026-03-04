import { LitElement, css, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { unsafeSVG } from 'lit/directives/unsafe-svg.js';
import { panelStyles } from './base-styles.js';

@customElement('kroki-preview-pane')
export class KrokiPreviewPane extends LitElement {
  @property({ type: String }) svg = '';

  static override styles = [
    panelStyles,
    css`
      .wrap {
        height: 100%;
        padding: 12px;
      }

      .canvas {
        align-items: center;
        background: rgba(9, 8, 24, 0.48);
        border: 1px solid var(--glass-border);
        border-radius: 12px;
        display: flex;
        height: 100%;
        justify-content: center;
        min-height: 360px;
        overflow: auto;
        padding: 16px;
      }
    `,
  ];

  override render() {
    return html`<div class="panel wrap"><div class="canvas">${this.svg ? unsafeSVG(this.svg) : 'No render yet'}</div></div>`;
  }
}
