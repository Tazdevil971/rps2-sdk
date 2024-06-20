ENTRY(__stage0_entry);

SECTIONS {
    .text 0x00100000 : {
        __executable_start = .;
        *(.text)
        *(.text.*)
        QUAD(0)
        __etext = .;
    }
    
    .reginfo ALIGN(16): { *(.reginfo) }

    .data ALIGN(128): {
        *(.data)
        *(.data.*)
    }

    .rodata ALIGN(128): {
        *(.rodata)
        *(.rodata.*)
    }

    /* preinit/init/fini sections */
    .preinit_array : {
        __preinit_array_start = .;
        KEEP(*(.preinit_array*))
        __preinit_array_end = .;
    }

    .init_array : {
        __init_array_start = .;
        KEEP(*(SORT(.init_array.*)))
        KEEP(*(.init_array*))
        __init_array_end = .;
    }

    .fini_array : {
        __fini_array_start = .;
        KEEP(*(SORT(.fini_array.*)))
        KEEP(*(.fini_array*))
        __fini_array_end = .;
    }

    /* EH frame data, right after read-only data */
    .eh_frame_hdr : { 
        __GNU_EH_FRAME_HDR = .;
        KEEP(*(.eh_frame_hdr)) 
    }
    .eh_frame : { KEEP(*(.eh_frame)) }

    /* Start of small data section */
    .sdata ALIGN(128): {
        *(.sdata)
        *(.sdata.*)
        __edata = .;
    }

    .sbss ALIGN(128) : {
        __BSS_START = .;
        *(.sbss)
        *(.sbss.*)
    }

    /* End of small data section */
    
    .bss ALIGN(128) : {
        __GP = .;
        *(.bss)
        *(.bss.*)
    }

    . = ALIGN(128);
    __BSS_END = .;
    __end = .;
    __HEAP_START = .;

    /* Unwanted stuff */
    /DISCARD/ : {
        *(.MIPS.abiflags)
    }
}