
SECTIONS {
  /* Kernel load address for AArch64 */
  . = 0x80000;

  .text : {
    /* LONG(__stack_top); */
    KEEP(*(.text.boot));
    *(.text*)
    *(.rodata*)
  } 

  .data : {
      *(.data)
      *(.data.*)
  } 

  .bss (NOLOAD) : {
    . = ALIGN(16);
    *(.bss)
    *(.bss.*);
  }

  /DISCARD/ :
  {
    *(.debug_*);
    *(.ARM.*);
    *(.comment);
  }

}

/* All the memory from the end of bss to the top of RAM */
__heap_start = .;

/* VMA of the .data section */
__data_start = ADDR(.data); 
__data_end   = __data_start + SIZEOF(.data);

/* LMA of the .data section */
__data_load_start = LOADADDR(.data);

/* VMA of the .bss section */
__bss_start = ADDR(.bss);
__bss_end   = __bss_start + SIZEOF(.bss);

/* Entry point (for gdb) */
ENTRY(Reset_Handler);
