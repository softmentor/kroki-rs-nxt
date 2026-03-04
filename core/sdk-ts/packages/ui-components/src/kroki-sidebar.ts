import { LitElement, css, html } from 'lit';
import { customElement, property } from 'lit/decorators.js';
import { panelStyles } from './base-styles.js';

export interface PlaygroundExample {
  id: string;
  label: string;
  diagramType: string;
  source: string;
}

@customElement('kroki-sidebar')
export class KrokiSidebar extends LitElement {
  @property({ attribute: false }) examples: PlaygroundExample[] = [];
  @property({ type: String }) selectedId = '';

  static override styles = [
    panelStyles,
    css`
      .wrap {
        height: 100%;
        overflow: auto;
        padding: 12px;
      }

      .item {
        border: 1px solid var(--glass-border);
        border-radius: 10px;
        cursor: pointer;
        margin-bottom: 8px;
        padding: 10px;
      }

      .item[data-selected='true'] {
        border-color: var(--accent-secondary);
        box-shadow: 0 0 0 1px var(--accent-secondary);
      }

      .name {
        font-size: 13px;
        font-weight: 600;
      }

      .meta {
        color: var(--text-dim);
        font-size: 12px;
      }
    `,
  ];

  override render() {
    return html`
      <div class="panel wrap">
        ${this.examples.map(
          (example) => html`
            <div
              class="item"
              data-selected=${String(example.id === this.selectedId)}
              @click=${() => this.select(example.id)}
            >
              <div class="name">${example.label}</div>
              <div class="meta">${example.diagramType}</div>
            </div>
          `,
        )}
      </div>
    `;
  }

  private select(id: string): void {
    this.dispatchEvent(
      new CustomEvent('example-selected', {
        detail: { id },
        bubbles: true,
        composed: true,
      }),
    );
  }
}
