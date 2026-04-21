export type PresetConvolution = {
  wavPath: string;
  wavBase64?: string | null;
  error?: string | null;
};

export type PresetItem = {
  name: string;
  order: number;
  content: string;
  convolution?: PresetConvolution | null;
};

export type PresetGroup = {
  name: string;
  order: number;
  emoji: string | null;
  activePreset: string | null;
  presets: PresetItem[];
};

export type PresetLibrary = {
  appDataDir: string;
  configPath: string;
  defaultConfigPath: string;
  installedConfigPath: string | null;
  groups: PresetGroup[];
  needsConfigMigration: boolean;
  configPathPrompted: boolean;
};

export type AppRuntimeSettings = {
  autorunEnabled: boolean;
};

export type LogSnapshot = {
  logPath: string;
  content: string;
  exists: boolean;
};
