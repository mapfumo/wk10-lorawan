/* STM32WL55JC Memory Layout */
/* Cortex-M4 core */

MEMORY
{
  /* Flash: 256KB total (0x40000 bytes) */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K

  /* RAM: 64KB total (0x10000 bytes) */
  /* Note: STM32WL has RAM1 (32KB) and RAM2 (32KB) */
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

/* Entry point */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
