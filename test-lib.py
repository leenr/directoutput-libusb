import time
from pathlib import Path

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
HRESULT __stdcall DirectOutput_SetImage(void* hDevice, DWORD dwPage, DWORD dwIndex, DWORD cbValue, const void* pvValue);
HRESULT __stdcall DirectOutput_GetSerialNumber(void* hDevice, wchar_t* pszSerialNumber, DWORD dwSize);
'''.replace('HRESULT', 'int64_t').replace('DWORD', 'int32_t')
)
m = f.dlopen('./target/debug/liblibfip.so')


device_addr = None


@f.callback("void(void*, void *)")
def enumerate_callback(addr, handle):
    global device_addr
    print('enumerate_callback', addr, handle)
    device_addr = addr


m.DirectOutput_Initialize('test')
x = f.new("int *")
m.DirectOutput_Enumerate(enumerate_callback, x)
# print(m.DirectOutput_GetSerialNumber(device_addr))
# m.DirectOutput_Enumerate(enumerate_callback, x)
time.sleep(600)
m.DirectOutput_Deinitialize()
