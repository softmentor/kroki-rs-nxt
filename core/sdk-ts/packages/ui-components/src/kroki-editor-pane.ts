import { LitElement, css, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { panelStyles } from './base-styles.js';

@customElement('kroki-editor-pane')
export class KrokiEditorPane extends LitElement {
  @property({ type: String }) value = '';

  static override styles = [
    panelStyles,
    css`
      .wrap {
        height: 100%;
        padding: 12px;
      }

      textarea {
        background: rgba(9, 8, 24, 0.62);
        border: 1px solid var(--glass-border);
        border-radius: 12px;
        color: var(--text-main);
        font: 14px/1.5 'JetBrains Mono', 'Fira Code', monospace;
        height: 100%;
        min-height: 360px;
        outline: none;
        padding: 14px;
        width: 100%;
      }
    `,
  ];

  override render() {
    return html`<div class="panel wrap"><textarea .value=${this.value} @input=${this.onInput}></textarea></div>`;
  }

  private onInput(event: Event): void {
    const target = event.target as HTMLTextAreaElement;
    this.dispatchEvent(
      new CustomEvent('source-changed', {
        detail: { value: target.value },
        bubbles: true,
        composed: true,
      }),
    );
  }
}
