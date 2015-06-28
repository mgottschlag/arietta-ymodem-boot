Arietta G25 YMODEM bootloader
=============================

This repository contains a bootloader for the Arietta G25 board which supports loading the OS image via the YMODEM protocol via the debug port.
Such a bootloader is useful for OS development when you do not want to swap SD cards to copy the OS all the time.
Instead, minicom or any YMODEM capable terminal can be used to upload the OS.

The bootloader only supports uncompressed images in the U-Boot image format (uImage).
It loads itself to 0x23000000 and loads the incoming image to 0x22000000 (before relocating).
It unpacks the image to whatever address specified in the U-Boot header and calls the entry point.
Make sure that you do not specify the two addresses above as your image destination, or the bootloader might fail.

Warning: The code has not been tested much, does not implement the whole YMODEM standard and will likely fail with anything but minicom.

Compiling
---------

To compile the bootloader, you need somewhat up-to date rustc and cargo, an ARM GCC toolchain (the scripts assume that the prefix is "arm-none-eabi-") and the U-Boot image tools.
When all dependencies are available, the code can be compiled by executing "./compile.sh".

Installation
------------

The bootloader is compiled to a uImage which is placed in the root directory of the repository.
This uImage can be loaded for example by AT91Bootstrap configured to load an uImage file directly.
Both AT91Bootstrap and this bootloader need to be placed on the kernel partition of the SD card.

Usage
-----

When the installation is correct, the output on the debug port should look like this:

    RomBOOT


    AT91Bootstrap 3.7-00025-g3f957ce (So 14. Jun 09:59:14 CEST 2015)

    1-Wire: Loading 1-Wire information ...
    1-Wire: ROM Searching ... Done, 0x0 1-Wire chips found

    WARNING: 1-Wire: No 1-Wire chip found
 
    1-Wire: Using defalt information

    1-Wire: SYS_GPBR2: 0x4010425, SYS_GPBR3: 0x8421

    SD/MMC: Image: Read file uImage to 0x22000000
    Cmd: 0x8 Response Time-out
    SD: Card Capacity: Standard
    SD: Specification Version 1.0 and 1.01
    SD/MMC: dt blob: Read file acme-arietta.dtb to 0x21000000
    Cmd: 0x8 Response Time-out
    SD: Card Capacity: Standard
    SD: Specification Version 1.0 and 1.01

    Booting uImage ......
    uImage magic: 0x27051956 is found
    Relocating kernel image, dest: 0x23000000, src: 0x22000040
     ...... 0x46ac bytes data transferred

    Using device tree in place at 0x21000000

    Starting linux kernel ..., machid: 0xffffffff

     ymodem bootloader
    ===================

    downloading uImage to 0x22000000...
    CCC

When the code starts printing "CCC...", you can use YMODEM to upload a uImage.

