import importlib.util
import os
import sys
import threading
import time
from contextlib import asynccontextmanager
from pathlib import Path

import uvicorn
from fastapi import FastAPI


if getattr(sys, "frozen", False):
    bundle_root = Path(sys._MEIPASS)
    audio_root = bundle_root / "audio"
    qobuz_root = bundle_root / "qobuz"
else:
    repository_root = Path(__file__).resolve().parents[1]
    audio_root = repository_root / "audio-service"
    qobuz_root = repository_root / "qobuz-rest-api"

sys.path.insert(0, str(audio_root))


def load_service_module(name, source_path):
    spec = importlib.util.spec_from_file_location(name, source_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Could not load service module: {source_path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def configure_background_chromium():
    from pydoll.browser.chromium import Chrome
    from SpotiFLAC.core import signed_session_mono
    from SpotiFLAC.core.solver import build_chromium_options

    def hidden_options(**_kwargs):
        return build_chromium_options(hidden=True)

    signed_session_mono.build_chromium_options = hidden_options
    original_start = Chrome.start

    async def start_without_taskbar_entry(browser, *args, **kwargs):
        tab = await original_start(browser, *args, **kwargs)
        process = getattr(browser._browser_process_manager, "_process", None)
        if os.name == "nt" and process:
            threading.Thread(
                target=hide_browser_taskbar_window,
                args=(process.pid,),
                daemon=True,
            ).start()
        return tab

    Chrome.start = start_without_taskbar_entry


def hide_browser_taskbar_window(process_id):
    if os.name != "nt":
        return

    import ctypes
    from ctypes import wintypes

    user32 = ctypes.WinDLL("user32", use_last_error=True)
    enum_windows_proc = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    get_window_thread_process_id = user32.GetWindowThreadProcessId
    get_window_thread_process_id.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    get_window_long_ptr = getattr(user32, "GetWindowLongPtrW", user32.GetWindowLongW)
    set_window_long_ptr = getattr(user32, "SetWindowLongPtrW", user32.SetWindowLongW)
    get_window_long_ptr.argtypes = [wintypes.HWND, ctypes.c_int]
    get_window_long_ptr.restype = ctypes.c_ssize_t
    set_window_long_ptr.argtypes = [wintypes.HWND, ctypes.c_int, ctypes.c_ssize_t]
    set_window_long_ptr.restype = ctypes.c_ssize_t
    set_window_pos = user32.SetWindowPos
    set_window_pos.argtypes = [
        wintypes.HWND,
        wintypes.HWND,
        ctypes.c_int,
        ctypes.c_int,
        ctypes.c_int,
        ctypes.c_int,
        wintypes.UINT,
    ]
    set_window_pos.restype = wintypes.BOOL

    gwl_exstyle = -20
    ws_ex_toolwindow = 0x00000080
    ws_ex_appwindow = 0x00040000
    swp_nosize = 0x0001
    swp_nomove = 0x0002
    swp_nozorder = 0x0004
    swp_noactivate = 0x0010
    swp_framechanged = 0x0020
    deadline = time.monotonic() + 15

    while time.monotonic() < deadline:
        found = False

        @enum_windows_proc
        def update_window(hwnd, _lparam):
            nonlocal found
            owner_process_id = wintypes.DWORD()
            get_window_thread_process_id(hwnd, ctypes.byref(owner_process_id))
            if owner_process_id.value != process_id:
                return True
            style = get_window_long_ptr(hwnd, gwl_exstyle)
            hidden_style = (style & ~ws_ex_appwindow) | ws_ex_toolwindow
            if style != hidden_style:
                set_window_long_ptr(hwnd, gwl_exstyle, hidden_style)
                set_window_pos(
                    hwnd,
                    None,
                    0,
                    0,
                    0,
                    0,
                    swp_nosize | swp_nomove | swp_nozorder | swp_noactivate | swp_framechanged,
                )
            found = True
            return True

        user32.EnumWindows(update_window, 0)
        if found:
            return
        time.sleep(0.25)


configure_background_chromium()
audio_service = load_service_module("wavelength_audio_service", audio_root / "main.py")
qobuz_service = load_service_module("wavelength_qobuz_service", qobuz_root / "main.py")


@asynccontextmanager
async def lifespan(application):
    async with qobuz_service.app.router.lifespan_context(qobuz_service.app):
        async with audio_service.app.router.lifespan_context(audio_service.app):
            yield


app = FastAPI(lifespan=lifespan)
app.mount("/qobuz", qobuz_service.app)
app.mount("/", audio_service.app)


if __name__ == "__main__":
    uvicorn.run(
        app,
        host="127.0.0.1",
        port=int(os.getenv("PORT", "8787")),
    )