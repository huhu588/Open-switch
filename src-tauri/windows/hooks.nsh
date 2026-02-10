; Ai Switch NSIS Installer Hooks
; 用于在安装新版本前自动卸载旧版 "Open Switch"
;
; 关键修复：NSIS 卸载器默认会将自身复制到 $TEMP 后重新启动，
; 导致 ExecWait 提前返回、安装流程中断。
; 使用 _?=<dir> 参数强制卸载器在原地运行，确保 ExecWait 等待完成。

!macro NSIS_HOOK_PREINSTALL
  ; === 检查当前用户级别（HKCU）的旧版安装 ===
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "UninstallString"
  ReadRegStr $1 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "InstallLocation"

  ${If} $0 != ""
    DetailPrint "检测到旧版 Open Switch，正在卸载..."

    ; 从 UninstallString 中提取卸载器路径（去掉引号）
    StrCpy $2 $0 "" 1
    StrLen $3 $2
    IntOp $3 $3 - 1
    StrCpy $2 $2 $3

    ; 使用 _?= 参数防止卸载器复制到临时目录后重启
    ; 这样 ExecWait 才能正确等待卸载完成
    ${If} $1 != ""
      ExecWait '"$2" /S _?=$1' $4
    ${Else}
      ExecWait '"$2" /S' $4
    ${EndIf}

    DetailPrint "旧版 Open Switch 卸载返回代码: $4"

    ; 使用 _?= 后卸载器不会自删除，手动清理
    ${If} $1 != ""
      Delete "$1\uninstall.exe"
      RMDir "$1"
    ${EndIf}

    ; 清理旧版快捷方式
    Delete "$DESKTOP\Open Switch.lnk"
    Delete "$SMPROGRAMS\Open Switch.lnk"
  ${EndIf}

  ; === 检查系统级别（HKLM）的旧版安装 ===
  ReadRegStr $0 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "UninstallString"
  ReadRegStr $1 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "InstallLocation"

  ${If} $0 != ""
    DetailPrint "检测到旧版 Open Switch（系统级），正在卸载..."

    StrCpy $2 $0 "" 1
    StrLen $3 $2
    IntOp $3 $3 - 1
    StrCpy $2 $2 $3

    ${If} $1 != ""
      ExecWait '"$2" /S _?=$1' $4
    ${Else}
      ExecWait '"$2" /S' $4
    ${EndIf}

    DetailPrint "旧版 Open Switch（系统级）卸载返回代码: $4"

    ${If} $1 != ""
      Delete "$1\uninstall.exe"
      RMDir "$1"
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; 安装完成后无额外操作
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 卸载前无额外操作
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; 卸载后无额外操作
!macroend
