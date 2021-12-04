MEMORY
{
    VECTOR : ORIGIN = 0x10000000, LENGTH = 4K
    ROM    : ORIGIN = 0x38000000, LENGTH = 64K
    RAM    : ORIGIN = 0x38010000, LENGTH = 64K
    STACK  : ORIGIN = 0x38020000, LENGTH = 16K
}

SECTIONS
{
    .vector_table ORIGIN(VECTOR) :
    {
        KEEP(*(.vector_table));
    } > VECTOR

    .rom ORIGIN(ROM) :
    {
        *(.text .text.*);

        /* for `-C relocation-model=pic` */
        *(.got .got.*);

        . = ALIGN(4);
        __rodata_start = .;
        *(.rodata .rodata.*);
        . = ALIGN(4);
        __rodata_end = .;
    } > ROM

    .ram ORIGIN(RAM) :
    {
        . = ALIGN(4);
        __data_start = .;
        *(.data .data.*);
        . = ALIGN(4);
        __data_end = .;

        . = ALIGN(4);
        __bss_start = .;
        *(.bss .bss.*);
        . = ALIGN(4);
        __bss_end = .;
    } > RAM

    .stack ORIGIN(STACK) :
    {
        __stack_top = .;
        . += LENGTH(STACK);
        __stack_bottom = .;
    } > STACK

    /DISCARD/ :
    {
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}

PROVIDE(__nmi         = DefaultExceptionHandler);
PROVIDE(__hardfault   = DefaultExceptionHandler);
PROVIDE(__memmanage   = DefaultExceptionHandler);
PROVIDE(__busfault    = DefaultExceptionHandler);
PROVIDE(__usagefault  = DefaultExceptionHandler);
PROVIDE(__securefault = DefaultExceptionHandler);
PROVIDE(__svc         = DefaultExceptionHandler);
PROVIDE(__debugmon    = DefaultExceptionHandler);
PROVIDE(__pendsv      = DefaultExceptionHandler);
PROVIDE(__systick     = DefaultExceptionHandler);
