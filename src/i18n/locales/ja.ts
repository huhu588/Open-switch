export default {
  // App
  app: {
    title: 'Ai Switch - コーディングエージェント設定マネージャー'
  },

  // Common
  common: {
    loading: '読み込み中...',
    cancel: 'キャンセル',
    save: '保存',
    saving: '保存中...',
    confirm: '確認',
    delete: '削除',
    edit: '編集',
    add: '追加',
    apply: '適用',
    applying: '適用中...',
    selectAll: '全選択',
    clearAll: 'クリア',
    tip: 'ヒント',
    optional: '任意',
    refresh: '更新',
    done: '完了',
    close: '閉じる'
  },

  // Navigation
  nav: {
    providers: 'プロバイダー',
    mcp: 'Mcp/ルール',
    skills: 'スキル',
    ohmy: 'oh-my-opencode',
    backup: 'バックアップ',
    status: 'ステータス',
    claudeCode: 'Claude Code',
    codex: 'Codex',
    gemini: 'Gemini',
    prompts: 'プロンプト',
    speedTest: '速度テスト',
    usage: '使用統計'
  },

  // System status
  system: {
    operational: 'システム正常',
    darkMode: 'ダークモード',
    lightMode: 'ライトモード'
  },

  // Language
  language: {
    switch: '言語切替',
    zh: '中文',
    en: 'English',
    ja: '日本語'
  },

  // Provider related
  provider: {
    title: 'プロバイダー',
    new: '新規',
    apply: '適用',
    edit: '編集',
    delete: '削除',
    noProviders: 'プロバイダーがありません',
    models: 'モデル',
    addProvider: 'プロバイダーを追加',
    editProvider: 'プロバイダーを編集',
    name: '名前',
    nameRequired: '名前を入力してください',
    apiKey: 'APIキー',
    apiKeyRequired: 'APIキーを入力してください',
    baseUrl: 'ベースURL',
    description: '説明',
    preset: 'プリセットを選択',
    protocol: 'APIプロトコル',
    protocolAnthropic: 'Anthropicプロトコル',
    protocolOpenAI: 'OpenAIプロトコル',
    autoAddModels: 'プリセットモデルを自動追加',
    getApiKey: 'APIキーを取得',
    modelsSelected: '{selected} / {total} モデル選択中',
    selectAll: '全選択',
    unselectAll: '選択解除',
    placeholder: {
      name: 'my-provider',
      apiKey: 'sk-...',
      baseUrl: 'https://api.anthropic.com',
      description: '任意'
    },
    showApiKey: '表示',
    hideApiKey: '非表示',
    // Multi-URL management
    baseUrls: 'ベースURLリスト',
    addUrl: 'URL追加',
    testAllUrls: '全URLテスト',
    autoSelectFastest: '最速を自動選択',
    activeUrl: '有効',
    latencyExcellent: '優秀',
    latencyGood: '良好',
    latencyFair: '普通',
    latencyPoor: '不良',
    latencyFailed: '失敗',
    notTested: '未テスト',
    modelManagement: 'モデル管理',
    addModel: 'モデル追加',
    noModels: 'モデルなし',
    // Ai Switch 統一設定
    saveToOpenSwitch: 'Ai Switch 統一設定に保存',
    saveToOpenSwitchDesc: '設定を ~/.ai-switch/config.json に保存し、ツール間で共有できます'
  },

  // Model related
  model: {
    title: 'モデル',
    addModel: 'モデル追加',
    sync: '同期',
    selectProviderFirst: '先にプロバイダーを選択',
    noModels: 'モデルがありません',
    delete: '削除',
    modelId: 'モデルID',
    modelIdRequired: 'モデルIDを入力してください',
    displayName: '表示名',
    providerNotSelected: 'プロバイダー未選択',
    placeholder: {
      modelId: 'gpt-4o',
      displayName: '任意、デフォルトはID'
    },
    adding: '追加中...',
    reasoningEffort: '推論レベル',
    reasoningEffortHint: 'GPT5.2/GPT5.1推論モデル専用、通常モデルは「なし」を選択',
    thinkingBudget: '思考予算',
    thinkingBudgetHint: 'Claudeモデルの思考トークンを制御、高い値でより深い推論が可能',
    editModel: 'モデル編集',
    edit: '編集'
  },

  // Detail panel
  detail: {
    title: '設定詳細',
    selectProvider: 'プロバイダーを選択して詳細を表示',
    providerSpec: 'プロバイダー仕様',
    selectedModel: '選択モデル',
    name: '名前',
    endpoint: 'エンドポイント',
    models: 'モデル',
    desc: '説明',
    id: 'ID',
    context: 'コンテキスト',
    output: '出力',
    available: '利用可能',
    tokens: 'トークン'
  },

  // Confirm dialog
  confirm: {
    title: '確認',
    defaultMessage: 'この操作を実行しますか？',
    deleteTitle: '削除確認',
    deleteProvider: 'プロバイダー「{name}」を削除しますか？',
    deleteModel: 'モデル「{name}」を削除しますか？'
  },

  // Apply config
  applyConfig: {
    title: '設定を適用',
    applyTo: 'プロバイダー',
    configTo: 'の設定を適用先:',
    currentProject: '現在のプロジェクト',
    globalConfig: 'グローバル設定',
    projectPath: './.opencode/opencode.json',
    globalPath: '~/.opencode/opencode.json',
    selectTarget: '少なくとも1つの対象を選択してください',
    cliTools: 'CLIツール'
  },

  // Fetch models
  fetchModels: {
    title: 'サイトモデルを取得',
    fetching: 'モデルリストを取得中...',
    noModels: 'モデルが見つかりません',
    totalModels: '合計{total}モデル、{selected}選択中',
    addModels: '{count}モデルを追加'
  },

  // MCP page
  mcp: {
    title: 'MCPサーバー',
    loading: '読み込み中...',
    noServers: 'MCPサーバーがありません',
    enabled: '有効',
    disabled: '無効',
    selectServer: 'MCPサーバーを選択して詳細を表示',
    selectItem: 'リストから項目を選択して詳細を表示',
    type: 'タイプ',
    local: 'ローカル',
    remote: 'リモート',
    status: 'ステータス',
    statusEnabled: '有効',
    statusDisabled: '無効',
    command: 'コマンド',
    url: 'URL',
    installPath: 'インストールパス',
    package: 'パッケージ',
    effective: '有効状態',
    effectiveYes: '✓ アクティブ',
    effectiveNo: '✗ 非アクティブ',
    effectiveDisabled: '無効',
    // New
    addRecommended: 'おすすめMCPを追加',
    addCustom: 'カスタム追加',
    installing: '追加中...',
    recommended: 'おすすめMCPサーバー',
    addSelected: '選択を追加 ({count})',
    addAll: '全て追加',
    serverAdded: '{count}サーバーを追加しました',
    serverSkipped: '{count}サーバーは既存のためスキップ',
    visitSite: 'サイトを訪問',
    // Custom MCP
    customTitle: 'カスタムMCP追加',
    customName: 'サーバー名',
    customNamePlaceholder: '例: my-mcp-server',
    customConfig: 'JSON設定',
    customConfigHint: 'OpenCode形式をサポート: command, args, env, type, url, headers, oauth, timeout, enabled',
    customNameRequired: 'サーバー名を入力してください',
    customJsonInvalid: 'JSON形式が無効です',
    customCommandRequired: 'commandまたはargsを設定してください',
    customUrlRequired: 'URLを設定してください',
    customAdded: '{name}を追加しました',
    customUpdated: '{name}を更新しました',
    editTitle: 'MCPサーバーを編集',
    // Delete
    deleteConfirm: 'MCPサーバー「{name}」を削除しますか？',
    serverDeleted: '{name}を削除しました',
    // Sync
    syncToOpenCode: 'OpenCodeに同期',
    syncing: '同期中...',
    syncSuccess: '~/.opencode/opencode.jsonに同期しました',
    syncFailed: '同期に失敗しました',
    // Deployed apps
    deployedApps: 'デプロイ済みアプリ',
    import: 'インポート',
    importFromApp: 'アプリからMCPをインポート',
    importedCount: '{count}件インポート',
    skippedCount: '{count}件スキップ（既存）',
    failedCount: '{count}件失敗',
    noMcpToImport: 'インポートするMCPがありません',
    importFailed: 'インポートに失敗しました',
    // MCP管理
    manage: 'MCP管理',
    manageTitle: 'MCP管理',
    installed: 'インストール済み',
    searchMcp: 'MCPを検索...',
    deleteFromAll: '全てのアプリから削除',
    totalMcps: '合計{count}件のMCP'
  },

  // Backup page
  backup: {
    title: 'バックアップとインポート',
    exportTitle: 'バックアップをエクスポート',
    exportDesc: '現在の全設定をJSONファイルにエクスポートし、他のデバイスへの移行やバックアップに使用します。',
    exportBtn: 'バックアップファイルをエクスポート',
    exporting: 'エクスポート中...',
    exportSuccess: 'エクスポート成功！ {providers}プロバイダー、{models}モデル、{mcp} MCP、{rules}ルール、{skills}スキルを含む',
    exportFailed: 'エクスポート失敗',
    
    importTitle: '設定をインポート',
    importDesc: 'バックアップファイルから設定をインポートします。プロバイダー、MCP、ルール、スキルを選択的にインポートできます。',
    selectFile: 'バックアップファイルを選択',
    importing: 'インポート中...',
    importBtn: 'インポート開始',
    importSuccess: 'インポート成功！ {providers}プロバイダー、{mcp} MCP、{rules}ルール、{skills}スキル、{codex} Codex、{gemini} Gemini',
    importPartial: '部分インポート: {providers}プロバイダー、{mcp} MCP、{rules}ルール、{skills}スキル、{codex} Codex、{gemini} Gemini ({errors}エラー)',
    importFailed: 'インポート失敗',
    previewFailed: 'バックアップファイルの読み込みに失敗',
    
    backupVersion: 'バックアップバージョン',
    backupTime: 'バックアップ日時',
    providers: 'プロバイダー',
    models: 'モデル',
    rules: 'ルール',
    items: '項目',
    
    importOptions: 'インポートオプション',
    importProviders: 'プロバイダー設定をインポート',
    importMcp: 'MCPサーバーをインポート',
    importRules: 'ルールをインポート',
    importSkills: 'スキルをインポート',
    importCodex: 'Codex CLI設定をインポート',
    importGemini: 'Gemini CLI設定をインポート',
    overwriteExisting: '既存の設定を上書き',
    overwriteHint: 'チェック時、同名の既存設定は上書きされます。未チェック時はスキップされます',
    
    providerPreview: 'プロバイダープレビュー',
    
    whatIncluded: 'バックアップに含まれる内容',
    includeProviders: 'プロバイダー設定（APIキー、URL、モデルリストを含む）',
    includeMcp: 'MCPサーバー設定',
    includeRules: 'グローバルルールファイル',
    includeSkills: 'グローバルスキルファイル',
    includeCodex: 'Codex CLI設定（モデルプロバイダー、MCP）',
    includeGemini: 'Gemini CLI設定（APIキー、MCP）',
    securityWarning: 'バックアップファイルには機密のAPIキー情報が含まれています。安全に保管してください！'
  },

  // Skills page
  skills: {
    title: 'スキル',
    addRecommended: 'おすすめスキルを追加',
    refresh: '更新',
    noSkills: 'スキルがインストールされていません',
    installFirst: '最初のスキルをインストール',
    recommended: 'おすすめスキル',
    installLocation: 'インストール場所',
    selected: '{count}選択中',
    installing: 'インストール中...',
    installAll: '全てインストール',
    installed: 'インストール済み',
    view: '内容を表示',
    deleteConfirm: 'スキル「{name}」を削除しますか？',
    // Discover Skills
    discover: 'スキルハブ',
    discoverTitle: 'スキルハブ',
    discovering: 'リポジトリからスキルを取得中...',
    noSkillsFound: 'スキルが見つかりません',
    installSelected: '選択をインストール',
    searchPlaceholder: '名前または説明で検索...',
    viewSource: 'ソースを表示',
    noDescription: '説明なし',
    // Repository management
    manageRepos: 'リポジトリ管理',
    repoManagement: 'スキルリポジトリ管理',
    addRepo: 'リポジトリを追加',
    repoUrlPlaceholder: 'GitHubリポジトリURLを入力、例: https://github.com/user/skills',
    repoUrlHint: 'skillsディレクトリを持つ任意のGitHubリポジトリをサポート',
    builtin: '組み込み',
    enabled: '有効',
    disabled: '無効',
    enable: '有効化',
    disable: '無効化',
    noRepos: 'スキルリポジトリがありません',
    // Install locations
    locations: {
      globalOpencode: 'グローバルOpenCode',
      globalClaude: 'グローバルClaude',
      globalCursor: 'グローバルCursor',
      projectOpencode: 'プロジェクトOpenCode',
      projectClaude: 'プロジェクトClaude'
    },
    // Location labels with paths
    locationLabels: {
      GlobalOpenCode: 'グローバルOpenCode (~/.config/opencode/skills/)',
      GlobalClaude: 'グローバルClaude (~/.claude/skills/)',
      GlobalCursor: 'グローバルCursor (~/.cursor/skills/)',
      ProjectOpenCode: 'プロジェクトOpenCode (.opencode/skills/)',
      ProjectClaude: 'プロジェクトClaude (.claude/skills/)'
    },
    rateLimitError: 'GitHub APIレート制限に達しました。後でお試しください'
  },

  // Rule page
  rule: {
    title: 'ルール',
    noRules: 'ルールがインストールされていません',
    addRecommended: 'おすすめルールを追加',
    addCustom: 'カスタムルール',
    deleteConfirm: 'ルール「{name}」を削除しますか？',
    deleted: 'ルール{name}を削除しました',
    saved: 'ルール{name}を保存しました',
    editTitle: 'ルールを編集',
    type: 'タイプ',
    path: 'パス',
    desc: '説明',
    recommendedTitle: 'おすすめルール',
    customTitle: 'カスタムルール',
    installLocation: 'インストール場所',
    locationOptions: {
      globalOpencode: 'グローバルOpenCode (~/.config/opencode/rules/)',
      projectOpencode: 'プロジェクトOpenCode (.opencode/rules/)',
      globalClaude: 'グローバルClaude (~/.claude/rules/)',
      projectClaude: 'プロジェクトClaude (.claude/rules/)'
    },
    selectedCount: '{count}選択中',
    installing: 'インストール中...',
    addAll: '全てインストール',
    installed: 'インストール済み',
    rulesAdded: '{count}ルールを追加しました',
    rulesFailed: '{count}ルールのインストールに失敗',
    customName: 'ルール名',
    customNamePlaceholder: '例: my-coding-rules',
    customNameRequired: 'ルール名を入力してください',
    customContent: 'ルール内容（Markdown）',
    customContentRequired: 'ルール内容を入力してください',
    customContentHint: 'Markdown形式をサポート、YAML frontmatterでglobsパターンを定義',
    customAdded: 'ルール{name}を追加しました',
    content: 'ルール内容',
    syncToCliTools: 'CLIツールに同期',
    syncToCliToolsHint: 'ルール内容を対応するCLIツールのシステムプロンプトファイルにも書き込む',
    selectInstallTarget: '少なくとも1つのインストール対象を選択してください',
    // ルール管理
    manage: 'ルール管理',
    manageTitle: 'ルール管理',
    installed: 'インストール済み',
    searchRule: 'ルールを検索...',
    deleteFromAll: '全てのアプリから削除',
    totalRules: '合計{count}件のルール'
  },

  // Status page
  status: {
    title: 'システムステータス',
    appInfo: 'アプリ情報',
    currentVersion: '現在のバージョン',
    providerCount: 'プロバイダー数',
    configStatus: '設定ステータス',
    globalConfig: 'グローバル設定',
    projectConfig: 'プロジェクト設定',
    configured: '✓ 設定済み',
    notConfigured: '✗ 未設定',
    currentProvider: '現在のプロバイダー',
    mcpServers: 'MCPサーバー',
    count: '{count}',
    configPaths: '設定パス',
    openCode: 'OpenCode',
    // Update related
    checkUpdate: '更新を確認',
    checking: '確認中...',
    noUpdates: '最新バージョンを使用中',
    updateAvailable: '更新あり',
    updateVersion: '新バージョン',
    downloading: 'ダウンロード中...',
    installing: 'インストール中...',
    updateLater: '後で',
    updateNow: '今すぐ更新',
    updateNotes: 'リリースノート',
    downloadProgress: 'ダウンロード進捗',
    // Auto start
    autoStart: '自動起動',
    autoStartDesc: 'システム起動時にアプリを自動起動',
    // Environment conflicts
    envConflicts: '環境変数の競合',
    noConflicts: '競合なし',
    conflictsFound: '{count}件の競合が検出されました',
    conflictVariable: '変数',
    conflictSources: '競合元'
  },

  // App Settings
  settings: {
    title: 'アプリ設定',
    description: 'アプリの動作をカスタマイズ',
    closeAction: '閉じる時の動作',
    closeActionDesc: '閉じるボタンをクリックした時の動作を選択',
    closeAsk: '確認',
    closeTray: 'トレイ',
    closeQuit: '終了',
    closeDialogTitle: 'ウィンドウを閉じる',
    closeDialogMessage: 'システムトレイに最小化しますか？'
  },

  // oh-my-opencode configuration page
  ohmy: {
    title: 'oh-my-opencode設定',
    subtitle: '7つのエージェントのモデルを設定、ワンクリックでインストールと自動設定',
    installed: 'インストール済み',
    notInstalled: '未インストール',
    loadFailed: 'ステータスの読み込みに失敗',
    
    // Quick set
    quickSet: 'クイック設定',
    quickSetDesc: '全エージェントに同じモデルを設定',
    selectModelForAll: 'モデルを選択（全てに適用）',
    
    // Model selection
    selectModel: 'モデルを選択',
    usage: '使用方法',
    yourModels: '設定済みモデル',
    freeModels: 'OpenCode Zen無料モデル',
    
    // No models warning
    noModels: '利用可能なモデルがありません',
    noModelsDesc: '「プロバイダー」ページでプロバイダーとモデルを追加してから、oh-my-opencodeを設定してください。',
    
    // Installation
    installAndConfigure: 'oh-my-opencodeをインストール＆設定',
    installing: 'インストール中...',
    installLog: 'インストールログ',
    installSuccess: 'oh-my-opencodeのインストールと設定が完了しました！',
    installFailed: 'インストール失敗、ログを確認してください',
    startingInstall: 'インストール開始...',
    
    // Save config
    saveConfig: '設定を保存',
    saved: '設定を保存しました',
    saveFailed: '保存に失敗しました',
    refresh: '更新',
    
    // Uninstall
    uninstall: 'アンインストール',
    uninstalling: 'アンインストール中...',
    confirmUninstall: 'oh-my-opencodeをアンインストールしますか？設定ファイルとプラグイン登録が削除されます。',
    uninstallSuccess: 'oh-my-opencodeをアンインストールしました！',
    uninstallFailed: 'アンインストール失敗、ログを確認してください',
    
    // Agent descriptions
    agents: {
      sisyphus: {
        name: 'Sisyphus',
        desc: 'メインオーケストレーター'
      },
      oracle: {
        name: 'Oracle',
        desc: 'アーキテクチャ設計、コードレビュー、戦略計画',
        usage: '@oracleにこの設計をレビューしてアーキテクチャを提案してもらう'
      },
      librarian: {
        name: 'Librarian',
        desc: 'マルチリポジトリ分析、ドキュメント検索、実装例',
        usage: '@librarianにこれがどう実装されているか聞く'
      },
      explore: {
        name: 'Explore',
        desc: '高速コードベース探索とパターンマッチング',
        usage: '@exploreにこの機能のポリシーを聞く'
      },
      frontend: {
        name: 'Frontend',
        desc: 'フロントエンドUI/UX開発',
        usage: '美しいユーザーインターフェースの構築を委任'
      },
      documentWriter: {
        name: 'Document Writer',
        desc: '技術ドキュメント作成'
      },
      multimodalLooker: {
        name: 'Multimodal Looker',
        desc: 'マルチモーダルコンテンツ表示'
      }
    }
  },

  // Deployed Providers Management
  deployed: {
    title: 'デプロイ済みプロバイダー',
    sectionTitle: 'デプロイ済みプロバイダー',
    manageTitle: 'デプロイ済みプロバイダー管理',
    manageDesc: 'opencode設定ファイルにデプロイされたプロバイダーを表示・管理',
    manage: '管理',
    description: '以下のプロバイダーがopencode設定ファイルにデプロイされています。不要なプロバイダーは削除できます。',
    noProviders: 'デプロイ済みプロバイダーなし',
    global: 'グローバル',
    project: 'プロジェクト',
    models: 'モデル',
    removeAll: '全て削除',
    syncAll: '全て同期'
  },

  // Deep Link Configuration
  deepLink: {
    title: 'プロバイダーを追加',
    subtitle: 'ディープリンク経由',
    confirmMessage: '以下のプロバイダー設定を追加しますか？',
    providerName: 'プロバイダー名',
    baseUrl: 'ベースURL',
    apiKey: 'APIキー',
    modelType: 'APIプロトコル',
    customModels: 'カスタムモデル',
    description: '説明',
    securityNote: 'この設定が信頼できるソースからのものであることを確認してください。APIキーはローカル設定ファイルに保存されます。',
    confirm: '追加を確認',
    adding: '追加中...',
    success: 'プロバイダーを追加しました！',
    error: '追加に失敗しました'
  },

  // Claude Code Configuration
  claudeCode: {
    description: 'Claude Code CLI設定を管理',
    notConfigured: '未設定',
    usingOfficial: '公式APIを使用中',
    mcpServers: 'MCPサーバー',
    servers: '',
    setApiKey: 'APIキーを設定',
    apiKeyPlaceholder: 'Anthropic APIキーを入力 (sk-ant-...)',
    setBaseUrl: 'ベースURLを設定',
    baseUrlPlaceholder: '公式APIは空欄、カスタムURLを入力',
    baseUrlHint: '空欄でAnthropic公式API、カスタムURLでサードパーティプロキシ',
    setModel: 'デフォルトモデルを設定',
    modelPlaceholder: '例: claude-sonnet-4-5-20250929'
  },

  // Codex Configuration
  codex: {
    description: 'OpenAI Codex CLI設定を管理',
    authStatus: '認証ステータス',
    authenticated: '認証済み',
    notAuthenticated: '未認証',
    providers: 'モデルプロバイダー',
    configured: '設定済み',
    mcpServers: 'MCPサーバー',
    servers: '',
    modelProviders: 'モデルプロバイダー',
    noProviders: 'カスタムモデルプロバイダーなし',
    addProvider: 'プロバイダーを追加',
    providerKey: 'プロバイダーキー',
    providerKeyPlaceholder: '例: custom-provider',
    displayName: '表示名',
    displayNamePlaceholder: '例: My Custom Provider',
    envKey: '環境変数',
    envKeyPlaceholder: '例: CUSTOM_API_KEY'
  },

  // Gemini Configuration
  gemini: {
    description: 'Google Gemini CLI設定を管理',
    notConfigured: '未設定',
    authMode: '認証モード',
    mcpServers: 'MCPサーバー',
    servers: '',
    setApiKey: 'APIキーを設定',
    apiKeyPlaceholder: 'Gemini APIキーを入力',
    setBaseUrl: 'ベースURLを設定',
    baseUrlPlaceholder: '公式APIは空欄、カスタムURLを入力',
    baseUrlHint: '空欄でGoogle公式API、カスタムURLでサードパーティプロキシ',
    setModel: 'デフォルトモデルを設定',
    modelPlaceholder: '例: gemini-2.5-pro'
  },

  // Prompts Management
  prompts: {
    title: 'プロンプト管理',
    description: 'CLIツールのシステムプロンプトを管理',
    characters: '文字',
    placeholder: 'システムプロンプト内容を入力（Markdownサポート）...',
    syncTo: '同期先',
    sync: '同期',
    presets: 'プリセットテンプレート',
    saved: '保存しました',
    confirmDelete: 'このプロンプトファイルを削除しますか？'
  },

  // Speed Test
  speedTest: {
    title: '速度テスト',
    description: 'APIエンドポイントの応答遅延をテスト',
    selectProviders: 'テストするプロバイダーを選択',
    noProviders: '有効なプロバイダーがありません。先にプロバイダーを追加して有効にしてください。',
    selectAll: '全選択',
    runTest: 'テスト実行',
    testing: 'テスト中...',
    noResults: '「テスト実行」をクリックして選択したプロバイダーをテスト',
    retest: '再テスト',
    avgLatency: '平均遅延',
    successRate: '成功率',
    testCount: 'テスト回数'
  },

  // Extended Navigation
  navExt: {
    claudeCode: 'Claude Code',
    codex: 'Codex',
    gemini: 'Gemini',
    prompts: 'プロンプト',
    speedTest: '速度テスト',
    tools: 'ツール'
  },

  // Usage Statistics
  usage: {
    title: '使用統計',
    description: 'AIモデルの使用状況とコスト統計を表示',
    totalRequests: '総リクエスト数',
    totalCost: '総コスト',
    totalTokens: '総トークン',
    textTokens: 'テキストトークン',
    cursorTokenTooltip: '実際のトークン = テキストトークン + コードコンテキスト + キャッシュ等（ローカルはテキストトークンのみ記録）',
    cacheTokens: 'キャッシュトークン',
    cacheCreation: '作成',
    cacheHit: 'ヒット',
    totalDuration: '累計時間',
    conversations: '会話数',
    trend: '使用トレンド',
    past24h: '過去24時間（時間別）',
    past7d: '過去7日間',
    past30d: '過去30日間',
    pastAll: '全期間',
    period24h: '24時間',
    period7d: '7日',
    period30d: '30日',
    periodAll: '全部',
    requests: 'リクエスト',
    cost: 'コスト',
    model: 'モデル',
    unknownModel: '不明なモデル',
    clearStats: '統計をクリア',
    confirmClear: '全ての使用統計をクリアしますか？この操作は元に戻せません。',
    // Proxy Control
    proxyControl: 'プロキシ制御',
    startProxy: 'プロキシ開始',
    stopProxy: 'プロキシ停止',
    takeover: '設定を引き継ぐ',
    uptime: '稼働時間',
    success: '成功',
    failed: '失敗',
    successRate: '成功率',
    byProvider: 'プロバイダー別',
    noData: 'データなし',
    // ローカルログインポート
    importLocalLogs: 'ローカルログをインポート',
    scanning: 'スキャン中...',
    files: 'ファイル',
    entries: 'エントリー',
    notFound: '見つかりません',
    existingRecords: '既存のレコード',
    noLogsFound: 'インポート可能なログファイルが見つかりません',
    clearLocalLogs: 'ローカルログをクリア',
    confirmClearLocal: 'インポートしたローカルログを全てクリアしますか？',
    clearedLocalLogs: 'クリアしたローカルログレコード',
    import: 'インポート',
    importing: 'インポート中...',
    importFailed: 'インポート失敗',
    importComplete: 'インポート完了',
    imported: 'インポート済み',
    skipped: 'スキップ（重複）',
    failedEntries: '失敗エントリー',
    // ログ保持設定
    logRetention: 'ログ保持',
    logRetentionDesc: '使用統計の保持期間を選択',
    retentionPermanent: '永久',
    retention30Days: '30日間',
    // モデル価格設定
    modelPricing: 'モデル価格',
    modelPricingDesc: 'コスト計算用にモデル価格をカスタマイズ',
    editPricing: '価格を編集',
    inputCost: '入力価格 ($/M tokens)',
    outputCost: '出力価格 ($/M tokens)',
    cacheReadCost: 'キャッシュ読取価格',
    cacheCreationCost: 'キャッシュ作成価格',
    resetPricing: 'デフォルトに戻す',
    confirmResetPricing: '全てのモデル価格をデフォルトにリセットしますか？',
    confirmDeletePricing: 'この価格設定を削除しますか？',
    input: '入力',
    output: '出力',
    // プロバイダーフィルター
    allProviders: '全て',
    noProviderData: 'このプロバイダーのデータがありません',
    // プロバイダー価格
    selectProvider: 'プロバイダーを選択',
    selectProviderHint: 'プロバイダーを選択して、その固有のモデル価格を設定してください。プロバイダー固有の価格が設定されていない場合は、以下のデフォルト価格が使用されます。',
    customPricing: '価格',
    defaultPricing: 'デフォルト価格（グローバル）',
    noCustomPricing: 'カスタム価格がありません。追加ボタンをクリックして作成してください。',
    modelId: 'モデル ID',
    selectModel: 'モデルを選択',
    // モデルトレンドチャート
    modelTokenUsage: 'モデルトークン使用量',
    dailyUsage: '日別',
    cumulativeUsage: '累計',
    moreModels: '他のモデル',
    otherModels: '他のモデル',
    total: '合計',
    // 自動インポート
    autoImport: 'ログ自動インポート',
    autoImportDesc: '使用統計ページを開く時にローカルログを自動的にインポート',
    autoImported: '{count} 件を自動インポートしました',
    // セッション統計
    conversationStats: '会話統計',
    totalConversations: '総会話数',
    totalToolCalls: 'ツール呼び出し数',
    mcpCount: 'MCP数',
    filesChanged: 'ファイル変更数',
    codeChanges: 'コード変更',
    avgResponseTime: '平均応答時間',
    avgThinkingTime: '平均思考時間',
    totalTime: '総所要時間',
    // ツール呼び出し統計
    toolCallStats: 'ツール呼び出し統計',
    moreTools: '他のツール',
    expand: '展開',
    collapse: '折りたたむ',
    toolTypes: '種類のツール',
    calls: '回の呼び出し',
    viewAll: 'すべて表示 ({count} 種類)'
  }
}
