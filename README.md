# Another-Vantage

<p align="center">
  <a href="./README.md">简体中文</a> | <a href="./README_EN.md">English</a>
</p>

一款在Linux上实现部分 Lenovo Vantage 的工具。适用于拯救者，小新，Think等联想设备。

支持TUI与GUI两种呈现方式与命令行调用。
命令行调用示例：
```bash
./another-vantage [option] <category> [parameter]
```
```text
用法：
选项    
-h, --help  显示帮助
-v, --version  显示版本信息并退出

分类
--info  显示所有基本信息
--config  显示当前配置文件状态
--firmware 检查BIOS更新

参数
model 显示型号与SN
spec 显示参数信息
performance 显示性能配置状态
charge 显示充电配置状态
lid 显示开盖开机状态
power 显示USB供电状态
io 显示IO开关状态
```

示例：
```bash
➜  ./another-vantage --info model
Another Vantage ver 0.1.0
Device Model: ThinkBook G4+ IAP
```

期望实现功能：
- [ ] 机器基本信息
    - [ ]  机型与序列号读取
    - [ ]  机型图片展示
    - [ ]  核心配置展示
        - [ ] CPU型号，核心数，线程数，超线程开启状态，当前TDP
        - [ ] GPU型号，流处理器/流处理器组数量和当前TGP
        - [ ] 内存频率，容量，型号，形态
        - [ ] 硬盘容量，型号，接口
        - [ ] 电池供应商，型号，容量，类型，健康度
        - [ ] 屏幕尺寸，型号，分辨率
        - [ ] 主板型号，BIOS固件版本，芯片组型号
        - [ ] 声卡型号
        - [ ] 网卡型号，特性支持
    - [ ] 简易性能监视
        - [ ] CPU温度与占用率
        - [ ] GPU温度与占用率
        - [ ] 内存温度，剩余容量与占用率
        - [ ] swap可用容量与占用率
        - [ ] 硬盘温度，可用容量与占用率
- [ ]  配置文件调节
    - [ ] 性能模式更改（静音，默认，性能，性能增强，超频）
    - [ ] 充电模式更改（常规，养护，快充）
    - [ ] 开盖开机开关
    - [ ] USB关机充电开关与关闭阈值
    - [ ] IO开关
- [ ]  部分机型的高级功能
    - [ ] 部分独显机型的“仅核显/混合/独显直连”开关
    - [ ] 部分独显机型的预置超频幅度调节
    - [ ] 部分机型的风扇曲线调节
- [ ] BIOS固件版本检查更新
- [ ] 高级功能
    - [ ] 手动变更TDP，显卡核心与显存功率墙与频率
