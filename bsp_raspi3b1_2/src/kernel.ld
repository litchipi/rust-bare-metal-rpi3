__executable_load_addr = 0x80000;

ENTRY(__executable_load_addr)

/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 */
PHDRS
{
    segment_code PT_LOAD FLAGS(5);
}

SECTIONS
{
    . =  __executable_load_addr;

    .text :
    {
        KEEP(*(.text._start))
    } :segment_code
}
