; Instalador genérico (M10): o mesmo script e a mesma compressão para os quatro candidatos.
; makensis -DNOME=... -DPASTA=... -DEXE=... -DSAIDA=... generico.nsi
Unicode true
SetCompressor /SOLID lzma
Name "${NOME}"
OutFile "${SAIDA}"
RequestExecutionLevel user
InstallDir "$LOCALAPPDATA\Programs\${NOME}"
Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles
Section
  SetOutPath "$INSTDIR"
  File /r "${PASTA}\*.*"
  CreateShortCut "$SMPROGRAMS\${NOME}.lnk" "$INSTDIR\${EXE}"
  WriteUninstaller "$INSTDIR\desinstalar.exe"
SectionEnd
Section "Uninstall"
  Delete "$SMPROGRAMS\${NOME}.lnk"
  RMDir /r "$INSTDIR"
SectionEnd
