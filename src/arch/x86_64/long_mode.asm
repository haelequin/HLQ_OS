global long_mode_start
extern rust_main

section .text
bits 64
long_mode_start:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; print `LONG BOOT` to screen
    ;'LONG'
    mov rax, 0x0247024e024f024c
    mov qword [0xb80a0], rax
    ;Space
    mov dword [0xb80a8], 0x0220
    ;'BOOT'
    mov rax, 0x0254024f024f0242
    mov qword [0xb80aa], rax

    cli

    call rust_main
    hlt