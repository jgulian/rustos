ENTRY(_start)

SECTIONS
{
  . = 0xffffffffc0000000;

  .text : {
    *(.text .text.*)
  }

  .rodata : {
    . = ALIGN(16);
    *(.rodata .rodata.*)
  }

  .data : {
    . = ALIGN(16);
    *(.data .data.*)
  }

  .bss (NOLOAD) : {
    . = ALIGN(32);
    __bss_beg = .;
    *(.bss .bss.*)
    __bss_end = .;
    . = ALIGN(32);
  }
}
