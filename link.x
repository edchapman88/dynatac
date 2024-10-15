
SECTIONS {
  /* Kernel load address for the Raspberry Pi 4 B. */
  . = 0x80000;
  /* Set the top of the stack at the load address. The stack will grow downwards towards 0x0. */
  __stack_top = .;
  /* The following sections are loaded into the memory after 0x80000 (growing towards 0xFFF...). */ 
  .text : ALIGN(4096) {
    /* Keep the .text.boot section at the start of .text. This section with contain the program entry point. */
    KEEP(*(.text.boot));
    *(.text)
    *(.text*)
    *(.rodata)
    *(.rodata*)
  } 

  .data : ALIGN(4096) {
    *(.data)
    *(.data.*)
  } 

  .bss : ALIGN(4096) {
    __bss_start = .;
    bss = .;
    *(.bss)
    *(.bss.*);
    __bss_end = .;
  }

  /DISCARD/ :
  {
    *(.debug_*);
    *(.ARM.*);
    *(.comment);
  }

}

__bss_size = (__bss_end - __bss_start)>>3;
