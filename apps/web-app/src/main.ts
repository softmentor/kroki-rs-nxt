import '@kroki/app-playground';
import { RuntimeBridgeStub } from '@kroki/runtime-wasm';
import { RuntimeHostAdapter } from '@kroki/host-adapters';
import type { KrokiPlayground } from '@kroki/app-playground';

const root = document.querySelector<HTMLDivElement>('#app');
if (!root) {
  throw new Error('missing app root');
}

const element = document.createElement('kroki-playground') as KrokiPlayground;
element.adapter = new RuntimeHostAdapter(new RuntimeBridgeStub());
root.appendChild(element);
