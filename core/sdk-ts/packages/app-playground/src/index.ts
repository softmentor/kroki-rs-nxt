import { LitElement, css, html } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { applyTheme } from '@kroki/ui-tokens';
import type { HostAdapter } from '@kroki/host-adapters';
import type { DiagramType, OutputFormat, RenderResult } from '@kroki/runtime-wasm';
import '@kroki/ui-components';
import '@kroki/ui-tokens/theme.css';
import type { PlaygroundExample } from '@kroki/ui-components';
import { isLatestRevision, mapRenderResult, nextRevision } from './render-flow.js';

export interface PlaygroundState {
  source: string;
  diagramType: DiagramType;
  outputFormat: OutputFormat;
}

@customElement('kroki-playground')
export class KrokiPlayground extends LitElement {
  @property({ attribute: false }) adapter?: HostAdapter;

  @state() private state: PlaygroundState = {
    source: 'a -> b',
    diagramType: 'd2',
    outputFormat: 'svg',
  };

  @state() private examples: PlaygroundExample[] = [
    { id: 'd2-simple', label: 'D2: Simple Flow', diagramType: 'd2', source: 'a -> b' },
    { id: 'graphviz-basic', label: 'Graphviz: Directed', diagramType: 'graphviz', source: 'digraph G { A -> B; B -> C; }' },
    { id: 'mermaid-seq', label: 'Mermaid: Sequence', diagramType: 'mermaid', source: 'sequenceDiagram\n  Alice->>Bob: Hello Bob' },
  ];

  @state() private selectedExampleId = 'd2-simple';
  @state() private status = 'idle';
  @state() private durationMs = 0;
  @state() private renderedSvg = '';
  @state() private revisionId = 0;
  private activeAbort?: AbortController;

  static override styles = css`
    :host {
      background:
        radial-gradient(circle at 12% 20%, rgba(95, 81, 180, 0.24), transparent 40%),
        radial-gradient(circle at 85% 20%, rgba(255, 111, 97, 0.2), transparent 38%),
        var(--bg-deep);
      color: var(--text-main);
      display: block;
      min-height: 100vh;
      padding: 14px;
    }

    .layout {
      display: grid;
      gap: 12px;
      grid-template-columns: 240px 1fr 1fr;
      grid-template-rows: auto 1fr auto;
      min-height: calc(100vh - 28px);
    }

    kroki-topbar {
      grid-column: 1 / 4;
    }

    kroki-sidebar {
      grid-column: 1;
      grid-row: 2;
    }

    kroki-editor-pane {
      grid-column: 2;
      grid-row: 2;
    }

    kroki-preview-pane {
      grid-column: 3;
      grid-row: 2;
    }

    kroki-status-bar {
      grid-column: 1 / 4;
      grid-row: 3;
    }

    @media (max-width: 1140px) {
      .layout {
        grid-template-columns: 1fr;
        grid-template-rows: auto auto auto auto auto;
      }

      kroki-topbar,
      kroki-sidebar,
      kroki-editor-pane,
      kroki-preview-pane,
      kroki-status-bar {
        grid-column: 1;
      }
    }
  `;

  override connectedCallback(): void {
    super.connectedCallback();
    applyTheme('dark');
    void this.renderDiagram();
  }

  override render() {
    return html`
      <div class="layout">
        <kroki-topbar @theme-toggle=${this.onThemeToggle}></kroki-topbar>
        <kroki-sidebar
          .examples=${this.examples}
          .selectedId=${this.selectedExampleId}
          @example-selected=${this.onExampleSelected}
        ></kroki-sidebar>
        <kroki-editor-pane .value=${this.state.source} @source-changed=${this.onSourceChanged}></kroki-editor-pane>
        <kroki-preview-pane .svg=${this.renderedSvg}></kroki-preview-pane>
        <kroki-status-bar .status=${this.status} .durationMs=${this.durationMs}></kroki-status-bar>
      </div>
    `;
  }

  private onThemeToggle(event: CustomEvent<{ mode: 'dark' | 'light' }>): void {
    applyTheme(event.detail.mode);
  }

  private onExampleSelected(event: CustomEvent<{ id: string }>): void {
    const example = this.examples.find((candidate) => candidate.id === event.detail.id);
    if (!example) {
      return;
    }

    this.selectedExampleId = example.id;
    this.state = {
      ...this.state,
      source: example.source,
      diagramType: example.diagramType as DiagramType,
    };
    void this.renderDiagram();
  }

  private onSourceChanged(event: CustomEvent<{ value: string }>): void {
    this.state = {
      ...this.state,
      source: event.detail.value,
    };
    void this.renderDiagram();
  }

  private async renderDiagram(): Promise<void> {
    if (!this.adapter) {
      this.status = 'missing-adapter';
      return;
    }

    this.activeAbort?.abort();
    this.activeAbort = new AbortController();

    const revisionId = nextRevision(this.revisionId);
    this.revisionId = revisionId;
    const startedAt = performance.now();
    this.status = 'running';

    const result = await this.adapter.render(
      {
        ...this.state,
        revisionId,
      },
      this.activeAbort.signal,
    );

    if (!isLatestRevision(revisionId, this.revisionId)) {
      return;
    }

    this.durationMs = Math.round(performance.now() - startedAt);
    this.applyResult(result);
  }

  private applyResult(result: RenderResult): void {
    const mapped = mapRenderResult(result);
    this.status = mapped.status;
    if (mapped.renderedSvg !== undefined) {
      this.renderedSvg = mapped.renderedSvg;
    }
  }
}

export type { HostAdapter } from '@kroki/host-adapters';
