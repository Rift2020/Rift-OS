# Rift-OS

Rift-OS是[**2023全国大学生计算机系统能力大赛操作系统设计赛-内核实现赛**](https://os.educg.net/#/oldDetail?name=2023%E5%85%A8%E5%9B%BD%E5%A4%A7%E5%AD%A6%E7%94%9F%E8%AE%A1%E7%AE%97%E6%9C%BA%E7%B3%BB%E7%BB%9F%E8%83%BD%E5%8A%9B%E5%A4%A7%E8%B5%9B%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E8%AE%BE%E8%AE%A1%E8%B5%9B-%E5%86%85%E6%A0%B8%E5%AE%9E%E7%8E%B0%E8%B5%9B)的参赛作品，获得了初赛杰出奖和决赛优胜奖，也是出于好玩和学习的目的进行开发的操作系统。

- 100% Rust
- 从零编写
- ~~多核~~
- RISC-V 64

## 运行

#### 安装依赖

- Rust版本nightly-2023-03-29
- QEMU7.0或更新(qemu-system-riscv64)

```shell
rustup target add riscv64imac-unknown-none-elf
rustup default nightly-2023-03-29; rustup component add llvm-tools-preview;  rustup target add riscv64imac-unknown-none-elf; cargo install cargo-binutils; rustup target add riscv64imac-unknown-none-elf --toolchain nightly-2023-03-29; rustup component add llvm-tools-preview --toolchain nightly-2023-03-29
```

仓库中附带了一个example镜像文件，你可以直接：

```shell
make qemu
```

来启动Rift-OS，Rift-OS在完成初始化后会运行该镜像文件中的简易sh，你可以运行`lsroot`来查看有哪些命令行程序可以使用。

## 完成度

(对于比赛要求的系统调用而言)



|                                                              | (基本)完整的实现功能(无论性能) | 不完整的实现 | 可以算是作弊的实现(骗分) | 没有实现 | 备注 |
| :----------------------------------------------------------: | :----------------------------: | :----------: | :----------------------: | :------: | ---- |
|                     **文件系统syscal**l                      |                                |              |                          |          |      |
|                          sys_getcwd                          |            &#10004;            |              |                          |          |      |
|                          sys_pipe2                           |                                |              |                          | &#10004; |      |
|                           sys_dup                            |            &#10004;            |              |                          |          |      |
|                           sys_dup3                           |            &#10004;            |              |                          |          |      |
|                          sys_chdir                           |            &#10004;            |              |                          |          |      |
|                          sys_openat                          |                                |   &#10004;   |                          |          |      |
|                          sys_close                           |            &#10004;            |              |                          |          |      |
|                        sys_getdents64                        |                                |              |                          | &#10004; |      |
|                           sys_read                           |            &#10004;            |              |                          |          |      |
|                          sys_write                           |            &#10004;            |              |                          |          |      |
|                          sys_linkat                          |                                |              |                          | &#10004; |      |
|                         sys_unlinkat                         |                                |              |                          | &#10004; |      |
|                         sys_mkdirat                          |            &#10004;            |              |                          |          |      |
|                         sys_umount2                          |                                |              |         &#10004;         |          |      |
|                          sys_mount                           |                                |              |         &#10004;         |          |      |
|                          sys_fstat                           |                                |              |                          | &#10004; |      |
|                     **进程管理syscall**                      |                                |              |                          |          |      |
|                          sys_clone                           |                                |   &#10004;   |                          |          |      |
|                          sys_execve                          |            &#10004;            |              |                          |          |      |
|                           sys_wait                           |            &#10004;            |              |                          |          |      |
|                           sys_exit                           |            &#10004;            |              |                          |          |      |
|                         sys_getppid                          |            &#10004;            |              |                          |          |      |
|                          sys_getpid                          |            &#10004;            |              |                          |          |      |
|                           sys_brk                            |                                |              |         &#10004;         |          |      |
|                          sys_munmap                          |                                |              |                          | &#10004; |      |
|                           sys_mmap                           |                                |              |                          | &#10004; |      |
|                       **其他syscall**                        |                                |              |                          |          |      |
|                          sys_times                           |            &#10004;            |              |                          |          |      |
|                          sys_uname                           |            &#10004;            |              |                          |          |      |
|                       sys_sched_yield                        |            &#10004;            |              |                          |          |      |
|                       sys_gettimeofday                       |            &#10004;            |              |                          |          |      |
|                        sys_nanosleep                         |            &#10004;            |              |                          |          |      |
| sys_getline(非比赛要求，为了支持命令行交互的最简单的实现方式) |                                |              |                          |          |      |
|                    SYS_LSROOT(非比赛要求)                    |                                |              |                          |          |      |

实现了的内容与模块(斜体意味着主要工作是调库完成的)

- 内存管理（页帧分配，堆分配，页表映射管理，地址转换）
- 线程管理，切换，调度
- *文件系统下层*
- 文件系统上层（赶工以至于很糙，不完善
- *块设备驱动*
- *elf文件解析*
- 中断以及Trap的处理
- 少量系统调用
- ~~多核启动~~
- 与opensbi交互
- example镜像里的用户态程序(粗糙，检查不完整)

没有实现的功能

- 信号量
- 动态链接
- 睡眠锁与唤醒
- 进程间通信
- 虚拟文件系统
- 网络，图形界面......

## TODO

- 支持更多的系统调用，以使得busybox sh可以运行
- 文件系统重构
- 调整修改一些比赛期间赶工的代码（尤其是文件系统，系统调用参数检查等重灾区）
- 增加一些用户态程序初始化的内容
- 修复bug
- ...

### Bug

1. virtio driver 初始化大概有不到2%的概率直接崩溃
2. Rift-OS在以调试模式构建的情况下的特有bug

### 依赖

依赖库为我省去了很多工作，感谢他们的开源工作

- [riscv](https://docs.rs/riscv/latest/riscv/)

- [spin](https://docs.rs/spin/0.9.8/spin/)
- [lazy_static](https://docs.rs/lazy_static/1.4.0/lazy_static/)

- [bitflags](https://docs.rs/bitflags/2.3.1/bitflags/)
- [virtio-drivers](https://docs.rs/virtio-drivers/0.4.0/virtio_drivers/) 轻微修改
- [fat32](https://docs.rs/fat32/0.2.0/fat32/) 有修改
- [xmas-elf](https://docs.rs/xmas-elf/0.9.0/xmas_elf/)

- [buddy-system-allocator](https://github.com/rcore-os/buddy_system_allocator) 有修改 位于kernel/src/memory/allocator/

### 致谢

- 非常感谢mit的6.s081，它让我第一次一窥操作系统内核，感受到编写内核的挑战性和趣味
- 感谢rcore-tutorial，Rift-OS最初的开发很大程度上参考了rcore
- 感谢南京大学jyy，计组和OS课很炫啦，有很多其他正经课看不到的新东西
- 感谢去年的FTL OS，你们的工作非常的炫酷，许多技术我闻所未闻，让我大开眼界
- 感谢去年的Maturin，我在调研内核设计的时候，好多次查看了你们的项目
- 感谢今年的BiteTheDisk，（虽然其实是比赛对手）在初赛的最后的赶工环节，仔细阅读linux manual有时不太来得及，因此有几个syscall的参数含义我是查看你们的实现来了解的，还有一点最重要的，我真的不知道奇妙C语言居然还有24位的数据，还好看了你们的sys_wait里面有个移动8位我才发现
