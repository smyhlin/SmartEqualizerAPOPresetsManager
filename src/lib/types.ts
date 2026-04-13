export type PresetItem = {
  name: string;
  order: number;
  content: string;
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
