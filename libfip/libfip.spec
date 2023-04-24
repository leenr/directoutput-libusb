1 stdcall -ret64 DirectOutput_Initialize (wstr) ProxyDirectOutput_Initialize
2 stdcall -ret64 DirectOutput_Deinitialize () ProxyDirectOutput_Deinitialize
3 stdcall -ret64 DirectOutput_RegisterDeviceCallback (ptr ptr) ProxyDirectOutput_RegisterDeviceCallback
4 stdcall -ret64 DirectOutput_Enumerate (ptr ptr) ProxyDirectOutput_Enumerate
5 stdcall -ret64 DirectOutput_RegisterPageCallback (ptr ptr ptr) ProxyDirectOutput_RegisterPageCallback
6 stdcall -ret64 DirectOutput_RegisterSoftButtonCallback (ptr ptr ptr) ProxyDirectOutput_RegisterSoftButtonCallback
@ stdcall -ret64 DirectOutput_GetDeviceType (ptr ptr) ProxyDirectOutput_GetDeviceType
@ stdcall -ret64 DirectOutput_GetDeviceInstance (ptr ptr) ProxyDirectOutput_GetDeviceInstance
@ stdcall -ret64 DirectOutput_SetProfile (ptr long wstr) ProxyDirectOutput_SetProfile
@ stdcall -ret64 DirectOutput_AddPage (ptr long wstr long) ProxyDirectOutput_AddPage
@ stdcall -ret64 DirectOutput_RemovePage (ptr long) ProxyDirectOutput_RemovePage
@ stdcall -ret64 DirectOutput_SetLed (ptr long long long) ProxyDirectOutput_SetLed
@ stdcall -ret64 DirectOutput_SetString (ptr long long long wstr) ProxyDirectOutput_SetString
@ stdcall -ret64 DirectOutput_SetImage (ptr long long long ptr) ProxyDirectOutput_SetImage
@ stdcall -ret64 DirectOutput_SetImageFromFile (ptr long long long wstr) ProxyDirectOutput_SetImageFromFile
@ stdcall -ret64 DirectOutput_StartServer (ptr long wstr ptr ptr) ProxyDirectOutput_StartServer
@ stdcall -ret64 DirectOutput_CloseServer (ptr long ptr) ProxyDirectOutput_CloseServer
@ stdcall -ret64 DirectOutput_SendServerMsg (ptr long long long long ptr long ptr ptr) ProxyDirectOutput_SendServerMsg
@ stdcall -ret64 DirectOutput_SendServerFile (ptr long long long long ptr long wstr long ptr ptr) ProxyDirectOutput_SendServerFile
@ stdcall -ret64 DirectOutput_SaveFile (ptr long long long wstr ptr) ProxyDirectOutput_SaveFile
@ stdcall -ret64 DirectOutput_DisplayFile (ptr long long long ptr) ProxyDirectOutput_DisplayFile
@ stdcall -ret64 DirectOutput_DeleteFile (ptr long long ptr) ProxyDirectOutput_DeleteFile
@ stdcall -ret64 DirectOutput_GetSerialNumber (ptr ptr long) ProxyDirectOutput_GetSerialNumber
