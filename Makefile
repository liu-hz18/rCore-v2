# 生成内核镜像 所需命令
TARGET      := riscv64gc-unknown-none-elf
MODE        := debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/os
BIN_FILE    := target/$(TARGET)/$(MODE)/kernel.bin

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

USER_DIR    := ../user
USER_BUILD  := $(USER_DIR)/build
IMG_FILE    := $(USER_BUILD)/disk.img

.PHONY: doc kernel build clean qemu run

# 默认 build 为输出二进制文件
build: $(BIN_FILE)

# 通过 Rust 文件中的注释生成文档
doc:
	@cargo doc --document-private-items

# 编译 kernel
kernel:
	@cargo build

# 生成 kernel 的二进制文件
$(BIN_FILE): kernel
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $@

# 查看反汇编结果
asm:
	@$(OBJDUMP) -d $(KERNEL_FILE) | less

# 清理编译出的文件
clean:
	@cargo clean

# 运行 QEMU
# 为了让 QEMU 挂载上我们虚拟的存储设备，我们这里选了 QEMU 支持的 virtio 协议，需要在 QEMU 运行的时候加入选项
qemu: build
	@qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios default \
    		-device loader,file=$(BIN_FILE),addr=0x80200000 \
    		-drive file=$(IMG_FILE),format=qcow2,id=sfs \
    		-device virtio-blk-device,drive=sfs 
# 模拟存储设备
# 以 virtio Block Device 的形式挂载到 virtio 总线上

# 一键运行
run: build qemu
