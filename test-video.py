import time
from pathlib import Path

from PIL import Image
from cffi import FFI


f = FFI()
#f.cdef(Path('./target/libfip.h').read_text())
f.cdef(
'''
typedef void (__stdcall *Pfn_DirectOutput_EnumerateCallback)(void* hDevice, void* pCtxt);
typedef void (__stdcall *Pfn_DirectOutput_DeviceChange)(void* hDevice, bool bAdded, void* pCtxt);
typedef void (__stdcall *Pfn_DirectOutput_PageChange)(void* hDevice, DWORD dwPage, bool bSetActive, void* pCtxt);
typedef void (__stdcall *Pfn_DirectOutput_SoftButtonChange)(void* hDevice, DWORD dwButtons, void* pCtxt);

HRESULT __stdcall DirectOutput_Initialize(const wchar_t* wszPluginName);
HRESULT __stdcall DirectOutput_Deinitialize();
HRESULT __stdcall DirectOutput_RegisterDeviceCallback(Pfn_DirectOutput_DeviceChange pfnCb, void* pCtxt);
HRESULT __stdcall DirectOutput_Enumerate(Pfn_DirectOutput_EnumerateCallback pfnCb, void* pCtxt);
HRESULT __stdcall DirectOutput_RegisterPageCallback(void* hDevice, Pfn_DirectOutput_PageChange pfnCb, void* pCtxt);
HRESULT __stdcall DirectOutput_RegisterSoftButtonCallback(void* hDevice, Pfn_DirectOutput_SoftButtonChange pfnCb, void* pCtxt);
HRESULT __stdcall DirectOutput_SetLed(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD dwValue);
HRESULT __stdcall DirectOutput_SetImage(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD cbValue, const void* pvValue);
HRESULT __stdcall DirectOutput_GetSerialNumber(void* hDevice, wchar_t* pszSerialNumber, DWORD dwSize);
'''.replace('HRESULT', 'int64_t').replace('DWORD', 'int32_t')
)
m = f.dlopen('./liblibfip.so')


import av
import queue
import sys
import threading
from itertools import chain


device_addr = None


@f.callback("void(void*, void *)")
def enumerate_callback(addr, handle):
    global device_addr
    print('enumerate_callback', addr, handle)
    device_addr = addr


def thread_target(q):
    while True:
        image_pixels = q.get()
        m.DirectOutput_SetImage(device_addr, 0, 0, len(image_pixels), image_pixels)


m.DirectOutput_Initialize('test')
try:
    x = f.new("int *")
    import time; time.sleep(0.5)
    m.DirectOutput_Enumerate(enumerate_callback, x)
    if device_addr is None:
        print('No devices found!')
        exit(1)

    container = av.open(sys.argv[1])

    q = queue.Queue(32)
    threading.Thread(target=thread_target, args=(q,), daemon=True).start()
    for packet in container.demux():
        for i, frame in enumerate(packet.decode()):
            if isinstance(frame, av.video.frame.VideoFrame):
                img = frame.to_image().resize((320, 240))
                image_pixels = bytes(chain.from_iterable(img.getdata()))[::-1]
                q.put(image_pixels)
finally:
    m.DirectOutput_Deinitialize()
