export default {
  // 应用
  app: {
    title: 'Open Switch工具'
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
    optional: '可选'
  },

  // 导航
  nav: {
    providers: '服务商',
    mcp: 'Mcp/规则',
    skills: 'skills',
    backup: '备份',
    status: '状态'
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
    en: 'English'
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
    hideApiKey: '隐藏'
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
    reasoningEffortHint: '仅适用于 GPT5.2/GPT5.1 等推理模型，普通模型请选择“无”'
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
    selectTarget: '请至少选择一个应用目标'
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
    customConfigHint: '支持 OpenCode 格式: command, args, env, type',
    customNameRequired: '请输入服务器名称',
    customJsonInvalid: 'JSON 格式无效',
    customCommandRequired: '请配置 command 或 args',
    customAdded: '已添加 {name}',
    // 删除
    deleteConfirm: '确定要删除 MCP 服务器 "{name}" 吗？',
    serverDeleted: '已删除 {name}',
    // 同步
    syncToOpenCode: '同步到 OpenCode',
    syncing: '同步中...',
    syncSuccess: '已同步到 ~/.opencode/opencode.json',
    syncFailed: '同步失败'
  },

  // 备份页面
  backup: {
    title: '备份与导入',
    exportTitle: '导出备份',
    exportDesc: '将当前所有配置导出为一个 JSON 文件，可用于迁移到其他设备或备份保存。',
    exportBtn: '导出备份文件',
    exporting: '导出中...',
    exportSuccess: '导出成功！包含 {providers} 个服务商, {models} 个模型, {mcp} 个 MCP, {rules} 个规则, {skills} 个 skills',
    exportFailed: '导出失败',
    
    importTitle: '导入配置',
    importDesc: '从备份文件导入配置，可选择性地导入服务商、MCP、规则和 skills。',
    selectFile: '选择备份文件',
    importing: '导入中...',
    importBtn: '开始导入',
    importSuccess: '导入成功！导入了 {providers} 个服务商, {mcp} 个 MCP, {rules} 个规则, {skills} 个 skills',
    importPartial: '部分导入成功：{providers} 个服务商, {mcp} 个 MCP, {rules} 个规则, {skills} 个 skills ({errors} 个错误)',
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
    overwriteExisting: '覆盖已存在的配置',
    overwriteHint: '勾选后，已存在的同名配置将被覆盖；否则将跳过已存在的配置',
    
    providerPreview: '服务商预览',
    
    whatIncluded: '备份包含哪些内容？',
    includeProviders: '服务商配置（包含 API Key、URL、模型列表）',
    includeMcp: 'MCP 服务器配置',
    includeRules: '全局规则文件',
    includeSkills: '全局 Skills 文件',
    securityWarning: '备份文件包含敏感的 API Key 信息，请妥善保管！'
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
      projectOpencode: '项目 OpenCode',
      projectClaude: '项目 Claude'
    },
    // 位置标签（带路径）
    locationLabels: {
      GlobalOpenCode: '全局 OpenCode (~/.config/opencode/skills/)',
      GlobalClaude: '全局 Claude (~/.claude/skills/)',
      ProjectOpenCode: '项目 OpenCode (.opencode/skills/)',
      ProjectClaude: '项目 Claude (.claude/skills/)'
    },
    rateLimitError: 'GitHub API 速率限制已达上限，请稍后再试'
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
    content: '规则内容'
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
    downloadProgress: '下载进度'
  }
}
