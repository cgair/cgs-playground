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

NAND Wear-Out 的解决: 软件定义磁盘
* 每一个 SSD 里都藏了一个完整的计算机系统
![软件定义磁盘](https://jyywiki.cn/pages/OS/img/ssd.png)
* [NAND flash管理的核心FTL](https://zhuanlan.zhihu.com/p/26944064)
* 
**Flash Translation Layer (FTL): 安全性的难题**
* 首先, (快速) 格式化是没用的
(M5 会告诉你这一点)
* 在你理解了 FTL 之后
  * 即便格式化后写入数据 (不写满)
    * 同一个 logic block 被覆盖, physical block 依然存储了数据 (copy-on-write)
    * 需要文件系统加密



# 文件系统 API
## [文件系统.Q - 如何使应用程序能共享存储设备?]
