!include "LogicLib.nsh"

Section -InstallWavelengthServices
    DetailPrint "Downloading and verifying Wavelength local services..."
    ExecWait '"$INSTDIR\wavelength.exe" --install-services' $0
    ${If} $0 != 0
        MessageBox MB_ICONSTOP|MB_OK "Wavelength could not download and verify its local services. Check your internet connection, then run the installer again."
        Abort
    ${EndIf}
SectionEnd
