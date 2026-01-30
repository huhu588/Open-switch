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
    optional: 'Optional',
    refresh: 'Refresh'
  },

  // Navigation
  nav: {
    providers: 'Providers',
    mcp: 'Mcp/Rules',
    skills: 'skills',
    ohmy: 'oh-my-opencode',
    backup: 'Backup',
    status: 'Status',
    claudeCode: 'Claude Code',
    codex: 'Codex',
    gemini: 'Gemini',
    prompts: 'Prompts',
    speedTest: 'Speed Test',
    usage: 'Usage Stats'
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
    en: 'English',
    ja: '日本語'
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
    hideApiKey: 'Hide',
    // Multi-URL management
    baseUrls: 'Base URL List',
    addUrl: 'Add URL',
    testAllUrls: 'Test All URLs',
    autoSelectFastest: 'Auto Select Fastest',
    activeUrl: 'Active',
    latencyExcellent: 'Excellent',
    latencyGood: 'Good',
    latencyFair: 'Fair',
    latencyPoor: 'Poor',
    latencyFailed: 'Failed',
    notTested: 'Not Tested',
    modelManagement: 'Model Management',
    addModel: 'Add Model',
    noModels: 'No models',
    // Open Switch unified config
    saveToOpenSwitch: 'Save to Open Switch Unified Config',
    saveToOpenSwitchDesc: 'Save configuration to ~/.open-switch/config.json for cross-tool sharing'
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
    adding: 'Adding...',
    reasoningEffort: 'Reasoning Effort',
    reasoningEffortHint: 'Only for GPT5.2/GPT5.1 reasoning models, select "None" for regular models',
    thinkingBudget: 'Thinking Budget',
    thinkingBudgetHint: 'Controls the thinking tokens for Claude models, higher values allow deeper reasoning',
    editModel: 'Edit Model',
    edit: 'Edit'
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
    selectTarget: 'Please select at least one target',
    cliTools: 'CLI Tools'
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
    selectItem: 'Select an item from the list to view details',
    type: 'Type',
    local: 'Local',
    remote: 'Remote',
    status: 'Status',
    statusEnabled: 'Enabled',
    statusDisabled: 'Disabled',
    command: 'Command',
    url: 'URL',
    installPath: 'Install Path',
    package: 'Package',
    effective: 'Effective',
    effectiveYes: '✓ Active',
    effectiveNo: '✗ Inactive',
    effectiveDisabled: 'Disabled',
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
    customConfigHint: 'Supports OpenCode format: command, args, env, type, url, headers, oauth, timeout, enabled',
    customNameRequired: 'Please enter server name',
    customJsonInvalid: 'Invalid JSON format',
    customCommandRequired: 'Please configure command or args',
    customUrlRequired: 'Please configure URL',
    customAdded: 'Added {name}',
    customUpdated: 'Updated {name}',
    editTitle: 'Edit MCP Server',
    // Delete
    deleteConfirm: 'Are you sure you want to delete MCP server "{name}"?',
    serverDeleted: 'Deleted {name}',
    // Sync
    syncToOpenCode: 'Sync to OpenCode',
    syncing: 'Syncing...',
    syncSuccess: 'Synced to ~/.opencode/opencode.json',
    syncFailed: 'Sync failed'
  },

  // Backup page
  backup: {
    title: 'Backup & Import',
    exportTitle: 'Export Backup',
    exportDesc: 'Export all current configurations to a JSON file for migration to other devices or backup.',
    exportBtn: 'Export Backup File',
    exporting: 'Exporting...',
    exportSuccess: 'Export successful! Contains {providers} providers, {models} models, {mcp} MCP, {rules} rules, {skills} skills',
    exportFailed: 'Export failed',
    
    importTitle: 'Import Configuration',
    importDesc: 'Import configurations from a backup file. You can selectively import providers, MCP, rules and skills.',
    selectFile: 'Select Backup File',
    importing: 'Importing...',
    importBtn: 'Start Import',
    importSuccess: 'Import successful! Imported {providers} providers, {mcp} MCP, {rules} rules, {skills} skills',
    importPartial: 'Partial import: {providers} providers, {mcp} MCP, {rules} rules, {skills} skills ({errors} errors)',
    importFailed: 'Import failed',
    previewFailed: 'Failed to read backup file',
    
    backupVersion: 'Backup Version',
    backupTime: 'Backup Time',
    providers: 'Providers',
    models: 'Models',
    rules: 'Rules',
    items: 'items',
    
    importOptions: 'Import Options',
    importProviders: 'Import Provider Configurations',
    importMcp: 'Import MCP Servers',
    importRules: 'Import Rules',
    importSkills: 'Import Skills',
    overwriteExisting: 'Overwrite existing configurations',
    overwriteHint: 'When checked, existing configurations with the same name will be overwritten; otherwise they will be skipped',
    
    providerPreview: 'Provider Preview',
    
    whatIncluded: 'What is included in the backup?',
    includeProviders: 'Provider configurations (including API Keys, URLs, model lists)',
    includeMcp: 'MCP server configurations',
    includeRules: 'Global rule files',
    includeSkills: 'Global Skills files',
    securityWarning: 'Backup file contains sensitive API Key information, please keep it safe!'
  },

  // Skills page
  skills: {
    title: 'Skills',
    addRecommended: 'Add Recommended Skills',
    refresh: 'Refresh',
    noSkills: 'No Skills installed',
    installFirst: 'Install your first Skills',
    recommended: 'Recommended Skills',
    installLocation: 'Install Location',
    selected: '{count} selected',
    installing: 'Installing...',
    installAll: 'Install All',
    installed: 'Installed',
    view: 'View Content',
    deleteConfirm: "Are you sure you want to delete Skills '{name}'?",
    // Discover Skills
    discover: 'Skills Hub',
    discoverTitle: 'Skills Hub',
    discovering: 'Fetching Skills from repositories...',
    noSkillsFound: 'No Skills found',
    installSelected: 'Install Selected',
    searchPlaceholder: 'Search by name or description...',
    viewSource: 'View Source',
    noDescription: 'No description available',
    // Repository management
    manageRepos: 'Manage Repos',
    repoManagement: 'Skills Repository Management',
    addRepo: 'Add Repository',
    repoUrlPlaceholder: 'Enter GitHub repo URL, e.g. https://github.com/user/skills',
    repoUrlHint: 'Supports any GitHub repository with a skills directory',
    builtin: 'Built-in',
    enabled: 'Enabled',
    disabled: 'Disabled',
    enable: 'Enable',
    disable: 'Disable',
    noRepos: 'No Skills repositories',
    // Install locations
    locations: {
      globalOpencode: 'Global OpenCode',
      globalClaude: 'Global Claude',
      projectOpencode: 'Project OpenCode',
      projectClaude: 'Project Claude'
    },
    // Location labels with paths
    locationLabels: {
      GlobalOpenCode: 'Global OpenCode (~/.config/opencode/skills/)',
      GlobalClaude: 'Global Claude (~/.claude/skills/)',
      ProjectOpenCode: 'Project OpenCode (.opencode/skills/)',
      ProjectClaude: 'Project Claude (.claude/skills/)'
    },
    rateLimitError: 'GitHub API rate limit reached, please try again later'
  },

  // Rule page
  rule: {
    title: 'Rules',
    noRules: 'No rules installed',
    addRecommended: 'Add Recommended Rules',
    addCustom: 'Custom Rule',
    deleteConfirm: "Are you sure you want to delete rule '{name}'?",
    deleted: 'Deleted rule {name}',
    saved: 'Saved rule {name}',
    editTitle: 'Edit Rule',
    type: 'Type',
    path: 'Path',
    desc: 'Description',
    recommendedTitle: 'Recommended Rules',
    customTitle: 'Custom Rule',
    installLocation: 'Install Location',
    locationOptions: {
      globalOpencode: 'Global OpenCode (~/.config/opencode/rules/)',
      projectOpencode: 'Project OpenCode (.opencode/rules/)',
      globalClaude: 'Global Claude (~/.claude/rules/)',
      projectClaude: 'Project Claude (.claude/rules/)'
    },
    selectedCount: '{count} selected',
    installing: 'Installing...',
    addAll: 'Install All',
    installed: 'Installed',
    rulesAdded: 'Added {count} rule(s)',
    rulesFailed: '{count} rule(s) failed to install',
    customName: 'Rule Name',
    customNamePlaceholder: 'e.g. my-coding-rules',
    customNameRequired: 'Please enter rule name',
    customContent: 'Rule Content (Markdown)',
    customContentRequired: 'Please enter rule content',
    customContentHint: 'Supports Markdown format, use YAML frontmatter to define globs patterns',
    customAdded: 'Added rule {name}',
    content: 'Rule Content',
    syncToCliTools: 'Sync to CLI Tools',
    syncToCliToolsHint: 'Also write rule content to corresponding CLI tool system prompt files',
    selectInstallTarget: 'Please select at least one install target'
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
    openCode: 'OpenCode',
    // Update related
    checkUpdate: 'Check for Updates',
    checking: 'Checking...',
    noUpdates: 'You are using the latest version',
    updateAvailable: 'Update Available',
    updateVersion: 'New Version',
    downloading: 'Downloading...',
    installing: 'Installing...',
    updateLater: 'Later',
    updateNow: 'Update Now',
    updateNotes: 'Release Notes',
    downloadProgress: 'Download Progress',
    // Auto start
    autoStart: 'Auto Start',
    autoStartDesc: 'Auto launch app on system startup',
    // Environment conflicts
    envConflicts: 'Environment Conflicts',
    noConflicts: 'No Conflicts',
    conflictsFound: '{count} conflict(s) detected',
    conflictVariable: 'Variable',
    conflictSources: 'Conflict Sources'
  },

  // App Settings
  settings: {
    title: 'App Settings',
    description: 'Customize application behavior',
    closeAction: 'On Close',
    closeActionDesc: 'Choose what happens when you click the close button',
    closeAsk: 'Ask',
    closeTray: 'Tray',
    closeQuit: 'Quit',
    closeDialogTitle: 'Close Window',
    closeDialogMessage: 'Minimize to system tray?'
  },

  // oh-my-opencode configuration page
  ohmy: {
    title: 'oh-my-opencode Configuration',
    subtitle: 'Configure models for 7 Agents, install and auto-configure with one click',
    installed: 'Installed',
    notInstalled: 'Not Installed',
    loadFailed: 'Failed to load status',
    
    // Quick set
    quickSet: 'Quick Setup',
    quickSetDesc: 'Set the same model for all Agents',
    selectModelForAll: 'Select model (apply to all)',
    
    // Model selection
    selectModel: 'Select Model',
    usage: 'Usage',
    yourModels: 'Your Configured Models',
    freeModels: 'OpenCode Zen Free Models',
    
    // No models warning
    noModels: 'No available models found',
    noModelsDesc: 'Please add Providers and models in the "Providers" page first, then configure oh-my-opencode.',
    
    // Installation
    installAndConfigure: 'Install & Configure oh-my-opencode',
    installing: 'Installing...',
    installLog: 'Installation Log',
    installSuccess: 'oh-my-opencode installed and configured successfully!',
    installFailed: 'Installation failed, check the log',
    startingInstall: 'Starting installation...',
    
    // Save config
    saveConfig: 'Save Configuration',
    saved: 'Configuration saved',
    saveFailed: 'Failed to save',
    refresh: 'Refresh',
    
    // Uninstall
    uninstall: 'Uninstall',
    uninstalling: 'Uninstalling...',
    confirmUninstall: 'Are you sure you want to uninstall oh-my-opencode? This will remove the config file and plugin registration.',
    uninstallSuccess: 'oh-my-opencode uninstalled successfully!',
    uninstallFailed: 'Uninstall failed, check the log',
    
    // Agent descriptions (fallback, mainly use backend data)
    agents: {
      sisyphus: {
        name: 'Sisyphus',
        desc: 'Main orchestrator'
      },
      oracle: {
        name: 'Oracle',
        desc: 'Architecture design, code review and strategy planning',
        usage: 'Ask @oracle to review this design and propose an architecture'
      },
      librarian: {
        name: 'Librarian',
        desc: 'Multi-repo analysis, documentation lookup and implementation examples',
        usage: 'Ask @librarian how this is implemented—why does the behavior keep changing?'
      },
      explore: {
        name: 'Explore',
        desc: 'Fast codebase exploration and pattern matching',
        usage: 'Ask @explore for the policy on this feature'
      },
      frontend: {
        name: 'Frontend',
        desc: 'Frontend UI/UX development',
        usage: 'Delegate to build beautiful user interfaces'
      },
      documentWriter: {
        name: 'Document Writer',
        desc: 'Technical documentation writing'
      },
      multimodalLooker: {
        name: 'Multimodal Looker',
        desc: 'Multimodal content viewing'
      }
    }
  },

  // Deployed Providers Management
  deployed: {
    title: 'Deployed Providers',
    sectionTitle: 'Deployed Providers',
    manageTitle: 'Manage Deployed Providers',
    manageDesc: 'View and manage providers deployed to opencode configuration files',
    manage: 'Manage',
    description: 'The following providers are deployed to opencode configuration files. You can remove providers you no longer need.',
    noProviders: 'No deployed providers',
    global: 'Global',
    project: 'Project',
    models: 'models',
    removeAll: 'Remove All',
    syncAll: 'Sync All'
  },

  // Deep Link Configuration
  deepLink: {
    title: 'Add Provider',
    subtitle: 'Via Deep Link',
    confirmMessage: 'Do you want to add the following provider configuration?',
    providerName: 'Provider Name',
    baseUrl: 'Base URL',
    apiKey: 'API Key',
    modelType: 'API Protocol',
    customModels: 'Custom Models',
    description: 'Description',
    securityNote: 'Please confirm this configuration is from a trusted source. The API Key will be saved to your local config file.',
    confirm: 'Confirm Add',
    adding: 'Adding...',
    success: 'Provider added successfully!',
    error: 'Failed to add'
  },

  // Claude Code Configuration
  claudeCode: {
    description: 'Manage Claude Code CLI configuration',
    notConfigured: 'Not Configured',
    usingOfficial: 'Using Official API',
    mcpServers: 'MCP Servers',
    servers: '',
    setApiKey: 'Set API Key',
    apiKeyPlaceholder: 'Enter Anthropic API Key (sk-ant-...)',
    setBaseUrl: 'Set Base URL',
    baseUrlPlaceholder: 'Leave empty for official API, or enter custom URL',
    baseUrlHint: 'Leave empty to use Anthropic official API, enter custom URL for third-party proxy',
    setModel: 'Set Default Model',
    modelPlaceholder: 'e.g. claude-sonnet-4-5-20250929'
  },

  // Codex Configuration
  codex: {
    description: 'Manage OpenAI Codex CLI configuration',
    authStatus: 'Auth Status',
    authenticated: 'Authenticated',
    notAuthenticated: 'Not Authenticated',
    providers: 'Model Providers',
    configured: 'configured',
    mcpServers: 'MCP Servers',
    servers: '',
    modelProviders: 'Model Providers',
    noProviders: 'No custom model providers',
    addProvider: 'Add Provider',
    providerKey: 'Provider Key',
    providerKeyPlaceholder: 'e.g. custom-provider',
    displayName: 'Display Name',
    displayNamePlaceholder: 'e.g. My Custom Provider',
    envKey: 'Environment Variable',
    envKeyPlaceholder: 'e.g. CUSTOM_API_KEY'
  },

  // Gemini Configuration
  gemini: {
    description: 'Manage Google Gemini CLI configuration',
    notConfigured: 'Not Configured',
    authMode: 'Auth Mode',
    mcpServers: 'MCP Servers',
    servers: '',
    setApiKey: 'Set API Key',
    apiKeyPlaceholder: 'Enter Gemini API Key',
    setBaseUrl: 'Set Base URL',
    baseUrlPlaceholder: 'Leave empty for official API, or enter custom URL',
    baseUrlHint: 'Leave empty to use Google official API, enter custom URL for third-party proxy',
    setModel: 'Set Default Model',
    modelPlaceholder: 'e.g. gemini-2.5-pro'
  },

  // Prompts Management
  prompts: {
    title: 'Prompts Management',
    description: 'Manage system prompts for CLI tools',
    characters: 'characters',
    placeholder: 'Enter system prompt content here (supports Markdown)...',
    syncTo: 'Sync to',
    sync: 'Sync',
    presets: 'Preset Templates',
    saved: 'Saved successfully',
    confirmDelete: 'Are you sure you want to delete this Prompt file?'
  },

  // Speed Test
  speedTest: {
    title: 'Speed Test',
    description: 'Test API endpoint response latency',
    selectProviders: 'Select providers to test',
    noProviders: 'No enabled providers. Please add and enable providers first.',
    selectAll: 'Select All',
    runTest: 'Run Test',
    testing: 'Testing...',
    noResults: 'Click "Run Test" to test selected providers',
    retest: 'Retest',
    avgLatency: 'Avg Latency',
    successRate: 'Success Rate',
    testCount: 'Test Count'
  },

  // Extended Navigation
  navExt: {
    claudeCode: 'Claude Code',
    codex: 'Codex',
    gemini: 'Gemini',
    prompts: 'Prompts',
    speedTest: 'Speed Test',
    tools: 'Tools'
  },

  // Usage Statistics
  usage: {
    title: 'Usage Statistics',
    description: 'View AI model usage and cost statistics',
    totalRequests: 'Total Requests',
    totalCost: 'Total Cost',
    totalTokens: 'Total Tokens',
    cacheTokens: 'Cache Tokens',
    cacheCreation: 'Creation',
    cacheHit: 'Hits',
    trend: 'Usage Trend',
    past24h: 'Past 24 Hours (Hourly)',
    past7d: 'Past 7 Days',
    past30d: 'Past 30 Days',
    requests: 'Requests',
    cost: 'Cost',
    clearStats: 'Clear Statistics',
    confirmClear: 'Are you sure you want to clear all usage statistics? This action cannot be undone.',
    // Proxy Control
    proxyControl: 'Proxy Control',
    startProxy: 'Start Proxy',
    stopProxy: 'Stop Proxy',
    takeover: 'Takeover Config',
    uptime: 'Uptime',
    success: 'Success',
    failed: 'Failed',
    successRate: 'Success Rate',
    byProvider: 'By Provider',
    noData: 'No Data'
  }
}
