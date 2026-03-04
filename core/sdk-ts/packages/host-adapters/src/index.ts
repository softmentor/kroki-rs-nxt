import type { RenderRequest, RenderResult, RuntimeBridge } from '@kroki/runtime-wasm';

export interface HostAdapter {
  render(request: RenderRequest, signal?: AbortSignal): Promise<RenderResult>;
}

export class RuntimeHostAdapter implements HostAdapter {
  constructor(private readonly bridge: RuntimeBridge) {}

  render(request: RenderRequest, signal?: AbortSignal): Promise<RenderResult> {
    return this.bridge.render(request, signal);
  }
}

export class ServerHttpAdapter implements HostAdapter {
  constructor(private readonly endpoint = '/render') {}

  async render(request: RenderRequest, signal?: AbortSignal): Promise<RenderResult> {
    const response = await fetch(this.endpoint, {
      method: 'POST',
      signal,
      headers: {
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        diagram_type: request.diagramType,
        output_format: request.outputFormat,
        diagram_source: request.source,
      }),
    });

    const body = (await response.json()) as {
      data?: string;
      error?: { code?: string; message?: string };
    };

    if (!response.ok || body.error) {
      return {
        kind: 'error',
        code: body.error?.code ?? `http_${response.status}`,
        message: body.error?.message ?? 'Request failed',
      };
    }

    return {
      kind: 'success',
      contentType: request.outputFormat === 'svg' ? 'image/svg+xml' : `image/${request.outputFormat}`,
      data: body.data ?? '',
      diagnostics: {
        durationMs: 0,
        provider: 'server-http',
      },
    };
  }
}
