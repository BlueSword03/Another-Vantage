# Another-Vantage

<p align="center">
  <a href="./README.md">简体中文</a> | <a href="./README_EN.md">English</a>
</p>

一款在 Linux 上实现部分 Lenovo Vantage 功能的工具。适用于拯救者、小新、Think 等联想设备。

**当前状态：规划中，代码未开工。** 第一版只做 **CLI 只读**（`--info` / `--config` / `--firmware`），写入调节后置。
支持矩阵以开发机 **ThinkBook 16 G4+ IAP（21CY）** 实测为准；其他机型标“未验证”。

## 使用

```bash
./another-vantage [--info [子项] | --config [子项] | --firmware] [选项]
```

```text
类别（flag）
  --info        读取基本信息
  --config      读取硬件配置状态（不是“配置文件”）
  --firmware    检查 BIOS 版本与更新可用性

子项（item）
  model  spec  performance  charge  lid  power  io

选项
  --json        机器可读输出
  --no-color    禁用颜色
  --verbose     详细日志
  --version     显示版本
  --help        显示帮助
```

说明：

- `--info` 不带子项 ⇒ 显示全部基本信息；`--config` 不带子项 ⇒ 显示全部配置状态。
- 先实现读取，**写入尚未实现**。后续写入计划使用 `--set KEY=VALUE`（如 `--set performance=quiet`），并补 `--dry-run` / `--yes`。
- 部分读取需要管理员权限：SN（`product_serial`）、BIOS 当前值（`thinklmi/*/current_value`）、RAPL 能耗。遇到权限不足时会提示：

```text
需要管理员权限：请使用 root 用户或 sudo 执行，例如：
  sudo another-vantage --info model
```

- 所有写入都需要 root；写入后必须回读校验。
- 不做内核模块，不做 Linux 侧 BIOS 刷写。

示例：

```bash
➜  ./another-vantage --info model
Another Vantage ver 0.1.0
Device Model: ThinkBook 16 G4+ IAP
```

> 机型以 DMI `product_version` 为准；旧示例里的 `ThinkBook G4+ IAP` 少了 `16`，已更正。

## 21CY 实测支持矩阵

> 下表是 **接口可行性实测**，不代表功能已经实现。实现进度见下方复选框。
> ✅ 可用｜⚠️ 部分可用/需 root/待实测｜❌ 本机不可用或不支持｜— 未验证/机型相关

| 功能 | 21CY | 说明 |
|---|---|---|
| 机型 / SN | ✅ / ⚠️ | 机型免 root；SN 为 `0400`，需 root |
| CPU 型号/核数/线程/HT | ✅ | i9-12900H，14C/20T |
| CPU 当前 TDP | ⚠️ | RAPL 读数口径待定；`energy_uj` 需 root |
| GPU 型号 | ✅ | Iris Xe（无 dGPU） |
| GPU 流处理器 / TGP | ❌ | 核显无此概念 |
| 内存容量 | ✅ | 32 GB |
| 内存频率/型号/形态 | ⚠️ | 依赖 `dmidecode`；非 root/lockdown 下可能不可得 |
| 硬盘容量/型号/接口 | ✅ | 2× NVMe |
| 电池厂商/型号/容量/类型/健康度 | ✅ | LGES / L21L4PD8 / 65.48 Wh / Li-ion / 92.2% / 165 cycles |
| 屏幕尺寸/型号/分辨率 | ✅ | EDID：CSO 5644，34×22 cm，2560×1600 |
| 主板/BIOS/芯片组 | ✅ | LNVNB161216 / HYCN54WW |
| 声卡/网卡型号/特性 | ✅ | Alder Lake PCH-P HDA / AX210 / I219-V |
| CPU 温度/占用 | ✅ | coretemp + `/proc/stat` |
| GPU 温度/占用 | ❌ | 核显无温度 sysfs；占用需 `intel_gpu_top`/PMU |
| 内存温度 | ❌ | 无 DIMM 通路 |
| 硬盘温度 | ✅ | NVMe hwmon，无需 `drivetemp` |
| swap | ✅ | 16 GB |
| 性能模式 | ⚠️ | `platform_profile` 3 档（low-power/balanced/performance）+ BIOS“性能增强”开关 |
| 充电模式 | ✅ | `charge_types`：Fast / Standard / Long_Life |
| 开盖开机 | ❌ | 21CY 无 `flip_to_start` / `instant_boot` |
| USB 关机充电 | ✅ / ❌ | 开关可做；**关闭阈值不可做**（无 `charge_control_*`） |
| IO 开关 | ⚠️ | `thinklmi` 10 项 allowlist；读值/写值需 root |
| 风扇模式 / 风扇曲线 | ⚠️ / ❌ | `fan_mode` 存在但语义待实测；曲线不可得 |
| 独显直连 / 超频 | ❌ | 本机无 dGPU、无 Legion 接口 |
| BIOS 更新 | ⚠️ | 只能检查；21CY 无 LVFS 刷写路径 |
| 手动 TDP / 功率墙 | ⚠️ | 需 root/MSR，本机未验证 |
| 机型图片 | — | 计划中，需要资产映射 |
| 中文文档 / 本地化 | ✅ | 中文优先 |

## 期望实现功能

- [ ] 机器基本信息
    - [ ] 机型与序列号读取（SN 需 root）
    - [ ] 机型图片展示（计划）
    - [ ] 核心配置展示
        - [ ] CPU 型号、核心数、线程数、超线程状态、当前 TDP（⚠️ 口径待定）
        - [ ] GPU 型号（本机 Iris Xe）；流处理器/TGP 对核显不适用
        - [ ] 内存频率、容量、型号、形态（⚠️ 依赖 dmidecode）
        - [ ] 硬盘容量、型号、接口
        - [ ] 电池供应商、型号、容量、类型、健康度
        - [ ] 屏幕尺寸、型号、分辨率（EDID）
        - [ ] 主板型号、BIOS 版本、芯片组型号
        - [x] 声卡型号（lspci 型号、平台/PCI 驱动、PCI ID）
        - [x] 网卡型号与驱动（含无线标识；频段/速率等特性后续补充）
    - [ ] 简易性能监视
        - [ ] CPU 温度与占用率
        - [ ] GPU 温度与占用率（⚠️ 核显可能不可得）
        - [ ] 内存温度（❌ 本机无通路）、剩余容量与占用率
        - [ ] swap 可用容量与占用率
        - [ ] 硬盘温度、可用容量与占用率
- [ ] 硬件配置调节（**写入后置**）
    - [ ] 性能模式：3 档 `platform_profile`；“性能增强”由 BIOS 设置决定，Linux 侧可能不可见
    - [ ] 充电模式：`charge_types` 三档（常规 / 养护 / 快充）
    - [ ] 开盖开机：机型相关；21CY 不支持
    - [ ] USB 关机充电开关（阈值不支持）
    - [ ] IO 开关：`thinklmi` allowlist（USBPort / MemoryCardSlot / IntegratedCamera / Bluetooth / FingerprintReader / Microphone / EthernetLAN / Thunderbolt / WirelessLAN / AlwaysOnUSB）
- [ ] 部分机型的高级功能
    - [ ] 独显直连（Legion/部分独显机型；21CY 不适用）
    - [ ] 预置超频（Legion/部分机型；21CY 不适用）
    - [ ] 风扇曲线（21CY 仅有粗粒度 `fan_mode`，曲线不可得）
- [ ] BIOS 固件版本检查（不做刷写）
- [ ] 高级功能
    - [ ] 手动变更 TDP、显卡核心与显存功率墙与频率（⚠️ 风险高，最后做）

## 从源码构建

需要 Rust stable 与 `cargo`。本会话把工具链装在工作区 `.toolchain/`（不污染系统；细节见 `docs/HANDOFF.md`）。

```bash
cargo build --release
cargo test

./target/release/another-vantage --info model
./target/release/another-vantage --info spec --json
sudo ./target/release/another-vantage --info model   # SN 等特权读需要 root/sudo
```

当前只读 CLI 已可用；写入、TUI、GUI 尚未实现。

## 设计原则

1. **能力探测优先**：启动扫描已知节点，产出“本机支持什么”；不支持项显示明确原因，不静默留白。
2. **只读先行**：先把 `--info` / `--config` / `--firmware` 做稳，再加写入。
3. **写后回读**：任何写入都必须回读断言；被 EC/固件覆盖时明确提示。
4. **最小特权**：普通读取不用 root；特权读取提示 sudo；写入必须 root；不引入常驻 daemon。
5. **不做内核模块、不做 BIOS 刷写、不写任意 BIOS 属性**：BIOS 设置只允许 allowlist。
6. **前端与数据层解耦**：CLI/TUI/GUI 共用同一套能力模型。

## 开发状态

- [x] 21CY 实机接口探测（见 `docs/probe-21CY.md`）
- [x] 竞品功能核实
- [x] CLI 只读方案与特性矩阵
- [x] 语言选型：Rust（GUI 时机待定）
- [x] 阶段 1：只读 CLI 初版（`--info` / `--config` / `--firmware` + `--json`）
- [ ] 阶段 2：监视
- [ ] 阶段 3：写入
- [ ] 阶段 4：TUI
- [ ] 阶段 5：GUI
