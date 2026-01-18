export default {
  // App
  app: {
    title: 'Open Switch - Coding Agent Configuration Manager'
  },

  // Common
  common: {
    loading: 'Loading...',
    cancel: 'Cancel',
    save: 'Save',
    saving: 'Saving...',
    confirm: 'Confirm',
    delete: 'Delete',
    edit: 'Edit',
    add: 'Add',
    apply: 'Apply',
    applying: 'Applying...',
    selectAll: 'Select All',
    clearAll: 'Clear All',
    tip: 'Tip',
    optional: 'Optional'
  },

  // Navigation
  nav: {
    providers: 'Providers',
    mcp: 'MCP',
    backup: 'Backup',
    status: 'Status'
  },

  // System status
  system: {
    operational: 'System Operational',
    darkMode: 'Dark Mode',
    lightMode: 'Light Mode'
  },

  // Language
  language: {
    switch: 'Switch Language',
    zh: '中文',
    en: 'English'
  },

  // Provider related
  provider: {
    title: 'Providers',
    new: 'New',
    apply: 'Apply',
    edit: 'Edit',
    delete: 'Delete',
    noProviders: 'No providers found',
    models: 'models',
    addProvider: 'Add Provider',
    editProvider: 'Edit Provider',
    name: 'Name',
    nameRequired: 'Please enter a name',
    apiKey: 'API Key',
    apiKeyRequired: 'Please enter an API Key',
    baseUrl: 'Base URL',
    description: 'Description',
    preset: 'Select Preset',
    protocol: 'API Protocol',
    protocolAnthropic: 'Anthropic Protocol',
    protocolOpenAI: 'OpenAI Protocol',
    autoAddModels: 'Auto add preset models',
    getApiKey: 'Get API Key',
    modelsSelected: '{selected} / {total} models selected',
    selectAll: 'Select All',
    unselectAll: 'Unselect All',
    placeholder: {
      name: 'my-provider',
      apiKey: 'sk-...',
      baseUrl: 'https://api.anthropic.com',
      description: 'Optional'
    },
    showApiKey: 'Show',
    hideApiKey: 'Hide'
  },

  // Model related
  model: {
    title: 'Models',
    addModel: 'Add Model',
    sync: 'Sync',
    selectProviderFirst: 'Select a provider first',
    noModels: 'No models found',
    delete: 'Delete',
    modelId: 'Model ID',
    modelIdRequired: 'Please enter a Model ID',
    displayName: 'Display Name',
    providerNotSelected: 'No provider selected',
    placeholder: {
      modelId: 'gpt-4o',
      displayName: 'Optional, uses ID by default'
    },
    adding: 'Adding...'
  },

  // Detail panel
  detail: {
    title: 'Configuration Details',
    selectProvider: 'Select a provider to view details',
    providerSpec: 'Provider Specification',
    selectedModel: 'Selected Model',
    name: 'NAME',
    endpoint: 'ENDPOINT',
    models: 'MODELS',
    desc: 'DESC',
    id: 'ID',
    context: 'CONTEXT',
    output: 'OUTPUT',
    available: 'available',
    tokens: 'tokens'
  },

  // Confirm dialog
  confirm: {
    title: 'Confirm',
    defaultMessage: 'Are you sure you want to perform this action?',
    deleteTitle: 'Confirm Delete',
    deleteProvider: "Are you sure you want to delete Provider '{name}'?",
    deleteModel: "Are you sure you want to delete Model '{name}'?"
  },

  // Apply config
  applyConfig: {
    title: 'Apply Configuration',
    applyTo: 'Apply Provider',
    configTo: "'s configuration to:",
    currentProject: 'Current Project',
    globalConfig: 'Global Configuration',
    projectPath: './.opencode/opencode.json',
    globalPath: '~/.opencode/opencode.json',
    selectTarget: 'Please select at least one target'
  },

  // Fetch models
  fetchModels: {
    title: 'Fetch Site Models',
    fetching: 'Fetching model list...',
    noModels: 'No models found',
    totalModels: '{total} models total, {selected} selected',
    addModels: 'Add {count} models'
  },

  // MCP page
  mcp: {
    title: 'MCP Servers',
    loading: 'Loading...',
    noServers: 'No MCP servers',
    enabled: 'Enable',
    disabled: 'Disable',
    selectServer: 'Select an MCP server to view details',
    type: 'Type',
    local: 'Local',
    remote: 'Remote',
    status: 'Status',
    statusEnabled: 'Enabled',
    statusDisabled: 'Disabled',
    command: 'Command',
    url: 'URL',
    // New
    addRecommended: 'Add Recommended MCP',
    addCustom: 'Custom Add',
    installing: 'Adding...',
    recommended: 'Recommended MCP Servers',
    addSelected: 'Add Selected ({count})',
    addAll: 'Add All',
    serverAdded: 'Added {count} server(s)',
    serverSkipped: '{count} server(s) already exist, skipped',
    visitSite: 'Visit Website',
    // Custom MCP
    customTitle: 'Custom Add MCP',
    customName: 'Server Name',
    customNamePlaceholder: 'e.g. my-mcp-server',
    customConfig: 'JSON Configuration',
    customConfigHint: 'Supports OpenCode format: command, args, env, type',
    customNameRequired: 'Please enter server name',
    customJsonInvalid: 'Invalid JSON format',
    customCommandRequired: 'Please configure command or args',
    customAdded: 'Added {name}',
    // Sync
    syncToOpenCode: 'Sync to OpenCode',
    syncing: 'Syncing...',
    syncSuccess: 'Synced to ~/.opencode/opencode.json',
    syncFailed: 'Sync failed'
  },

  // Backup page
  backup: {
    title: 'Backup & Restore',
    featureTitle: 'Configuration Backup',
    featureDesc: 'Backup feature supports backing up your Coding Agent configuration to a WebDAV server.',
    supportedTypes: 'Supported Backup Types',
    openCodeConfig: 'OpenCode Configuration',
    mcpConfig: 'MCP Server Configuration',
    usage: 'Usage',
    usageDesc: 'Since backup involves network operations, please use command line mode for backup/restore:',
    createBackup: 'Create backup',
    restoreBackup: 'Restore backup',
    tipTitle: 'Tip',
    tipContent: 'Backup feature requires a WebDAV server. You can use services that support WebDAV like Nutstore, NextCloud, etc.'
  },

  // Status page
  status: {
    title: 'System Status',
    appInfo: 'Application Info',
    currentVersion: 'Current Version',
    providerCount: 'Provider Count',
    configStatus: 'Configuration Status',
    globalConfig: 'Global Config',
    projectConfig: 'Project Config',
    configured: '✓ Configured',
    notConfigured: '✗ Not Configured',
    currentProvider: 'Current Provider',
    mcpServers: 'MCP Servers',
    count: '{count}',
    configPaths: 'Config Paths',
    openCode: 'OpenCode'
  }
}
