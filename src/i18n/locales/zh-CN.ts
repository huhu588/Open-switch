export default {
  // 应用
  app: {
    title: 'Open Switch - Coding Agent 配置管理工具'
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
    mcp: 'MCP',
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
    adding: '添加中...'
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
    type: '类型',
    local: '本地',
    remote: '远程',
    status: '状态',
    statusEnabled: '已启用',
    statusDisabled: '已禁用',
    command: '命令',
    url: 'URL'
  },

  // 备份页面
  backup: {
    title: '备份与恢复',
    featureTitle: '配置备份功能',
    featureDesc: '备份功能支持将您的 Coding Agent 配置备份到 WebDAV 服务器。',
    supportedTypes: '支持的备份类型',
    openCodeConfig: 'OpenCode 配置',
    mcpConfig: 'MCP 服务器配置',
    usage: '使用说明',
    usageDesc: '由于备份涉及网络操作，请使用命令行模式执行备份/恢复：',
    createBackup: '创建备份',
    restoreBackup: '恢复备份',
    tipTitle: '提示',
    tipContent: '备份功能需要配置 WebDAV 服务器。您可以使用坚果云、NextCloud 等支持 WebDAV 的服务。'
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
    openCode: 'OpenCode'
  }
}
