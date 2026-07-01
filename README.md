# HLQ-OS (x86_64 Bare-Metal Kernel)

A custom 64-bit operating system kernel built using Rust and Assembly.

---

## Prerequisites

This project requires a native **Linux (Ubuntu)** environment or **Windows Subsystem for Linux (WSL)**. 

### 1. Install System Dependencies
Install the required compilation tools, assembler, ISO creation utilities, and the QEMU emulator:

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    nasm \
    binutils \
    qemu-system-x86_64 \
    grub-common \
    grub-pc-bin \
    xorriso
