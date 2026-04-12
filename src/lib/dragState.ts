/**
 * Global drag state — shared between PresetWorkspace and GroupSidebar.
 *
 * We cannot rely on reading custom MIME-type data during `dragover` in
 * Tauri's WebView2 (the browser only exposes type keys, not values, for
 * security – and custom types may be silently dropped by WebView2 entirely).
 * Instead we track the active drag payload in a module-level variable that
 * every component can read synchronously.
 */

export type PresetDragPayload = {
  kind: 'preset';
  oldGroup: string;
  name: string;
};

export type GroupDragPayload = {
  kind: 'group';
  name: string;
};

export type DragPayload = PresetDragPayload | GroupDragPayload;

let _current: DragPayload | null = null;

export const dragState = {
  set(payload: DragPayload) {
    _current = payload;
  },
  clear() {
    _current = null;
  },
  get(): DragPayload | null {
    return _current;
  },
  isPreset(): boolean {
    return _current?.kind === 'preset';
  },
};
