!include "LogicLib.nsh"

Function .onInit
    FindWindow $R0 "" "Wavelength"
    ${If} $R0 != 0
        MessageBox MB_YESNO|MB_ICONEXCLAMATION "Wavelength is open. If you choose Yes, it will be closed forcefully before installation. Choose No if you have already closed it yourself." IDYES closeWavelengthAutomatically

    confirmWavelengthClosed:
        FindWindow $R0 "" "Wavelength"
        ${If} $R0 != 0
            MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION "Wavelength is still open. Close it and select OK to continue, or select Cancel to exit setup." IDOK confirmWavelengthClosed IDCANCEL cancelInstallation
        ${EndIf}
        Goto stopLocalService

    closeWavelengthAutomatically:
        nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /IM wavelength.exe /T /F'
        Pop $R1
        Sleep 750
        FindWindow $R0 "" "Wavelength"
        ${If} $R0 != 0
            MessageBox MB_ICONSTOP|MB_OK "Wavelength could not be closed. Close it manually and run setup again."
            Abort
        ${EndIf}

    stopLocalService:
    ${EndIf}
    nsExec::ExecToLog '"$SYSDIR\taskkill.exe" /IM WavelengthLocalService.exe /T /F'
    Pop $R1
    Sleep 750
    Return

    cancelInstallation:
        Abort
FunctionEnd

Section -InstallWavelengthServices
    GetDLLVersion "$INSTDIR\wavelength.exe" $R0 $R1
    IntOp $R2 $R0 >> 16
    IntOp $R3 $R0 & 0xFFFF
    IntOp $R4 $R1 >> 16
    StrCpy $R5 "$R2.$R3.$R4"

    StrCpy $R6 "$EXEDIR\Wavelength_$R5_services_windows-x86_64.zip"
    IfFileExists "$R6" installLocalServices downloadServices

installLocalServices:
    DetailPrint "Installing Wavelength local services from the package beside the installer..."
    ExecWait '"$INSTDIR\wavelength.exe" --install-services --archive "$R6"' $0
    Goto servicesInstalled

downloadServices:
    DetailPrint "Downloading Wavelength local services from the matching GitHub release..."
    ExecWait '"$INSTDIR\wavelength.exe" --install-services' $0

servicesInstalled:
    ${If} $0 != 0
        MessageBox MB_ICONSTOP|MB_OK "Wavelength could not install or verify its local services. Place the matching services ZIP beside the installer or check your internet connection, then retry."
        Abort
    ${EndIf}
SectionEnd
