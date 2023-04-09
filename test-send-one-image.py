import sys
import threading
import time
from itertools import chain

import usb.core
import usb.util
from PIL import Image


def input_thread_target(endpoint_in):
    while True:
        try:
            data = endpoint_in.read(44, timeout=0)
            print('in:', data)
        except usb.core.USBError:
            raise


def main():
    while True:
        device = usb.core.find(idVendor=0x06a3, idProduct=0xa2ae)
        if device is not None:
            break
        print('\bDevice not found', end='')
        sys.stdout.flush()
        time.sleep(0.5)

    interface = usb.util.find_descriptor(
        device[0],
        bInterfaceClass=0xff,  # vendor specific
        bDescriptorType=usb.util.DESC_TYPE_INTERFACE
    )
    if device.is_kernel_driver_active(interface.index):
        device.detach_kernel_driver(interface.index)
    usb.util.claim_interface(device, interface.index)

    endpoint_in = usb.util.find_descriptor(
        interface,
        custom_match=lambda x: (
            usb.util.endpoint_direction(x.bEndpointAddress) == usb.util.ENDPOINT_IN
        )
    )
    endpoint_out = usb.util.find_descriptor(
        interface,
        custom_match=lambda x: (
            usb.util.endpoint_direction(x.bEndpointAddress) == usb.util.ENDPOINT_OUT
        )
    )

    t = threading.Thread(target=input_thread_target, args=(endpoint_in,), daemon=True)
    t.start()

    while True:
        # endpoint_out.clear_halt()
        b = bytearray(44)
        b[ 3], b[ 2], b[ 1], b[ 0] = (0x00, 0x00, 0x00, 0x00)[::1]  #  0: (0) ??
        b[ 7], b[ 6], b[ 5], b[ 4] = (0x00, 0x00, 0x00, 0x00)[::1]  #  1: (1) ??
        b[11], b[10], b[ 9], b[ 8] = (0x00, 0x00, 0x00, 0x00)[::1]  #  2: (0)
        b[15], b[14], b[13], b[12] = (0x00, 0x00, 0x00, 0x00)[::1]  #  3: (0) ?? fixed
        b[19], b[18], b[17], b[16] = (0x00, 0x00, 0x00, 0x00)[::1]  #  4: (0) ?? fixed
        b[23], b[22], b[21], b[20] = (0x0a, 0x00, 0x00, 0x00)[::1]  #  5: (6) ??
        b[27], b[26], b[25], b[24] = (0x00, 0x00, 0x00, 0x00)[::1]  #  6: (0) ??
        b[31], b[30], b[29], b[28] = (0x00, 0x00, 0x00, 0x00)[::1]  #  7: (0) ??
        b[35], b[34], b[33], b[32] = (0x00, 0x00, 0x00, 0x00)[::1]  #  8: (0) ??
        b[39], b[38], b[37], b[36] = (0x00, 0x00, 0x00, 0x00)[::1]  #  9: (0) ??
        b[43], b[42], b[41], b[40] = (0x00, 0x00, 0x00, 0x00)[::1]  # 10: (0) ??
        try:
            endpoint_out.write(b, timeout=5000)
            print('out:', b)
        except usb.core.USBTimeoutError:
            # device.reset()
            continue
        else:
            break
    print('!!!!')

    b = bytearray(44)
    b[ 3], b[ 2], b[ 1], b[ 0] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  0: (0) ??
    b[ 7], b[ 6], b[ 5], b[ 4] = (0x00, 0x00, 0x00, 0x01)[::-1]  #  1: (1) ??
    b[11], b[10], b[ 9], b[ 8] = (0x00, 0x03, 0x84, 0x00)[::-1]  #  2: (320 * 240 * 3 bytes)
    b[15], b[14], b[13], b[12] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  3: (0) ?? fixed
    b[19], b[18], b[17], b[16] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  4: (0) ?? fixed
    b[23], b[22], b[21], b[20] = (0x00, 0x00, 0x00, 0x06)[::-1]  #  5: (6) ??
    b[27], b[26], b[25], b[24] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  6: (0) ??
    b[31], b[30], b[29], b[28] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  7: (0) ??
    b[35], b[34], b[33], b[32] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  8: (0) ??
    b[39], b[38], b[37], b[36] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  9: (0) ??
    b[43], b[42], b[41], b[40] = (0x00, 0x00, 0x00, 0x00)[::-1]  # 10: (0) ??
    image_pixels = chain.from_iterable(Image.open('SMPTEColor.png').resize((320, 240)).getdata())
    endpoint_out.write(b, timeout=5000)
    print('out:', b)
    endpoint_out.write(bytes(image_pixels), timeout=5000)
    print('out:', '[image data]')

    # b = bytearray(44)
    # b[ 3], b[ 2], b[ 1], b[ 0] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  0: (0) ??
    # b[ 7], b[ 6], b[ 5], b[ 4] = (0x00, 0x00, 0x00, 0x01)[::-1]  #  1: (1) ??
    # b[11], b[10], b[ 9], b[ 8] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  2: (0)
    # b[15], b[14], b[13], b[12] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  3: (0) ?? fixed
    # b[19], b[18], b[17], b[16] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  4: (0) ?? fixed
    # b[23], b[22], b[21], b[20] = (0x13, 0x00, 0x00, 0x00)[::-1]  #  5: (6) ??
    # b[27], b[26], b[25], b[24] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  6: (0) ??
    # b[31], b[30], b[29], b[28] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  7: (0) ??
    # b[35], b[34], b[33], b[32] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  8: (0) ??
    # b[39], b[38], b[37], b[36] = (0x00, 0x00, 0x00, 0x00)[::-1]  #  9: (0) ??
    # b[43], b[42], b[41], b[40] = (0x00, 0x00, 0x00, 0x00)[::-1]  # 10: (0) ??
    # endpoint_out.write(b)


if __name__ == '__main__':
    main()
