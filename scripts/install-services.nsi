!include "LogicLib.nsh"

Section -InstallWavelengthServices
    GetDLLVersion "$INSTDIR\wavelength.exe" $R0 $R1
    IntOp $R2 $R0 >> 16
    IntOp $R3 $R0 & 0xFFFF
    IntOp $R4 $R1 >> 16
    StrCpy $R5 "$R2.$R3.$R4"

    IfFileExists "$EXEDIR\Wavelength_$R5_services_windows-x86_64.zip" installVersionedLocalServices checkGenericLocalServices

checkGenericLocalServices:
    IfFileExists "$EXEDIR\Wavelength_services_windows-x86_64.zip" installGenericLocalServices downloadServices

installVersionedLocalServices:
    StrCpy $R6 "$EXEDIR\Wavelength_$R5_services_windows-x86_64.zip"
    Goto installLocalServices

installGenericLocalServices:
    StrCpy $R6 "$EXEDIR\Wavelength_services_windows-x86_64.zip"

installLocalServices:
    DetailPrint "Installing Wavelength local services from the package beside the installer..."
    ExecWait '"$INSTDIR\wavelength.exe" --install-services --archive "$R6"' $0
    Goto servicesInstalled
downloadServices:
    DetailPrint "Downloading and installing Wavelength local services..."
    ExecWait '"$INSTDIR\wavelength.exe" --install-services' $0
servicesInstalled:
    ${If} $0 != 0
        MessageBox MB_ICONSTOP|MB_OK "Wavelength could not install or verify its local services. Check the service package and your internet connection, then try the installer again."
        Abort
    ${EndIf}
SectionEnd
