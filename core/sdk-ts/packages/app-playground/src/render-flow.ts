import type { RenderResult } from '@kroki/runtime-wasm';

export interface ApplyRenderResult {
  status: string;
  renderedSvg?: string;
}

export const nextRevision = (currentRevision: number): number => currentRevision + 1;

export const isLatestRevision = (resolvedRevision: number, currentRevision: number): boolean => resolvedRevision === currentRevision;

export const mapRenderResult = (result: RenderResult): ApplyRenderResult => {
  if (result.kind === 'error') {
    return {
      status: `error:${result.code}`,
    };
  }

  return {
    status: 'success',
    renderedSvg: result.data,
  };
};
