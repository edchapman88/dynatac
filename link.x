
SECTIONS {
  /* Kernel load address for the Raspberry Pi 4 B. */
  . = 0x80000;
  /* Set the top of the stack at the load address. The stack will grow downwards towards 0x0. */
  __stack_top = .;
  /* The following sections are loaded into the memory after 0x80000 (growing towards 0xFFF...). */ 
  .text : {
    /* Keep the .text.boot section at the start of .text. This section with contain the program entry point. */
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
