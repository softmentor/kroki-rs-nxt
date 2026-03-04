export type DiagramType =
  | 'graphviz'
  | 'd2'
  | 'mermaid'
  | 'bpmn'
  | 'ditaa'
  | 'excalidraw'
  | 'wavedrom'
  | 'vega'
  | 'vegalite';

export type OutputFormat = 'svg' | 'png' | 'webp';

export interface RenderRequest {
  diagramType: DiagramType;
  outputFormat: OutputFormat;
  source: string;
  options?: Record<string, string | number | boolean>;
  revisionId: number;
}

export interface RenderDiagnostics {
  durationMs?: number;
  provider?: string;
}

export interface RenderSuccess {
  kind: 'success';
  contentType: string;
  data: string;
  diagnostics?: RenderDiagnostics;
}

export interface RenderFailure {
  kind: 'error';
  code: string;
  message: string;
  details?: string;
}

export type RenderResult = RenderSuccess | RenderFailure;

export interface RuntimeBridge {
  render(request: RenderRequest, signal?: AbortSignal): Promise<RenderResult>;
  getCapabilities(): Promise<DiagramType[]>;
}

export class RuntimeBridgeStub implements RuntimeBridge {
  async render(request: RenderRequest): Promise<RenderResult> {
    return {
      kind: 'success',
      contentType: request.outputFormat === 'svg' ? 'image/svg+xml' : `image/${request.outputFormat}`,
      data: `<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"640\" height=\"320\"><rect width=\"100%\" height=\"100%\" fill=\"#120f2d\"/><text x=\"24\" y=\"48\" fill=\"#ffb30f\" font-size=\"20\">Stub render for ${request.diagramType}</text><text x=\"24\" y=\"86\" fill=\"#f7f7fb\" font-size=\"14\">Revision ${request.revisionId}</text></svg>`,
      diagnostics: { durationMs: 1, provider: 'runtime-stub' },
    };
  }

  async getCapabilities(): Promise<DiagramType[]> {
    return ['graphviz', 'd2', 'mermaid', 'bpmn', 'ditaa', 'excalidraw', 'wavedrom', 'vega', 'vegalite'];
  }
}
