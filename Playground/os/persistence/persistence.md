**Persistence**
> Persistence: "A firm or obstinate continuance in a course of action in spite of difficulty or opposition."

# 存储设备原理

## [存储设备原理.Q1 - 状态机的状态是如何存储的?]
## [存储设备原理.Q2 - 更多的持久状态是如何存储的?]

## 计算机需要存储 "当前状态"
机器指令模型 (Instruction Set Architecture) 只有 "两种" 状态
* 寄存器: rax, rbx, ..., cr3, ...
* 物理内存
存储 "当前状态" 的需求
* 可以寻址 (根据编号读写数据)
* 访问速度尽可能快 (甚至不惜规定状态在掉电后丢失)
  * 也因此有了 memory hierarchy

### 持久化的第一课: 持久存储介质
构成一切文件的基础:
* 逻辑上是一个 bit/byte array
* 根据局部性原理，允许我们按 “大块” 读写

#### 存储介质: 磁
![铁磁体玩具](https://jyywiki.cn/pages/OS/img/mag-draw-board.jpg)
磁带 (Magnetic Tape, 1928) -> 磁鼓 (Magnetic Drum, 1932) -> 磁盘 (Hard Disk, 1956) -> 软盘 (Floppy Disk, 1971) -> Compact Disk (CD, 1980) -> Solid State Drive (SSD, 1991)

![磁盘](https://jyywiki.cn/pages/OS/img/disk-mechanism.jpg)

**磁盘：性能调优**

为了读/写一个扇区
1. 读写头需要到对应的磁道
* 7200rpm → 120rps → “寻道” 时间 8.3ms
2. 转轴将盘片旋转到读写头的位置
* 读写头移动时间通常也需要几个 ms
通过缓存/调度等缓解
* 例如著名的 "电梯" 调度算法
* 现代 HDD (hard disk drive) 都有很好的 firmware 管理磁盘 I/O 调度, 实际上操作系统不再"自以为聪明地"去调度磁盘了。
**软盘: 把读写头和盘片分开——实现数据移动 (今天已彻底被 USB Flash Disk 杀死)**

#### 存储介质: 坑
光盘
* 读写速度
  * 顺序读取速度高; 随机读取勉强
  * 写入速度低 (挖坑容易填坑难)

#### 存储介质: Finally 
之前的持久存储介质都有致命的缺陷
* 磁: 机械部件导致 ms 级延迟
* 坑 (光): 一旦挖坑, 填坑很困难 (光盘是只读的)

**最后还得靠电 (电路) 解决问题**
* Flash Memory "闪存"
  * Floating gate 的充电/放电实现 1-bit 信息的存储
分析
* 价格
  * 低 (大规模集成电路, 便宜)
* 容量
  * 高 (3D 空间里每个 (x, y, z)(x,y,z) 都是一个 bit)
* 读写速度
  * 高 (直接通过电路读写)
  * 不讲道理的特性: 容量越大, 速度越快 (电路级并行)
  * 快到淘汰了旧的 SATA 接口标准 (NVMe)
* 可靠性
  * 高 (没有机械部件, 随便摔)
但是,
放电 (erase) 做不到 100% 放干净
* 放电数千/数万次以后, 就好像是 "充电" 状态了
* dead cell; "wear out" (必须解决这个问题 SSD 才能实用)

[NAND](https://en.wikipedia.org/wiki/NAND_gate) Wear-Out 的解决: 软件定义磁盘
* 每一个 SSD 里都藏了一个完整的计算机系统
![软件定义磁盘](https://jyywiki.cn/pages/OS/img/ssd.png)
* [NAND flash管理的核心FTL](https://zhuanlan.zhihu.com/p/26944064)

**Flash Translation Layer (FTL): 安全性的难题**
* 首先, (快速) 格式化是没用的
(M5 会告诉你这一点)
* 在你理解了 FTL 之后
  * 即便格式化后写入数据 (不写满)
    * 同一个 logic block 被覆盖, physical block 依然存储了数据 (copy-on-write)
    * 需要文件系统加密

# 输入输出设备 (串口/键盘/磁盘/打印机/总线/中断控制器/DMA/GPU)
## [输入输出设备.Q1 器件之上的"设备"到底什么?]
I/O 设备 (控制器):一组交换数据的接口和协议

## 计算机与外设的接口
从一个需求说起: 如何用计算机实现核弹发射箱？
* 关键问题: 如何使计算机能**感知外部状态、对外实施动作?**

I/O 设备: "计算" 和 "物理世界" 之间的桥梁
* 其实 I/O 设备就是"几组约定好功能的线"
  * 每一组线有自己的地址, CPU 可以直接使用指令 (in/out/MMIO) 和设备交换数据

* I/O 设备 (CPU 视角): **"一个能与 CPU 交换数据的接口/控制器"**

![I/O 设备](https://jyywiki.cn/pages/OS/img/canonical-device.png)

### EXAMPLE 1: 键盘控制器
IBM PC/AT 8042 PS/2 (Keyboard) Controller
"硬编码" 到两个 I/O port: 0x60 (data), 0x64 (status/command)

|  Command Byte   |  Use        |   说明                             |
| --------------- | ------------| --------------------------------- |
|  0xED           |LED 灯控      | ScrollLock/NumLock/CapsLock       |
|  0xF3           |设置重复速度    | 30Hz - 2Hz; Delay: 250 - 1000ms  |
| 0xF4/0xF5     	|打开/关闭      |	N/A                              |
|  0xFE         	|重新发送	      |	N/A                              |
|  0xFF         	|RESET	       |        N/A                     

```text
+------｜--------------------------｜---------------+
|   +------+              +----------------+       |
|   | data |              | status/command |       |
|   +------+              +----------------+       |
+------｜--------------------------｜---------------+
       ｜                          ｜
+------｜--------------------------｜---------------+
| +---+  +---+ +---+ +---+ +---+                   ｜
| | Q |  | W | | E | | R | | T | ...               ｜
| +---+  +---+ +---+ +---+ +---+                   ｜
|                   keyboard                       ｜
+--------------------------------------------------+
```

### EXAMPLE 2: 磁盘控制器
ATA (Advanced Technology Attachment) 老的磁盘控制器
* IDE (Integrated Drive Electronics) 接口磁盘
  * primary: 0x1f0 - 0x1f7; secondary: 0x170 - 0x177
```c
void readsect(void *dst, int sect) {
  waitdisk();
  out_byte(0x1f2, 1);          // sector count (1)
  out_byte(0x1f3, sect);       // sector
  out_byte(0x1f4, sect >> 8);  // cylinder (low)
  out_byte(0x1f5, sect >> 16); // cylinder (high)
  out_byte(0x1f6, (sect >> 24) | 0xe0); // drive
  out_byte(0x1f7, 0x20);       // command (write)
  waitdisk();
  for (int i = 0; i < SECTSIZE / 4; i ++)
    ((uint32_t *)dst)[i] = in_long(0x1f0); // data
}
```

## 总线、中断控制器和 DMA
如果你只造 "一台计算机", 随便给每个设备定一个端口/地址, 用 [mux](https://en.wikipedia.org/wiki/Multiplexer) 连接到 CPU 就行. 但是总是希望接入更多 I/O 设备, 甚至是未知的设备, 但不希望改变 CPU **--> 总线: 一个特殊的 I/O 设备**

### 总线：一个特殊的 I/O 设备
提供**设备的注册**和**地址到设备的转发**
* (总线能干啥?) 把收到的地址 (总线地址) 和数据转发到相应的设备上

这样 CPU 只需要直连一个总线 就行了！
* 今天 PCI 总线肩负了这个任务
* `lspci -tv` 和 `lsusb -tv`: 查看系统中总线上的设备


[图解总线](https://www.cnblogs.com/pengxurui/p/16893747.html)
[深入 PCI 与 PCIe 之一](https://zhuanlan.zhihu.com/p/26172972)

### 中断控制器
CPU 有一个中断引脚
* 收到某个特定的电信号会触发中断
  * 保存 5 个寄存器 (cs, rip, rflags, ss, rsp)
  * 跳转到中断向量表对应项执行

![6502](https://jyywiki.cn/pages/OS/img/6502-pinout.jpg)

实际上今天 Intel 的 CPU: APIC (Advanced PIC)
* local APIC: 中断向量表, [Inter-processor interrupt, IPI](https://en.wikipedia.org/wiki/Inter-processor_interrupt), 时钟, ...
* I/O APIC: 其他 I/O 设备
  
```text
+----------------------+
|        CPU           |
|+-------+  +-------+  |
|| APIC  |  | IOAPIC|  |
|+--|----+  +-----|-+  | 
+---|--------|----|----+
addr|        |    | IRQ 
+---|--------|----|----+                   +-------------------+
|         总线          --------------------|      DRAM         |
|                      |                   +-------------------+
+----------|-----------+
           |
          设备
```

### 中断没能解的问题
假设程序希望写入 1 GB 的数据到磁盘
* 即便磁盘已经准备好, 依然需要非常浪费缓慢的循环
* out 指令写入的是设备缓冲区, 需要去总线上绕一圈
  * cache disable; store 其实很慢的
```c
for (int i = 0; i < 1 GB / 4; i++) {
  outl(PORT, ((u32 *)buf)[i]);
}
```
#### [Q - 能否把 CPU 从执行循环中解放出来?]
比如, 在系统里征用一个小 CPU, 专门复制数据? -> Direct Memory Access (DMA)
**DMA: 一个专门执行 "memcpy" 程序的 CPU** (不需要像传统 CPU 那么复杂)
* 支持的几种 memcpy
  * memory → memory
  * memory → device (register)
  * device (register) → memory
    * 实际实现: 直接把 DMA 控制器连接在总线和内存上
    * [Intel 8237A](https://jyywiki.cn/pages/OS/manuals/i8237a.pdf)

### GPU 和异构计算
我们还可以有做各种事情的 "CPU" (DMA 不就是一个 "做一件特别事情" 的 CPU 吗?)
例如, 显示图形
```c
for (int i = 1; i <= H; i++) {
  for (int j = 1; j <= W; j++)
    putchar(j <= i ? '*' : ' ');
  putchar('\n');
}
```
难办的是性能: NES: 6502 @ 1.79Mhz; IPC(instructions per cycle/clock.) = 0.43
* 屏幕共有 256 x 240 = 61K 像素 (256 色)
* 60FPS → 每一帧必须在 ~10K 条指令内完成

#### 现代 GPU: 一个通用计算设备
程序保存在内存 (显存) 中
* nvcc (LLVM) 分两个部分
  * main 编译/链接成本地可执行的 ELF
  * kernel 编译成 GPU 指令 (送给驱动)
数据也保存在内存 (显存) 中
* 可以输出到视频接口 (DP, HDMI, ...)
* 也可以通过 DMA 传回系统内存

![CPU versus GPU](https://developer-blogs.nvidia.com/zh-cn-blog/wp-content/uploads/sites/2/2022/04/gpu-devotes-more-transistors-to-data-processing-1024x506.png)

# 文件系统 API
## [文件系统.Q - 如何使应用程序能共享存储设备?]

## 为什么需要文件系统?
磁盘需要支持数据的持久化
* 程序数据
  * 可执行文件和动态链接库
  * 应用数据 (高清图片、过场动画、3D 模型...)
* 用户数据
  * 文档、下载、截图、replay...
* 系统数据
  * Manpages 
  * 系统配置

但是! 设备在应用程序之间的共享是有难度的
* 多个进程并行打印，如何保证不混乱? (printf-race.c)
* 让所有应用共享磁盘? 一个程序 bug 操作系统就没了.

**所以, 字节序列并不是磁盘的好抽象.**
## 文件系统: 虚拟磁盘
文件系统: 设计目标
* 提供合理的 API 使多个应用程序能共享数据
* 提供一定的隔离, 使恶意/出错程序的伤害不能任意扩大

"存储设备 (字节序列) 的虚拟化"
* 磁盘 (I/O 设备) = 一个可以读/写的字节序列
* **虚拟磁盘** (文件) = 一个可以读/写的动态字节序列
  * 命名管理: 虚拟磁盘的名称、检索和遍历
  * 数据管理: std::vector<char> (随机读写/resize)


## 虚拟磁盘: 命名管理
