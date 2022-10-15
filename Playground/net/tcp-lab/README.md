[toc]



# tcpdump and Wireshark

> 工欲善其事, 必先利其器.

*  Wireshark 除了可以抓包外, 还提供了可视化分析网络包的图形⻚面.
* tcpdump 仅支持命令行格式使用, 常用在 Linux 服务器中抓取和分析网络包.

## tcpdump
常用选项和过滤表达式:
**e.g.1: **
```bash
ping -I eth1 -c 3 xxx
```
使用 tcpdump 抓包 ping 数据包
```bash
tcpdump -i eth1 icmp and host xxx -nn
```

```bash
# tcpdump 常用选项
# -i        tcpdump -i eth1        指定网络接口, 默认是0号接口(如eth0), any表示所有接口
# -nn       tcpdump -nn            不解析 IP 地址和端口号名称
# -c        tcpdump -c 5           限制抓取的网络包的个数
# -w        tcpdump -w a.pcap      保存到文件中
#
# tcpdump 常用过滤表达式
# host, src host, dst host              tcpdump -nn host 192.168.1.100              主机过滤
# port, src port, dst port              tcpdump -nn port 80                         端口过滤
# ip, ip6, arp, tcp, udp, icmp          tcpdump -nn host tcp                        协议过滤
# and, or, not                          tcpdump -nn host 192.168.1.100 and port 80  逻辑表达式
# tcp[tcoflages]                        tcpdump -nn "tcp[tcoflages] & tcp-syn !=0"  特定状态的 TCP 包
```
**输出格式:** 时间戳 协议 源地址.源端口 > 目的地址.目的端口 网络包详细信息



# 发送窗口分析

![](https://pic4.zhimg.com/80/v2-5da0275eaa2c129fc09c523e2207d9fb_1440w.jpg)

`「Window size value」 x 「Window size scaling factor」 = 「Caculated window size 」`