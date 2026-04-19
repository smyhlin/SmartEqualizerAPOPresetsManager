<script lang="ts">
  import { onMount } from 'svelte';
  import {
    AudioLines,
    Download,
    FolderInput,
    Search,
  } from '@lucide/svelte';
  import { open, save, ask } from '@tauri-apps/plugin-dialog';

  import Button from '$lib/components/ui/button.svelte';
  import Input from '$lib/components/ui/input.svelte';
  import ConfigEditorModal from '$lib/components/ConfigEditorModal.svelte';
  import GroupSidebar from '$lib/components/GroupSidebar.svelte';
  import PresetWorkspace from '$lib/components/PresetWorkspace.svelte';
  import EditorPane from '$lib/components/EditorPane.svelte';
  import { presetStore } from '$lib/store';
  import { getAutorunEnabled, onRuntimeSettingsUpdated, setAutorunEnabled } from '$lib/tauri';
  import type { AppRuntimeSettings, PresetGroup, PresetItem, PresetLibrary } from '$lib/types';
  import { uniqueName } from '$lib/utils';

  let library: PresetLibrary | null = null;
  let selectedGroupName: string | null = null;
  let selectedPresetName: string | null = null;
  let draft = '';
  let dirty = false;
  let busy = false;
  let search = '';
  let configEditorOpen = false;
  let statusMessage = 'Loading presets...';
  let statusTone: 'info' | 'success' | 'error' = 'info';
  let autorunEnabled = false;
  let autorunLoaded = false;
  let autorunBusy = false;

  onMount(() => {
    const unsubscribe = presetStore.subscribe((value) => {
      const preserveDraft = dirty && selectionStillExists(value);
      syncSelection(value, preserveDraft);
    });
    let disposed = false;
    let unlistenRuntimeSettings: (() => void) | null = null;

    void onRuntimeSettingsUpdated((value) => {
      if (!disposed) {
        syncRuntimeSettings(value);
      }
    })
      .then((unlisten) => {
        if (disposed) {
          unlisten();
          return;
        }
        unlistenRuntimeSettings = unlisten;
      })
      .catch((error) => setStatus(getErrorMessage(error), 'error'));

    void loadAutorunState();

    void presetStore
      .start()
      .then(() => setStatus('Ready to manage Equalizer APO presets.'))
      .catch((error) => setStatus(getErrorMessage(error), 'error'));

    return () => {
      disposed = true;
      unsubscribe();
      unlistenRuntimeSettings?.();
      void presetStore.stop();
    };
  });

  function selectionStillExists(next: PresetLibrary | null) {
    if (!next || !selectedGroupName || !selectedPresetName) {
      return false;
    }

    return next.groups.some(
      (group) =>
        group.name === selectedGroupName &&
        group.presets.some((preset) => preset.name === selectedPresetName)
    );
  }

  function syncSelection(next: PresetLibrary | null, preserveDraft = false) {
    library = next;
    if (!next || next.groups.length === 0) {
      selectedGroupName = null;
      selectedPresetName = null;
      if (!preserveDraft) {
        draft = '';
        dirty = false;
      }
      return;
    }

    if (!selectedGroupName || !next.groups.some((group) => group.name === selectedGroupName)) {
      selectedGroupName = next.groups[0]?.name ?? null;
    }

    const group = currentGroup(next);
    if (!group) {
      selectedPresetName = null;
      if (!preserveDraft) {
        draft = '';
        dirty = false;
      }
      return;
    }

    if (!selectedPresetName || !group.presets.some((preset) => preset.name === selectedPresetName)) {
      selectedPresetName = group.presets[0]?.name ?? null;
    }

    if (!preserveDraft) {
      draft = currentPreset(next)?.content ?? '';
      dirty = false;
    }
  }

  function currentGroup(snapshot: PresetLibrary | null = library): PresetGroup | null {
    return snapshot?.groups.find((group) => group.name === selectedGroupName) ?? null;
  }

  function currentPreset(snapshot: PresetLibrary | null = library): PresetItem | null {
    return (
      currentGroup(snapshot)?.presets.find((preset) => preset.name === selectedPresetName) ?? null
    );
  }

  function presetForGroup(snapshot: PresetLibrary | null, groupName: string, presetName: string) {
    return (
      snapshot?.groups.find((group) => group.name === groupName)?.presets.find((preset) => preset.name === presetName) ??
      null
    );
  }

  function isSelectedPreset(groupName: string, presetName: string) {
    return selectedGroupName === groupName && selectedPresetName === presetName;
  }

  async function confirmDiscardIfNeeded() {
    if (!dirty) {
      return true;
    }

    return ask('Discard the current unsaved preset edits?', {
      title: 'Unsaved changes',
      kind: 'warning'
    });
  }

  function setStatus(message: string, tone: 'info' | 'success' | 'error' = 'info') {
    statusMessage = message;
    statusTone = tone;
  }

  function syncRuntimeSettings(value: AppRuntimeSettings) {
    autorunEnabled = value.autorunEnabled;
    autorunLoaded = true;
    autorunBusy = false;
  }

  function getErrorMessage(error: unknown) {
    if (typeof error === 'string') {
      return error;
    }
    if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string') {
      return error.message;
    }
    return 'An unexpected error occurred.';
  }

  async function withBusy<T>(task: () => Promise<T>, successMessage?: string) {
    busy = true;
    try {
      const result = await task();
      if (successMessage) {
        setStatus(successMessage, 'success');
      }
      return result;
    } catch (error) {
      setStatus(getErrorMessage(error), 'error');
      return null;
    } finally {
      busy = false;
    }
  }

  async function loadAutorunState() {
    try {
      syncRuntimeSettings({
        autorunEnabled: await getAutorunEnabled()
      });
    } catch (error) {
      autorunLoaded = true;
      setStatus(getErrorMessage(error), 'error');
    }
  }

  async function handleGroupSelect(groupName: string) {
    if (!(await confirmDiscardIfNeeded())) {
      return;
    }

    selectedGroupName = groupName;
    selectedPresetName = currentGroup()?.presets[0]?.name ?? null;
    draft = currentPreset()?.content ?? '';
    dirty = false;
  }

  async function handlePresetSelect(presetName: string) {
    if (!(await confirmDiscardIfNeeded())) {
      return;
    }

    selectedPresetName = presetName;
    draft = currentPreset()?.content ?? '';
    dirty = false;
  }

  async function handleCreateGroup(value: { name: string; emoji: string | null }) {
    const { name, emoji } = value;
    const snapshot = await withBusy(() => presetStore.createGroup(name), `Created group ${name}`);
    if (snapshot) {
      selectedGroupName = name;
      selectedPresetName = null;
      draft = '';
      dirty = false;

      if (emoji) {
        await withBusy(
          () => presetStore.setGroupEmoji(name, emoji),
          `Set emoji for ${name}`
        );
      }
    }
  }

  async function handleRenameGroup(value: { oldName: string; newName: string }) {
    const { oldName, newName } = value;
    const snapshot = await withBusy(
      () => presetStore.renameGroup(oldName, newName),
      `Renamed ${oldName} to ${newName}`
    );
    if (snapshot && selectedGroupName === oldName) {
      selectedGroupName = newName;
    }
  }

  async function handleDeleteGroup(groupName: string) {
    const confirmed = await ask(`Delete the group "${groupName}" and all presets inside it?`, {
      title: 'Delete group',
      kind: 'warning'
    });
    if (!confirmed) {
      return;
    }

    await withBusy(() => presetStore.deleteGroup(groupName), `Deleted group ${groupName}`);
  }

  async function handleSetGroupEmoji(value: { groupName: string; emoji: string | null }) {
    await withBusy(
      () => presetStore.setGroupEmoji(value.groupName, value.emoji),
      value.emoji ? `Updated emoji for ${value.groupName}` : `Cleared emoji for ${value.groupName}`
    );
  }


  async function handleCreatePreset(presetName: string) {
    if (!selectedGroupName) {
      return;
    }

    const snapshot = await withBusy(
      () => presetStore.createPreset(selectedGroupName as string, presetName, ''),
      `Created preset ${presetName}`
    );
    if (snapshot) {
      selectedPresetName = presetName;
      draft = '';
      dirty = false;
    }
  }

  async function handleRenamePreset(value: { oldName: string; newName: string }) {
    const { oldName, newName } = value;
    if (!selectedGroupName) {
      return;
    }

    const snapshot = await withBusy(
      () => presetStore.renamePreset(selectedGroupName as string, oldName, newName),
      `Renamed ${oldName} to ${newName}`
    );
    if (snapshot && selectedPresetName === oldName) {
      selectedPresetName = newName;
    }
  }

  async function handleDeletePreset(presetName: string) {
    if (!selectedGroupName) {
      return;
    }

    const confirmed = await ask(`Delete the preset "${presetName}"?`, {
      title: 'Delete preset',
      kind: 'warning'
    });
    if (!confirmed) {
      return;
    }

    await withBusy(
      () => presetStore.deletePreset(selectedGroupName as string, presetName),
      `Deleted preset ${presetName}`
    );
  }

  async function handleMovePreset(event: { oldGroup: string; newGroup: string; name: string; targetIndex?: number }) {
    const { oldGroup, newGroup, name, targetIndex } = event;
    const snapshot = await withBusy(
      () => presetStore.movePreset(oldGroup, newGroup, name, targetIndex),
      oldGroup === newGroup ? `Reordered ${name}` : `Moved ${name} to ${newGroup}`
    );
    if (snapshot && selectedPresetName === name) {
      selectedGroupName = newGroup;
    }
  }

  async function handleSave() {
    if (!selectedGroupName || !selectedPresetName) {
      return;
    }

    const snapshot = await withBusy(
      () => presetStore.savePreset(selectedGroupName as string, selectedPresetName as string, draft),
      `Saved ${selectedPresetName}`
    );
    if (snapshot) {
      dirty = false;
    }
  }

  async function handleApply() {
    if (!selectedGroupName || !selectedPresetName) {
      return;
    }

    if (dirty) {
      const saved = await withBusy(
        () => presetStore.savePreset(selectedGroupName as string, selectedPresetName as string, draft),
        `Saved ${selectedPresetName}`
      );
      if (!saved) {
        return;
      }
      dirty = false;
    }

    await withBusy(
      () => presetStore.applyPreset(selectedGroupName as string, selectedPresetName as string),
      `Applied ${selectedPresetName}`
    );
  }

  async function handleApplyPreset(name: string) {
    if (selectedPresetName !== name) {
      selectedPresetName = name;
      draft = currentPreset()?.content ?? '';
      dirty = false;
    }

    await handleApply();
  }

  async function handleImportPresets() {
    const selection = await open({
      multiple: true,
      filters: [{ name: 'Equalizer APO presets or WAV files', extensions: ['txt', 'wav'] }]
    });

    const paths = Array.isArray(selection) ? selection : selection ? [selection] : [];
    if (paths.length === 0) {
      return;
    }

    let targetGroupName = selectedGroupName;
    if (!targetGroupName) {
      const nextGroupName = uniqueName(
        'Imported',
        library?.groups.map((group) => group.name) ?? []
      );
      const snapshot = await withBusy(() => presetStore.createGroup(nextGroupName), `Created group ${nextGroupName}`);
      if (!snapshot) {
        return;
      }
      targetGroupName = nextGroupName;
      selectedGroupName = nextGroupName;
    }

    await withBusy(
      () => presetStore.importPresets(targetGroupName as string, paths),
      `Imported ${paths.length} preset${paths.length === 1 ? '' : 's'}`
    );
  }

  async function handleToggleConvolution(value: { groupName: string; presetName: string; enabled: boolean }) {
    const preset = presetForGroup(library, value.groupName, value.presetName);
    const baseContent =
      isSelectedPreset(value.groupName, value.presetName) && dirty
        ? draft
        : preset?.content ?? '';

    if (value.enabled) {
      const selection = await open({
        multiple: false,
        filters: [{ name: 'Convolution WAV', extensions: ['wav'] }]
      });

      if (typeof selection !== 'string') {
        return false;
      }

      const snapshot = await withBusy(
        () =>
          presetStore.attachConvolutionWav(
            value.groupName,
            value.presetName,
            baseContent,
            selection
          ),
        `Linked convolution WAV for ${value.presetName}`
      );

      if (!snapshot) {
        return false;
      }

      if (isSelectedPreset(value.groupName, value.presetName)) {
        draft = presetForGroup(snapshot, value.groupName, value.presetName)?.content ?? draft;
        dirty = false;
      }

      return true;
    }

    const snapshot = await withBusy(
      () => presetStore.removeConvolutionWav(value.groupName, value.presetName, baseContent),
      `Removed convolution WAV from ${value.presetName}`
    );

    if (!snapshot) {
      return false;
    }

    if (isSelectedPreset(value.groupName, value.presetName)) {
      draft = presetForGroup(snapshot, value.groupName, value.presetName)?.content ?? draft;
      dirty = false;
    }

    return true;
  }

  async function handleImportAppData() {
    const selection = await open({
      multiple: false,
      filters: [{ name: 'SmartEqualizerAPO Backup', extensions: ['json'] }]
    });

    if (typeof selection !== 'string') {
      return;
    }

    const confirmed = await ask(
      'Import app data from this backup file? This will replace the current groups, presets, and settings.',
      {
        title: 'Import App Data',
        kind: 'warning'
      }
    );

    if (!confirmed) {
      return;
    }

    await withBusy(() => presetStore.importAppSettings(selection), 'Imported app settings');
  }

  async function handleExport() {
    if (!selectedGroupName || !selectedPresetName) {
      return;
    }

    const destination = await save({
      defaultPath: `${selectedPresetName}.txt`,
      filters: [{ name: 'Equalizer APO Presets', extensions: ['txt'] }]
    });

    if (!destination) {
      return;
    }

    await withBusy(
      () => presetStore.exportPreset(selectedGroupName as string, selectedPresetName as string, destination),
      `Exported ${selectedPresetName}`
    );
  }

  async function handleExportAppSettings() {
    const destination = await save({
      defaultPath: 'smart-equalizer-apo-backup.json',
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });

    if (!destination) {
      return;
    }

    await withBusy(() => presetStore.exportAppSettings(destination), 'Exported app settings');
  }

  async function handleAutorunToggle(event: Event) {
    const nextEnabled = (event.currentTarget as HTMLInputElement).checked;
    autorunBusy = true;

    try {
      const actualEnabled = await setAutorunEnabled(nextEnabled);
      syncRuntimeSettings({
        autorunEnabled: actualEnabled
      });
      setStatus(
        actualEnabled
          ? 'Launch on Windows startup enabled.'
          : 'Launch on Windows startup disabled.',
        'success'
      );
    } catch (error) {
      autorunBusy = false;
      setStatus(getErrorMessage(error), 'error');
    }
  }


  function handleOpenConfigEditor() {
    if (selectedGroupName && selectedPresetName) {
      configEditorOpen = true;
    }
  }

  function handleCloseConfigEditor() {
    configEditorOpen = false;
  }


</script>

<svelte:head>
  <title>SmartEqualizerAPOPresetsManager</title>
  <meta
    name="description"
    content="Tray-first Equalizer APO preset management for Windows 11."
  />
</svelte:head>

<div class="min-h-screen bg-background text-foreground">
  <div class="mx-auto flex min-h-screen max-w-[1920px] flex-col px-4 py-4 sm:px-5">
    <header class="shell-surface mb-4 overflow-hidden p-4 shadow-[0_12px_30px_rgba(0,0,0,0.25)]">
      <div class="grid gap-4 xl:grid-cols-[1.2fr_1fr_auto] xl:items-center">
        <div class="flex items-center gap-4">
          <div class="flex size-11 items-center justify-center rounded-[12px] border border-accent/30 bg-accent-soft text-accent">
            <AudioLines size={22} />
          </div>
          <div class="min-w-0">
            <p class="text-[11px] font-semibold uppercase tracking-[0.22em] text-muted">
              SmartEqualizer APO
            </p>
            <h1 class="mt-1 text-[20px] font-semibold tracking-tight text-foreground sm:text-[22px]">
              Equalizer APO preset manager
            </h1>
            <p class="mt-1 max-w-3xl text-sm leading-6 text-muted">
              One active preset per group. Apply changes instantly, keep the tray checkmarks in sync, and work from a writable app folder.
            </p>
          </div>
        </div>

        <div class="shell-surface-2 flex items-center gap-3 px-3 py-2">
          <Search size={17} class="shrink-0 text-muted" />
          <Input
            bind:value={search}
            placeholder="Search groups, presets, or config text"
            class="border-0 bg-transparent px-0 shadow-none focus-visible:ring-0"
          />
        </div>

        <div class="flex flex-wrap justify-start gap-2 xl:justify-end">
          <Button variant="secondary" onclick={handleImportAppData}>
            <FolderInput size={14} />
            Import App Data
          </Button>
          <Button variant="secondary" onclick={handleExportAppSettings} disabled={!library}>
            <Download size={14} />
            Export App Data
          </Button>

        </div>
      </div>

    </header>

    <main class="grid min-h-0 flex-1 gap-4 overflow-y-auto xl:grid-cols-[300px_minmax(340px,1fr)_460px]">
      <GroupSidebar
        groups={library?.groups ?? []}
        {selectedGroupName}
        {search}
        onSelect={handleGroupSelect}
        onCreate={handleCreateGroup}
        onRename={handleRenameGroup}
        onDelete={handleDeleteGroup}
        onMovePreset={handleMovePreset}
        onEmojiChange={handleSetGroupEmoji}
      />

      <PresetWorkspace
        group={currentGroup()}
        {selectedPresetName}
        {search}
        presetFilePath={
          library && selectedGroupName && selectedPresetName
            ? `${library.appDataDir}\\presets\\${selectedGroupName}\\${selectedPresetName}.txt`
            : null
        }
        onSelect={handlePresetSelect}
        onCreate={handleCreatePreset}
        onRename={handleRenamePreset}
        onDelete={handleDeletePreset}
        onApply={handleApplyPreset}
        onMove={handleMovePreset}
        onImport={handleImportPresets}
      />

      <EditorPane
        groupName={selectedGroupName}
        presetName={selectedPresetName}
        configPath={library?.configPath ?? null}
        panelKey={selectedGroupName && selectedPresetName ? `${selectedGroupName}::${selectedPresetName}` : ''}
        presetConvolution={currentPreset()?.convolution ?? null}
        {draft}
        {dirty}
        onSave={handleSave}
        onApply={handleApply}
        onExport={handleExport}
        onEditConfig={handleOpenConfigEditor}
        onToggleConvolution={handleToggleConvolution}
      />
    </main>

    <ConfigEditorModal
      open={configEditorOpen}
      groupName={selectedGroupName}
      presetName={selectedPresetName}
      {draft}
      {dirty}
      presetFilePath={
        library && selectedGroupName && selectedPresetName
          ? `${library.appDataDir}\\presets\\${selectedGroupName}\\${selectedPresetName}.txt`
          : null
      }
      configPath={library?.configPath ?? null}
      configTargetLabel="Equalizer APO config"
      panelKey={selectedGroupName && selectedPresetName ? `${selectedGroupName}::${selectedPresetName}` : ''}
      presetConvolution={currentPreset()?.convolution ?? null}
      onDraftChange={(value) => { draft = value; dirty = true; }}
      onSave={handleSave}
      onClose={handleCloseConfigEditor}
      onToggleConvolution={handleToggleConvolution}
    />

    <footer class="shell-surface mt-4 flex flex-col gap-3 px-4 py-3 text-sm md:flex-row md:items-center md:justify-between">
      <div class="flex min-w-0 items-center gap-3">
        <span
          class={`inline-flex shrink-0 items-center rounded-full px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.16em] ${
            statusTone === 'success'
              ? 'bg-success-soft text-success'
              : statusTone === 'error'
                ? 'bg-danger-soft text-danger'
                : 'bg-accent-soft text-accent'
          }`}
        >
          {busy ? 'Working' : statusTone}
        </span>
        <span class="min-w-0 truncate text-muted">{statusMessage}</span>
      </div>

      <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-[11px] text-muted">
        <span>
          App data folder:
          <span class="ml-1 font-mono text-foreground">{library?.appDataDir ?? '%APPDATA%\\SmartEqualizerAPO'}</span>
        </span>
        <span class="h-3 w-px bg-border/60 hidden md:inline-block"></span>
        <span>
          APO EQ config:
          <span class="ml-1 font-mono text-foreground">{library?.configPath ?? '%ProgramFiles%\\EqualizerAPO\\config'}</span>
        </span>
        <span class="h-3 w-px bg-border/60 hidden md:inline-block"></span>
        <label
          class={`inline-flex items-center gap-2 ${autorunLoaded ? 'text-muted' : 'text-muted/70'}`}
          title="Launch the app automatically when you sign in to Windows"
        >
          <input
            type="checkbox"
            checked={autorunEnabled}
            disabled={!autorunLoaded || autorunBusy || busy}
            onchange={handleAutorunToggle}
            class="focus-ring size-3.5 rounded border border-border bg-surface-2 accent-accent disabled:cursor-not-allowed disabled:opacity-60"
          />
          <span>Launch on Windows startup</span>
        </label>
      </div>
    </footer>
  </div>
</div>
