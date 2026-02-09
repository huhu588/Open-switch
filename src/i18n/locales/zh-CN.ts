export default {
  // 应用
  app: {
    title: 'Ai Switch工具'
  },

  // 通用
  common: {
    loading: '加载中...',
    cancel: '取消',
    save: '保存',
    saving: '保存中...',
    confirm: '确认',
    delete: '删除',
    edit: '编辑',
    add: '添加',
    apply: '应用',
    applying: '应用中...',
    selectAll: '全选',
    clearAll: '清空',
    tip: '提示',
    optional: '可选',
    refresh: '刷新',
    done: '完成',
    close: '关闭',
    noDescription: '暂无描述',
    recommended: '推荐',
    notSelected: '未选择'
  },

  // 导航
  nav: {
    providers: '服务商',
    mcp: 'Mcp/规则',
    skills: 'skills',
    ohmy: 'oh-my-opencode',
    backup: '备份',
    status: '状态',
    usage: '使用统计',
    chatMigration: '对话迁移',
    devenv: '环境管理'
  },

  // 导航分组标题
  navExt: {
    tools: '工具'
  },

  // 系统状态
  system: {
    operational: '系统运行正常',
    darkMode: '深色模式',
    lightMode: '浅色模式'
  },

  // 语言
  language: {
    switch: '切换语言',
    zh: '中文',
    en: 'English',
    ja: '日本語'
  },

  // Provider 相关
  provider: {
    title: '服务商',
    new: '新建',
    apply: '应用',
    edit: '编辑',
    delete: '删除',
    noProviders: '暂无 Provider',
    models: '个模型',
    addProvider: '添加 Provider',
    editProvider: '编辑 Provider',
    name: '名称',
    nameRequired: '请输入名称',
    apiKey: 'API Key',
    apiKeyRequired: '请输入 API Key',
    baseUrl: 'Base URL',
    description: '描述',
    preset: '选择预设',
    protocol: 'API 协议',
    protocolAnthropic: 'Anthropic 协议',
    protocolOpenAI: 'OpenAI 协议',
    autoAddModels: '自动添加预设模型',
    getApiKey: '获取 API Key',
    modelsSelected: '已选择 {selected} / {total} 个模型',
    selectAll: '全选',
    unselectAll: '取消全选',
    placeholder: {
      name: 'my-provider',
      apiKey: 'sk-...',
      baseUrl: 'https://api.anthropic.com',
      description: '可选'
    },
    showApiKey: '显示',
    hideApiKey: '隐藏',
    // 多 URL 管理
    baseUrls: 'Base URL 列表',
    addUrl: '添加 URL',
    testAllUrls: '测试所有 URL',
    autoSelectFastest: '自动选择最快',
    activeUrl: '当前使用',
    latencyExcellent: '优秀',
    latencyGood: '良好',
    latencyFair: '一般',
    latencyPoor: '较差',
    latencyFailed: '失败',
    notTested: '未测试',
    modelManagement: '模型管理',
    addModel: '添加模型',
    // Ai Switch 统一配置
    saveToOpenSwitch: '保存到 Ai Switch 统一配置',
    saveToOpenSwitchDesc: '将配置保存到 ~/.ai-switch/config.json，支持跨工具共享',
    noModels: '暂无模型',
    // i18n 补充
    customConfig: '自定义配置',
    modelProvider: '模型提供商',
    applyTo: '应用到',
    applyToHint: '选择要将此服务商配置应用到的工具',
    baseUrlList: 'Base URL 列表',
    autoAddV1Suffix: '自动添加 /v1 后缀',
    baseUrlInputPlaceholder: '输入 Base URL，如 https://api.example.com',
    testing: '测试中...',
    testFailed: '测试失败: {error}',
    noTestResults: '没有可用的测试结果',
    apiKeyRequiredFirst: '请先填写 API Key',
    urlRequiredFirst: '请先添加 URL',
    baseUrlRequired: '请至少添加一个 Base URL',
    opencodeModelRequired: 'OpenCode 需要至少一个模型，请添加模型或取消勾选 OpenCode',
    deleteModelFailed: '删除模型失败: {error}',
    addModelFailed: '添加模型失败: {error}',
    modelGuidanceBoth: '需要添加模型。OpenCode 使用模型列表，CLI 工具将使用第一个模型作为默认模型。',
    modelGuidanceOpencode: 'OpenCode 需要模型列表才能正常工作，请添加至少一个模型。',
    modelGuidanceCli: 'CLI 工具只需要 API Key 和 Base URL，模型可选（将使用工具默认模型）。',
    quality: {
      excellent: '优秀',
      good: '良好',
      fair: '一般',
      poor: '较差',
      failed: '失败',
      untested: '未测试'
    },
    autoAddPresetModels: '自动添加预设模型',
    selectedPresetModels: '已选择 {selected} / {total} 个预设模型',
    addCustomModel: '添加自定义模型',
    customModelPlaceholder: '输入模型名称，如 gpt-4o-mini',
    customModelsAdded: '已添加 {count} 个自定义模型',
    clearAllModels: '清空全部',
    modelIdPlaceholder: '输入模型 ID',
    applyAllProviders: '应用全部 {count} 个服务商配置',
    disabled: '已禁用',
    speedTesting: '测速中...',
    testLatency: '测试延迟',
    clickToDisable: '点击禁用',
    clickToEnable: '点击启用',
    protocolOpenAICompat: 'OpenAI 兼容协议'
  },

  // Model 相关
  model: {
    title: '模型',
    addModel: '添加模型',
    sync: '同步',
    selectProviderFirst: '请先选择一个 Provider',
    noModels: '暂无模型',
    delete: '删除',
    modelId: 'Model ID',
    modelIdRequired: '请输入 Model ID',
    displayName: '显示名称',
    providerNotSelected: '未选择 Provider',
    placeholder: {
      modelId: 'gpt-4o',
      displayName: '可选，默认使用 ID'
    },
    adding: '添加中...',
    reasoningEffort: '推理强度',
    reasoningEffortHint: '仅适用于 GPT5.2/GPT5.1 等推理模型，普通模型请选择"无"',
    thinkingBudget: '思考预算',
    thinkingBudgetHint: '控制 Claude 模型的深度思考 token 数量，更高的值允许更深入的推理',
    editModel: '编辑模型',
    edit: '编辑'
  },

  // 详情面板
  detail: {
    title: '配置详情',
    selectProvider: '选择一个 Provider 查看详情',
    providerSpec: 'Provider 配置',
    selectedModel: '选中的模型',
    name: '名称',
    endpoint: '端点',
    models: '模型',
    desc: '描述',
    id: 'ID',
    context: '上下文',
    output: '输出',
    available: '个可用',
    tokens: 'tokens'
  },

  // 确认对话框
  confirm: {
    title: '确认',
    defaultMessage: '确定要执行此操作吗？',
    deleteTitle: '确认删除',
    deleteProvider: "确定要删除 Provider '{name}' 吗？",
    deleteModel: "确定要删除 Model '{name}' 吗？"
  },

  // 应用配置
  applyConfig: {
    title: '应用配置',
    applyTo: '将 Provider',
    configTo: '的配置应用到：',
    currentProject: '当前项目',
    globalConfig: '全局配置',
    projectPath: './.opencode/opencode.json',
    globalPath: '~/.opencode/opencode.json',
    selectTarget: '请至少选择一个应用目标',
    cliTools: 'CLI 工具',
    // i18n 补充
    selectProviderForCli: '请先选择一个服务商用于 CLI 配置',
    applyProvidersDesc: '将 {vendor} 厂家的 {count} 个服务商配置应用到：',
    willUseProvider: '将使用服务商：'
  },

  // 获取模型
  fetchModels: {
    title: '获取站点模型',
    fetching: '正在获取模型列表...',
    noModels: '未获取到模型',
    totalModels: '共 {total} 个模型，已选 {selected} 个',
    addModels: '添加 {count} 个模型'
  },

  // MCP 页面
  mcp: {
    title: 'MCP 服务器',
    loading: '加载中...',
    noServers: '暂无 MCP 服务器',
    enabled: '启用',
    disabled: '禁用',
    selectServer: '选择一个 MCP 服务器查看详情',
    selectItem: '选择左侧列表中的项目查看详情',
    type: '类型',
    local: '本地',
    remote: '远程',
    status: '状态',
    statusEnabled: '已启用',
    statusDisabled: '已禁用',
    command: '命令',
    url: 'URL',
    installPath: '安装位置',
    package: '包名',
    effective: '是否生效',
    effectiveYes: '✓ 已生效',
    effectiveNo: '✗ 未生效',
    effectiveDisabled: '已禁用',
    // 新增
    addRecommended: '添加推荐 MCP',
    addCustom: '自定义添加',
    installing: '添加中...',
    recommended: '推荐 MCP 服务器',
    addSelected: '添加选中 ({count})',
    addAll: '添加全部',
    serverAdded: '已添加 {count} 个服务器',
    serverSkipped: '{count} 个服务器已存在，已跳过',
    visitSite: '访问官网',
    // 自定义MCP
    customTitle: '自定义添加 MCP',
    customName: '服务器名称',
    customNamePlaceholder: '例如: my-mcp-server',
    customConfig: 'JSON 配置',
    customConfigHint: '支持 OpenCode 格式: command, args, env, type, url, headers, oauth, timeout, enabled',
    customNameRequired: '请输入服务器名称',
    customJsonInvalid: 'JSON 格式无效',
    customCommandRequired: '请配置 command 或 args',
    customUrlRequired: '请配置 URL',
    customAdded: '已添加 {name}',
    customUpdated: '已更新 {name}',
    editTitle: '编辑 MCP 服务器',
    // 删除
    deleteConfirm: '确定要删除 MCP 服务器 "{name}" 吗？',
    serverDeleted: '已删除 {name}',
    // 同步
    syncToOpenCode: '同步到 OpenCode',
    syncing: '同步中...',
    syncSuccess: '已同步到 ~/.opencode/opencode.json',
    syncFailed: '同步失败',
    // 已部署应用
    deployedApps: '已部署应用',
    import: '导入',
    importFromApp: '从应用导入 MCP',
    importedCount: '已导入 {count} 个',
    skippedCount: '跳过 {count} 个（已存在）',
    failedCount: '失败 {count} 个',
    noMcpToImport: '没有可导入的 MCP',
    importFailed: '导入失败',
    // MCP 管理
    manage: 'MCP管理',
    manageTitle: 'MCP 管理',
    installed: '已安装',
    searchMcp: '搜索 MCP...',
    deleteFromAll: '从所有应用中删除',
    totalMcps: '共 {count} 个 MCP',
    // i18n 补充
    deleteFromAllConfirm: '确定要从所有应用中删除 "{name}" 吗？',
    toggleFailed: '切换失败: {error}',
    deleted: '已删除 {name}',
    deleteFailed: '删除失败: {error}',
    healthCheckFailed: '检查失败',
    clickToAddServer: '点击右上角添加 MCP 服务器',
    locationLabels: {
      global_opencode: 'OpenCode',
      project_opencode: 'OpenCode 项目',
      project_root: '根目录',
      global_claude: 'Claude',
      project_claude: 'Claude 项目',
      global_cursor: 'Cursor',
      global_codex: 'Codex',
      global_gemini: 'Gemini'
    }
  },

  // 备份页面
  backup: {
    title: '备份与导入',
    exportTitle: '导出备份',
    exportDesc: '将当前所有配置导出为一个 JSON 文件，可用于迁移到其他设备或备份保存。',
    exportBtn: '导出备份文件',
    exporting: '导出中...',
    exportSuccess: '导出成功！包含 {providers} 个服务商, {models} 个模型, {mcp} 个 MCP, {rules} 个规则, {skills} 个 Skills',
    exportFailed: '导出失败',
    
    importTitle: '导入配置',
    importDesc: '从备份文件导入配置，可选择性地导入服务商、MCP、规则和 skills。',
    selectFile: '选择备份文件',
    importing: '导入中...',
    importBtn: '开始导入',
    importSuccess: '导入成功！导入了 {providers} 个服务商, {mcp} 个 MCP, {rules} 个规则, {skills} 个 Skills, {codex} 个 Codex, {gemini} 个 Gemini',
    importPartial: '部分导入成功：{providers} 个服务商, {mcp} 个 MCP, {rules} 个规则, {skills} 个 Skills, {codex} 个 Codex, {gemini} 个 Gemini ({errors} 个错误)',
    importFailed: '导入失败',
    previewFailed: '读取备份文件失败',
    
    backupVersion: '备份版本',
    backupTime: '备份时间',
    providers: '服务商',
    models: '模型',
    rules: '规则',
    items: '项',
    
    importOptions: '导入选项',
    importProviders: '导入服务商配置',
    importMcp: '导入 MCP 服务器',
    importRules: '导入规则',
    importSkills: '导入 Skills',
    importCodex: '导入 Codex CLI 配置',
    importGemini: '导入 Gemini CLI 配置',
    overwriteExisting: '覆盖已存在的配置',
    overwriteHint: '勾选后，已存在的同名配置将被覆盖；否则将跳过已存在的配置',
    importUsageStats: '导入使用统计',
    
    providerPreview: '服务商预览',
    
    // 导出选项
    exportOptions: '选择导出内容',
    includeUsageStats: '使用统计数据',
    usageRecords: '条记录',
    
    whatIncluded: '备份包含哪些内容？',
    includeProviders: '服务商配置（包含 API Key、URL、模型列表、Codex/Gemini CLI 配置）',
    includeMcp: 'MCP 服务器配置',
    includeRules: '全局规则文件',
    includeSkills: '全局 Skills 文件',
    includeUsageStatsDesc: '使用统计数据（请求日志、Token 用量、成本记录）',
    securityWarning: '备份文件包含敏感的 API Key 信息，请妥善保管！',
    // 导出详细选择面板
    confirmExport: '确认导出',
    chatRecords: '对话记录',
    loadingData: '加载数据中...',
    noExtractedChat: '暂无已提取的对话，请先扫描并提取数据源',
    geminiEnvConfig: '环境配置',
  },

  // Skills 页面
  skills: {
    title: 'Skills 技能',
    addRecommended: '添加推荐 Skills',
    refresh: '刷新',
    noSkills: '暂无已安装的 Skills',
    installFirst: '安装第一个 Skills',
    recommended: '推荐 Skills',
    installLocation: '安装位置',
    selected: '已选择 {count} 个',
    installing: '安装中...',
    installAll: '安装全部',
    installed: '已安装',
    view: '查看内容',
    deleteConfirm: "确定要删除 Skills '{name}' 吗？",
    // 发现技能
    discover: 'Skills库',
    discoverTitle: 'Skills库',
    discovering: '正在从仓库获取技能列表...',
    noSkillsFound: '未找到可用的技能',
    installSelected: '安装选中',
    searchPlaceholder: '搜索技能名称或描述...',
    viewSource: '查看源码',
    noDescription: '暂无描述',
    // 仓库管理
    manageRepos: '仓库管理',
    repoManagement: 'skills仓库管理',
    addRepo: '添加仓库',
    repoUrlPlaceholder: '输入 GitHub 仓库 URL，如 https://github.com/user/skills',
    repoUrlHint: '支持任何包含 skills 目录的 GitHub 仓库',
    builtin: '内置',
    enabled: '已启用',
    disabled: '已禁用',
    enable: '启用',
    disable: '禁用',
    noRepos: '暂无技能仓库',
    // 安装位置
    locations: {
      globalOpencode: '全局 OpenCode',
      globalClaude: '全局 Claude',
      globalCursor: '全局 Cursor',
      projectOpencode: '项目 OpenCode',
      projectClaude: '项目 Claude'
    },
    // 位置标签（带路径）
    locationLabels: {
      GlobalOpenCode: '全局 OpenCode (~/.config/opencode/skills/)',
      GlobalClaude: '全局 Claude (~/.claude/skills/)',
      GlobalCursor: '全局 Cursor (~/.cursor/skills/)',
      ProjectOpenCode: '项目 OpenCode (.opencode/skills/)',
      ProjectClaude: '项目 Claude (.claude/skills/)'
    },
    rateLimitError: 'GitHub API 速率限制已达上限，请稍后再试',
    // i18n 补充
    manage: 'Skills 管理',
    manageTitle: 'Skills 管理',
    searchSkills: '搜索 Skills...',
    noInstalledSkills: '暂无已安装的 Skills',
    local: '本地',
    deleteFromAll: '从所有工具中删除',
    deleteFromAllConfirm: '确定要从所有工具中删除 "{name}" 吗？',
    totalSkills: '共 {count} 个 Skills',
    statsInstalled: '已安装 · Claude: {claude} · Codex: {codex} · Gemini: {gemini} · OpenCode: {opencode} · Cursor: {cursor}',
    toggleFailed: '切换失败: {error}',
    deleteSuccess: '删除成功',
    deleteFailed: '删除失败: {error}',
    readFailed: '读取失败',
    installSuccess: '成功安装 {count} 个 skills',
    installFailed: '{count} 个 skills 安装失败',
    repoAddSuccess: '仓库添加成功',
    repoAddFailed: '添加失败: {error}',
    repoDeleted: '仓库已删除',
    repoDeleteFailed: '删除失败: {error}',
    operationFailed: '操作失败: {error}',
    discoverFailed: '获取技能列表失败',
    viewOnGithub: '在 GitHub 上查看'
  },

  // 规则页面
  rule: {
    title: '规则',
    noRules: '暂无已安装的规则',
    addRecommended: '添加推荐规则',
    addCustom: '自定义规则',
    deleteConfirm: "确定要删除规则 '{name}' 吗？",
    deleted: '已删除规则 {name}',
    saved: '已保存规则 {name}',
    editTitle: '编辑规则',
    type: '类型',
    path: '路径',
    desc: '描述',
    recommendedTitle: '推荐规则',
    customTitle: '自定义规则',
    installLocation: '安装位置',
    locationOptions: {
      globalOpencode: '全局 OpenCode (~/.config/opencode/rules/)',
      projectOpencode: '项目 OpenCode (.opencode/rules/)',
      globalClaude: '全局 Claude (~/.claude/rules/)',
      projectClaude: '项目 Claude (.claude/rules/)'
    },
    selectedCount: '已选择 {count} 个',
    installing: '安装中...',
    addAll: '安装全部',
    installed: '已安装',
    rulesAdded: '已添加 {count} 个规则',
    rulesFailed: '{count} 个规则安装失败',
    customName: '规则名称',
    customNamePlaceholder: '例如: my-coding-rules',
    customNameRequired: '请输入规则名称',
    customContent: '规则内容 (Markdown)',
    customContentRequired: '请输入规则内容',
    customContentHint: '支持 Markdown 格式，可使用 YAML frontmatter 定义 globs 匹配模式',
    customAdded: '已添加规则 {name}',
    content: '规则内容',
    syncToCliTools: '同步到 CLI 工具',
    syncToCliToolsHint: '同时将规则内容写入对应 CLI 工具的系统提示文件',
    selectInstallTarget: '请至少选择一个安装位置',
    // 规则管理
    manage: '规则管理',
    manageTitle: '规则管理',
    searchRule: '搜索规则...',
    deleteFromAll: '从所有应用中删除',
    totalRules: '共 {count} 个规则',
    // i18n 补充
    deleteFromAllConfirm: '确定要从所有应用中删除 "{name}" 吗？',
    deleteFromAllConfirmMultiple: '确定要从所有应用中删除规则 "{name}" 吗？（共 {count} 个副本）',
    toggleFailed: '切换失败: {error}',
    deleteFailed: '删除失败: {error}',
    clickToAddRule: '点击右上角添加规则',
    deployedTo: '已部署到',
    ruleContent: '规则内容',
    categoryLabels: {
      code_style: '代码风格',
      project: '项目结构',
      review: '代码审查',
      testing: '测试',
      workflow: '工作流',
      api: 'API',
      security: '安全',
      documentation: '文档'
    }
  },

  // 状态页面
  status: {
    title: '系统状态',
    appInfo: '应用信息',
    currentVersion: '当前版本',
    providerCount: 'Provider 数量',
    configStatus: '配置状态',
    globalConfig: '全局配置',
    projectConfig: '项目配置',
    configured: '✓ 已配置',
    notConfigured: '✗ 未配置',
    currentProvider: '当前 Provider',
    mcpServers: 'MCP 服务器',
    count: '{count} 个',
    configPaths: '配置路径',
    openCode: 'OpenCode',
    // 更新相关
    checkUpdate: '检查更新',
    checking: '检查中...',
    noUpdates: '当前已是最新版本',
    updateAvailable: '发现新版本',
    updateVersion: '新版本',
    downloading: '下载中...',
    installing: '安装中...',
    updateLater: '稍后更新',
    updateNow: '立即更新',
    updateNotes: '更新内容',
    downloadProgress: '下载进度',
    // 自动启动
    autoStart: '开机自启动',
    autoStartDesc: '系统启动时自动运行应用',
    // 环境变量冲突
    envConflicts: '环境变量冲突',
    noConflicts: '无冲突',
    conflictsFound: '检测到 {count} 个冲突',
    conflictVariable: '变量',
    conflictSources: '冲突来源',
    // CLI 工具版本检测
    cliTools: {
      title: 'CLI 工具版本',
      notInstalled: '未安装',
      checkingLatest: '查询最新版本中...',
      latest: '最新',
      newAvailable: '可更新',
      upToDate: '已是最新',
      update: '更新',
      updating: '更新中...',
      updateSuccess: '更新成功',
      install: '安装',
      installing: '安装中...',
      installSuccess: '安装成功'
    }
  },

  // 应用设置
  settings: {
    title: '应用设置',
    description: '自定义应用程序的行为',
    closeAction: '关闭窗口时',
    closeActionDesc: '选择点击关闭按钮时的默认行为',
    closeAsk: '询问',
    closeTray: '托盘',
    closeQuit: '退出',
    closeDialogTitle: '关闭窗口',
    closeDialogMessage: '是否最小化到系统托盘？'
  },

  // oh-my-opencode 配置页面
  ohmy: {
    title: 'oh-my-opencode 配置',
    subtitle: '为 7 个 Agent 配置模型，一键安装并自动配置',
    installed: '已安装',
    notInstalled: '未安装',
    loadFailed: '加载状态失败',
    
    // 快速设置
    quickSet: '快速设置',
    quickSetDesc: '为所有 Agent 设置相同的模型',
    selectModelForAll: '选择模型（应用到全部）',
    
    // 模型选择
    selectModel: '选择模型',
    usage: '用法示例',
    yourModels: '您配置的模型',
    freeModels: 'OpenCode Zen 免费模型',
    
    // 无模型提示
    noModels: '未找到可用模型',
    noModelsDesc: '请先在"服务商"页面添加 Provider 和模型，然后再配置 oh-my-opencode。',
    
    // 安装相关
    installAndConfigure: '安装并配置 oh-my-opencode',
    installing: '安装中...',
    installLog: '安装日志',
    installSuccess: 'oh-my-opencode 安装配置成功！',
    installFailed: '安装失败，请查看日志',
    startingInstall: '开始安装...',
    
    // 保存配置
    saveConfig: '保存配置',
    saved: '配置已保存',
    saveFailed: '保存失败',
    refresh: '刷新',
    
    // 卸载相关
    uninstall: '卸载',
    uninstalling: '卸载中...',
    confirmUninstall: '确定要卸载 oh-my-opencode 吗？这将删除配置文件和插件注册。',
    uninstallSuccess: 'oh-my-opencode 卸载成功！',
    uninstallFailed: '卸载失败，请查看日志',
    
    // 更新相关
    update: '更新',
    updating: '更新中...',
    startingUpdate: '开始更新...',
    updateSuccess: 'oh-my-opencode 更新成功！',
    updateFailed: '更新失败，请查看日志',
    latestVersion: '已是最新版本',
    
    // Agent 介绍（备用，主要使用后端返回的数据）
    agents: {
      sisyphus: {
        name: 'Sisyphus',
        desc: '主要编排者'
      },
      oracle: {
        name: 'Oracle',
        desc: '架构设计、代码审查和策略制定',
        usage: 'Ask @oracle to review this design and propose an architecture'
      },
      librarian: {
        name: 'Librarian',
        desc: '多仓库分析、文档查找和实现示例',
        usage: 'Ask @librarian how this is implemented—why does the behavior keep changing?'
      },
      explore: {
        name: 'Explore',
        desc: '快速代码库探索和模式匹配',
        usage: 'Ask @explore for the policy on this feature'
      },
      frontend: {
        name: 'Frontend',
        desc: '前端 UI/UX 开发',
        usage: '委托构建精美的用户界面'
      },
      documentWriter: {
        name: 'Document Writer',
        desc: '技术文档编写'
      },
      multimodalLooker: {
        name: 'Multimodal Looker',
        desc: '多模态内容查看'
      }
    }
  },

  // 已部署服务商管理
  deployed: {
    title: '已部署的服务商',
    sectionTitle: '已部署服务商',
    manageTitle: '管理已部署的服务商',
    manageDesc: '查看并管理已应用到 opencode 配置文件中的服务商',
    manage: '管理',
    description: '以下服务商已部署到 opencode 配置文件中，可以在此处删除不再需要的服务商。',
    noProviders: '暂无已部署的服务商',
    global: '全局',
    project: '项目',
    models: '个模型',
    removeAll: '删除全部',
    syncAll: '一键同步',
    // i18n 补充
    modelsFixedSuccess: '已补齐 {count} 个服务商模型',
    noProvidersToFix: '没有需要补齐的服务商',
    modelsFixFailed: '补齐失败: {names}',
    externalTool: '外部工具',
    cannotDeleteSource: '无法删除来源为 "{tool}" 的服务商',
    partialDeleteFailed: '部分删除失败: {names}',
    importedFrom: '从 {tool} 导入',
    importedSuccess: '成功导入 {count} 个',
    skippedExisting: '跳过已存在 {count} 个',
    selectProvidersToImport: '点击选择要导入的服务商（已选 {count} 个）',
    deselectAll: '取消全选',
    selectAll: '全选',
    apiKeyConfigured: '已配置 API Key',
    configured: '已配置',
    fillMissingModels: '一键补齐模型',
    confirmImport: '确认导入 ({count})',
    selectModelType: '选择模型类型',
    selectModelTypeFor: '为 {name} 选择所属的模型类型',
    claudeModelDesc: 'Anthropic Claude 模型',
    codexModelDesc: 'OpenAI GPT / Codex 模型',
    geminiModelDesc: 'Google Gemini 模型'
  },

  // 深链接配置
  deepLink: {
    title: '添加服务商',
    subtitle: '通过深链接配置',
    confirmMessage: '是否添加以下服务商配置？',
    providerName: '服务商名称',
    baseUrl: 'Base URL',
    apiKey: 'API Key',
    modelType: 'API 协议',
    customModels: '自定义模型',
    description: '描述',
    securityNote: '请确认此配置来自可信来源。API Key 将被保存到本地配置文件中。',
    confirm: '确认添加',
    adding: '添加中...',
    success: '服务商添加成功！',
    error: '添加失败'
  },

  

  // 使用统计
  usage: {
    title: '使用统计',
    description: '查看 AI 模型的使用情况和成本统计',
    totalRequests: '总请求数',
    totalCost: '总成本',
    totalTokens: '总Token数',
    textTokens: '文本Token数',
    cursorTokenTooltip: '实际Token = 文本Token + 代码上下文Token + 缓存等（本地仅记录文本Token）',
    cacheTokens: '缓存 Token',
    cacheCreation: '创建',
    cacheHit: '命中',
    totalDuration: '累计耗时',
    conversations: '对话数',
    trend: '使用趋势',
    past24h: '过去 24 小时（按小时）',
    past7d: '过去 7 天',
    past30d: '过去 30 天',
    pastAll: '全部时间',
    period24h: '24小时',
    period7d: '7天',
    period30d: '30天',
    periodAll: '全部',
    requests: '请求数',
    cost: '成本',
    model: '模型',
    unknownModel: '未知模型',
    clearStats: '清除统计',
    confirmClear: '确定要清除所有使用统计数据吗？此操作不可撤销。',
    // 代理控制
    proxyControl: '代理控制',
    startProxy: '启动代理',
    stopProxy: '停止代理',
    takeover: '接管配置',
    uptime: '运行时长',
    success: '成功',
    failed: '失败',
    successRate: '成功率',
    byProvider: '按服务商统计',
    noData: '暂无数据',
    // 本地日志导入
    importLocalLogs: '导入本地日志',
    scanning: '扫描中...',
    files: '个文件',
    entries: '条记录',
    notFound: '未找到',
    existingRecords: '已导入记录',
    noLogsFound: '未找到可导入的日志文件',
    clearLocalLogs: '清除本地日志',
    confirmClearLocal: '确定要清除所有本地导入的日志吗？',
    clearedLocalLogs: '已清除本地日志记录数',
    import: '导入',
    importing: '导入中...',
    importFailed: '导入失败',
    importComplete: '导入完成',
    imported: '已导入',
    skipped: '已跳过（重复）',
    failedEntries: '失败条目',
    // 日志保留设置
    logRetention: '日志保留',
    logRetentionDesc: '选择使用统计日志的保留时间',
    retentionPermanent: '永久',
    retention30Days: '30 天',
    // 模型定价设置
    modelPricing: '模型定价',
    modelPricingDesc: '自定义各模型的价格用于成本计算',
    editPricing: '编辑定价',
    inputCost: '输入价格 ($/M tokens)',
    outputCost: '输出价格 ($/M tokens)',
    cacheReadCost: '缓存读取价格',
    cacheCreationCost: '缓存创建价格',
    resetPricing: '重置为默认',
    confirmResetPricing: '确定要将所有模型定价重置为默认值吗？',
    confirmDeletePricing: '确定要删除此定价配置吗？',
    input: '输入',
    output: '输出',
    // 服务商筛选
    allProviders: '全部',
    noProviderData: '暂无该服务商的统计数据',
    // 服务商定价
    selectProvider: '选择服务商',
    selectProviderHint: '请先选择一个服务商来设置其特定的模型价格。如果没有设置服务商特定价格，将使用下方的默认价格。',
    customPricing: '定价',
    defaultPricing: '默认定价（全局）',
    noCustomPricing: '暂无自定义定价，点击添加按钮创建',
    modelId: '模型 ID',
    selectModel: '请选择模型',
    // 模型趋势图
    modelTokenUsage: '模型 Token 用量',
    dailyUsage: '每日用量',
    cumulativeUsage: '累计用量',
    moreModels: '其他模型',
    otherModels: '其他模型',
    total: '总计',
    // 自动导入
    autoImport: '自动导入日志',
    autoImportDesc: '打开使用统计页面时自动导入本地日志',
    autoImported: '已自动导入 {count} 条记录',
    // 会话统计
    conversationStats: '对话统计',
    totalConversations: '总对话数',
    totalToolCalls: '工具调用数',
    mcpCount: 'MCP 数量',
    filesChanged: '文件变更',
    codeChanges: '代码变更',
    avgResponseTime: '平均响应时间',
    avgThinkingTime: '平均思考时间',
    totalTime: '总耗时',
    // 工具调用统计
    toolCallStats: '工具调用统计',
    moreTools: '其他工具',
    expand: '展开',
    collapse: '收起',
    toolTypes: '种工具',
    calls: '次调用',
    viewAll: '查看全部 ({count} 种)'
  },

  // 对话迁移
  chatMigration: {
    title: '对话迁移',
    description: '从 AI 编程工具中提取完整对话历史并导出为 JSONL 文件',
    scan: '扫描',
    scanning: '扫描中...',
    extract: '提取',
    extracting: '提取中...',
    export: '导出',
    exporting: '导出中...',
    exportAll: '导出全部',
    exportSelected: '导出选中',
    conversations: '个对话',
    messages: '条消息',
    noData: '未检测到数据',
    notInstalled: '未安装',
    detected: '已检测到',
    dataPath: '数据路径',
    scanFirst: '请先扫描数据源',
    scanComplete: '扫描完成',
    extractComplete: '提取完成',
    exportComplete: '导出完成',
    exportSuccess: '成功导出 {count} 个对话到 {path}',
    exportFailed: '导出失败',
    noConversations: '暂无可导出的对话',
    selectExportPath: '选择导出路径',
    totalConversations: '总对话数',
    totalMessages: '总消息数',
    preview: '预览',
    close: '关闭',
    user: '用户',
    assistant: '助手',
    source: '来源',
    createdAt: '创建时间',
    toolUse: '工具调用',
    extractAll: '全部提取',
    extractAllDesc: '从所有可用工具中提取对话',
    clear: '清除',
    clearAll: '清除全部',
    confirmClear: '确定要清除所有已提取的对话吗？',
    fileFormat: '文件格式',
    jsonl: 'JSONL (每行一个对话)',
    // 导入
    importFile: '导入文件',
    importDesc: '从其他电脑导出的 JSONL 文件导入对话',
    importSuccess: '导入完成：新增 {imported} 个，跳过 {skipped} 个（重复）',
    importFailed: '导入失败',
    imported: '已导入',
    skippedDup: '跳过（重复）',
    viewImported: '查看已导入',
    noImportedData: '暂无已导入的对话',
    clearImported: '清除已导入',
    confirmClearImported: '确定要清除所有已导入的迁移对话吗？',
    clearedImported: '已清除所有导入数据',
    // 工具名称
    tools: {
      cursor: 'Cursor',
      claude: 'Claude Code',
      codex: 'Codex',
      windsurf: 'Windsurf',
      trae: 'Trae',
      trae_cn: 'Trae CN'
    }
  },

  // 编程环境管理
  devenv: {
    title: '编程环境管理',
    subtitle: '检测、安装和切换本地编程环境（全球 Top 10）',
    detecting: '检测中...',
    detectAll: '一键检测',
    refresh: '刷新',
    // 状态
    installed: '已安装',
    notInstalled: '未安装',
    currentVersion: '当前版本',
    noVersion: '无',
    // 版本管理器
    versionManager: '版本管理器',
    managerInstalled: '已安装 v{version}',
    managerNotInstalled: '未安装',
    installManager: '安装管理器',
    installing: '安装中...',
    // 版本操作
    installedVersions: '已安装版本',
    switchVersion: '切换版本',
    switching: '切换中...',
    installVersion: '安装版本',
    installingVersion: '安装中...',
    recommendedVersions: '推荐版本',
    forClaude: '适配 Claude',
    stableVersion: '稳定版',
    customVersion: '自定义版本',
    customVersionPlaceholder: '输入版本号，如 20.18.1',
    // 操作结果
    switchSuccess: '切换成功',
    switchFailed: '切换失败',
    installSuccess: '安装成功',
    installFailed: '安装失败',
    managerInstallSuccess: '管理器安装成功',
    managerInstallFailed: '管理器安装失败',
    detectFailed: '检测失败',
    // 卸载
    uninstall: '卸载',
    uninstalling: '卸载中...',
    uninstallSuccess: '卸载成功',
    uninstallFailed: '卸载失败',
    uninstallManager: '卸载管理器',
    managerUninstalling: '卸载中...',
    managerUninstallFailed: '管理器卸载失败',
    // 日志
    operationLog: '操作日志',
    noLogs: '暂无日志',
    clearLogs: '清除日志'
  }
}
