# Another-Vantage 实机探测快照：21CY

> 探测日期：2026-10-17
> 机器：LENOVO 21CY / ThinkBook 16 G4+ IAP
> 系统：Ubuntu 26.04.1 LTS，kernel `7.0.0-31-generic`
> 状态：只读探测。本次会话无法 `sudo`，`current_value` 等特权值未读取，写操作未测试。

## 0. 结论摘要

| 项 | 结论 |
|---|---|
| 机型识别 | `product_name=21CY`；`product_version` 与 `product_family` 均为 `ThinkBook 16 G4+ IAP`；**没有 `system-family` 节点** |
| 性能模式 | `/sys/firmware/acpi/platform_profile` 存在；当前 `performance`；choices 为 `low-power balanced performance`；provider 为 `ideapad-laptop` |
| DYTC | `allow_v4_dytc=N`，且 21CY 不在 v4 白名单（上游仅 82L5、IdeaPad 5 15ITL05）⇒ 实际为 **DYTC v5**，无需内核参数 |
| 性能增强模式 | 是 BIOS 设置项，不是第四个 `platform_profile`；本机 `thinklmi` 未导出相关属性，Linux 侧不可读写 |
| 充电模式 | `BAT0/charge_types` 可写，三档 `Fast / Standard / Long_Life`；`conservation_mode` 已被内核标记 deprecated；无 `charge_control_*` 阈值 |
| 开盖开机 | 本机 `VPC2004:00` 无 `flip_to_start` / `instant_boot`；`thinklmi` 也无 lid 项 ⇒ 21CY 不可用 |
| IO 开关 | `thinklmi` 已导出 `USBPort`、`MemoryCardSlot`、`IntegratedCamera`、`Bluetooth`、`FingerprintReader`、`Microphone`、`EthernetLAN`、`Thunderbolt(TM)`、`WirelessLAN`、`AlwaysOnUSB`；`current_value` 为 `0600`，读值需 root |
| GPU | 仅 Iris Xe 核显，无 dGPU；无 GPU hwmon；i915 只有 `gt_*_freq_mhz`，无温度/利用率 sysfs |
| 内存温度 | 无 DIMM hwmon ⇒ 不可得 |
| 硬盘温度 | NVMe 由 `nvme` 驱动直接提供 hwmon（实测 27.85 °C），不需要 `drivetemp` |
| BIOS 更新 | `fwupdmgr get-updates` 无更新；BIOS1 / System Firmware 列为“固件更新不可用” |
| PPD | `power-profiles-daemon` active，ActiveProfile=`performance`，PlatformDriver=`platform_profile` |
| Rust | 未安装；`cc/gcc/make/pkg-config` 可用 |

## 1. 运行环境

| 项 | 值 |
|---|---|
| 主机名 | `bluesword-linux-to-go` |
| 发行版 | Ubuntu 26.04.1 LTS (Resolute Raccoon) |
| 内核 | `7.0.0-31-generic` |
| 启动 | UEFI |
| 根文件系统 | `/dev/sda4`，btrfs，**ro**（`/` 与 `/home` 均为只读） |
| 工作区 | `/home/bluesword/Projects/Another-Vantage`，由沙箱 bind 为 **rw** |
| 系统盘 | USB SSD `SSD8` 931.5 GB（`sda`） |
| 内部盘 | `nvme0n1` WD Blue SN570 1 TB；`nvme1n1` SAMSUNG MZVL2512HCJQ-00BL2 476.9 GB |
| 提权 | 本次会话无 `sudo`（no_new_privs）；仅验证了权限位，未验证特权值/写入 |

> 注意：这是 USB 便携 Linux，不是最终的双系统安装。探测结果对 21CY 硬件有效，但开发环境（可写系统 + Rust）仍未就绪。

## 2. DMI / 机型

| 字段 | 值 | 权限 |
|---|---|---|
| `sys_vendor` | `LENOVO` | 0444 |
| `product_name` | `21CY` | 0444 |
| `product_version` | `ThinkBook 16 G4+ IAP` | 0444 |
| `product_family` | `ThinkBook 16 G4+ IAP` | 0444 |
| `product_sku` | `LENOVO_MT_21CY_BU_idea_FM_ThinkBook 16 G4+ IAP` | 0444 |
| `board_name` | `LNVNB161216` | 0444 |
| `board_version` | `SDK0K17763 WIN` | 0444 |
| `bios_version` | `HYCN54WW` | 0444 |
| `bios_date` | `06/30/2025` | 0444 |
| `ec_firmware_release` | `1.54` | 0444 |
| `chassis_type` | `10`（Notebook） | 0444 |
| `product_serial` | **不可读** | 0400 |
| `product_uuid` | **不可读** | 0400 |
| `system-family` | **不存在** | — |

结论：报告里的 `SystemFamily` 字段名不准确；实现应使用 `product_version` / `product_family`。README 示例 `ThinkBook G4+ IAP` 少了 `16`，应更正。

## 3. CPU / GPU / 主板

| 项 | 值 |
|---|---|
| CPU | `12th Gen Intel(R) Core(TM) i9-12900H` |
| 拓扑 | 14 核 / 20 线程 / 1 socket；`thread_siblings_list=0-1`（CPU0） |
| GPU | Intel Iris Xe Graphics（`8086:46a6`，Alder Lake-P GT2），驱动 `i915` |
| dGPU | 无（`lspci` 无 NVIDIA/AMD 独显） |
| i915 频率节点 | `/sys/class/drm/card1/gt_*_freq_mhz`（cur/min/max/RP0/RP1/RPn/boost） |
| i915 温度/利用率 | **无 sysfs 节点** |
| 声卡 | Intel Alder Lake PCH-P HD Audio（`8086:51c8`，`sof-audio-pci-intel-tgl`） |
| 有线网卡 | Intel I219-V（`8086:1a1f`，`e1000e`） |
| 无线网卡 | Intel Wi-Fi 6E AX210（`8086:2725`，`iwlwifi`） |

## 4. 性能模式 / DYTC / PPD

### 4.1 实测

```text
/sys/firmware/acpi/platform_profile         = performance
/sys/firmware/acpi/platform_profile_choices = low-power balanced performance
/sys/module/ideapad_laptop/parameters/allow_v4_dytc = N
/sys/class/platform-profile/platform-profile-0/name = ideapad-laptop
```

`/sys/class/platform-profile/platform-profile-0` 指向 `.../VPC2004:00/platform-profile/platform-profile-0`，即 provider 是 `ideapad-laptop`，不是 Intel SoC slider。

### 4.2 DYTC 版本推断

上游 `ideapad-laptop.c` 的逻辑：

- `dytc_version < 4`：不支持；
- `dytc_version < 5` 且既未设置 `allow_v4_dytc`、又不在 `ideapad_dytc_v4_allow_table[]`：不注册；
- v4 白名单目前只有 `82L5`（IdeaPad 5 Pro 16ACH6）和 `IdeaPad 5 15ITL05`。

21CY 不在白名单、`allow_v4_dytc=N`，但 provider 已注册 ⇒ **DYTC 版本为 v5**，不需要 `ideapad_laptop.allow_v4_dytc=Y`。报告 §3 的“最大技术风险”在本机不存在；v4 白名单问题只影响旧机型/旧内核。

### 4.3 DYTC 模式映射

上游源码注释与映射：

| DYTC 模式 | 值 | platform_profile | 语义 |
|---|---|---|---|
| `DYTC_MODE_LOW_POWER` | 3 | `low-power` | 低功耗 / 静音 |
| `DYTC_MODE_BALANCE` | 0xF | `balanced` | 默认 / 平衡 |
| `DYTC_MODE_PERFORM` | 2 | `performance` | 性能 |

**没有“性能增强”这个 DYTC 模式。** 用户所述“性能增强模式”是 BIOS 设置项：它改变 `performance` 档的实际 EC/固件行为。

### 4.4 BIOS “性能增强模式”

- 本机 `thinklmi` 的 28 个属性中没有 Performance / Thermal / Enhanced / Quiet 相关项（见 §5）。
- 因此 Linux 侧既不能读、也不能写这个 BIOS 开关；只能提示用户进 BIOS 设置。
- 在其他机型上，若 `firmware-attributes` 暴露了对应属性（名称可能是 PerformanceMode / EnhancedPerformance / ThermalMode 等），应通过能力探测发现，不能硬编码。
- “静音性能模式”同理：部分机型可能是独立的硬件/BIOS 配置；21CY 的静音档对应 `platform_profile=low-power`。实现时以 `platform_profile_choices` 和 BIOS 能力为准，不要固定成 5 档。

### 4.5 power-profiles-daemon

| 项 | 值 |
|---|---|
| 服务 | `power-profiles-daemon` **active** |
| 当前 profile | `performance` |
| D-Bus | `net.hadess.PowerProfiles` |
| Profiles | `power-saver` / `balanced` / `performance` |
| PlatformDriver | `platform_profile` |

⇒ 直接写 `/sys/firmware/acpi/platform_profile` 会与 PPD 的状态不一致。CLI 应优先通过 PPD 的 D-Bus 设置；若直接写 sysfs，必须检测 PPD 并提示“由 power-profiles-daemon 管理”。

## 5. BIOS 设置：`thinklmi`

### 5.1 存在性

```text
/sys/class/firmware-attributes/thinklmi/
├── attributes/
├── authentication/{Admin,Power-on}/
└── power/
```

`authentication/Admin`：`is_enabled=0`（未设 BIOS 管理员密码），`mechanism=password`，`role=bios-admin`。
`attributes/save_settings=single`，`attributes/pending_reboot=0`。

### 5.2 已导出属性（28 项）

```text
AlwaysOnUSB  BIOSBackFlash  Bluetooth  ChargeInBatteryMode  DeviceGuard
EFI-BootOrder  EthernetLAN  FastBoot  FingerprintReader  FoolProofFnCtrl
HotkeyMode  IntegratedCamera  IntelPlatformTrustTechnology
Intel(R)Hyper-ThreadingTechnology  Intel(R)VirtualizationTechnology
Intel(R)VT-dFeature  IPV4PXEFirst  LenovoCloudServices  MemoryCardSlot
Microphone  PXEBootToLAN  SATAControllerMode  SecureBoot  Thunderbolt(TM)
USBBoot  USBPort  WiFiLenovoCloudServices  WirelessLAN
```

### 5.3 与 README “IO 开关”相关的候选 allowlist

| 属性 | 语义 | 备注 |
|---|---|---|
| `USBPort` | USB 端口访问 | 具体是全部端口还是分组需实机确认 |
| `MemoryCardSlot` | 读卡器 | |
| `IntegratedCamera` | 摄像头 | 与 EC 的 `camera_power` 可能重叠 |
| `Bluetooth` | 蓝牙 | |
| `FingerprintReader` | 指纹 | |
| `Microphone` | 麦克风 | |
| `EthernetLAN` | 有线网卡 | |
| `Thunderbolt(TM)` | Thunderbolt | 本机是否有 TB 硬件需另查 |
| `WirelessLAN` | 无线网卡 | 与 `rfkill` 可能重叠 |
| `AlwaysOnUSB` | USB 关机充电 | 与 `VPC2004:00/usb_charging` 可能重叠 |

**不能把全部 28 项都当作 IO 开关**：`SecureBoot`、`DeviceGuard`、`BIOSBackFlash`、`SATAControllerMode` 等属于安全/启动/存储设置，误写风险高。

### 5.4 权限与读写

| 文件 | 权限 | 说明 |
|---|---|---|
| `attributes/*/current_value` | `0600` | **读值/写值都需要 root** |
| `attributes/*/possible_values` | `0444` | 可枚举 |
| `attributes/*/type` | `0444` | 正常导出；bug 216460 在本机未复现 |
| `attributes/save_settings` | `0644` | `single` |
| `attributes/pending_reboot` | `0444` | `0` |

`current_value` 本次未读取（无 root）；写入也未测试。

## 6. EC / VPC2004

`/sys/bus/platform/devices/VPC2004:00/` 文件清单：

```text
camera_power  conservation_mode  driver  driver_override  fan_mode
firmware_node  fn_lock  input  leds  modalias  platform-profile
power  rfkill  subsystem  uevent  usb_charging  wakeup
```

| 属性 | 值 | 说明 |
|---|---|---|
| `conservation_mode` | `0` | 已 deprecated，改用 `charge_types` |
| `usb_charging` | `1` | USB 关机充电开关；**无阈值** |
| `camera_power` | `1` | 与 thinklmi `IntegratedCamera` 可能重叠 |
| `fn_lock` | `1` | 非 README 主功能 |
| `fan_mode` | `16` | 驱动写只接受 0/1/2/4；读回 16 的语义待实测 |
| `flip_to_start` | **不存在** | 开盖开机在 21CY 不可用 |
| `instant_boot` | **不存在** | 同上 |

`fan_mode` 的已知映射（来自竞品/驱动）：0=Super Silent，1=Standard，2=Dust Cleaning，4=Efficient Thermal Dissipation。21CY 读回 16，不能直接当作上述四档；若要暴露，需先实测写 0/1/2/4 的效果。

## 7. 电池与充电

| 字段 | 值 |
|---|---|
| `manufacturer` | `LGES` |
| `model_name` | `L21L4PD8` |
| `technology` | `Li-ion` |
| `energy_full` | `65480000` µWh（65.48 Wh） |
| `energy_full_design` | `71000000` µWh（71 Wh） |
| 健康度 | 约 **92.2%** |
| `cycle_count` | `165` |
| `capacity` | `95` |
| `status` | `Not charging` |
| `charge_types` | `[Fast] Standard Long_Life`（可写，需 root） |
| `charge_control_*_threshold` | **不存在** |
| `charge_behaviour` | **不存在** |

⇒ README 的“常规 / 养护 / 快充”三档可通过 `charge_types` 实现（Standard / Long_Life / Fast）；“关闭阈值”不可实现。`conservation_mode` 仍可写，但内核会打印 deprecated 警告，不建议使用。

## 8. 传感器（hwmon）

| hwmon | name | 说明 |
|---|---|---|
| hwmon0 | `ADP1` | 电源适配器 |
| hwmon1 | `acpitz` | ACPI 热区 |
| hwmon2 | `BAT0` | 电池 |
| hwmon3 | `nvme` | NVMe 温度（实测 27.85 °C） |
| hwmon4 | `nvme` | 第二块 NVMe |
| hwmon5 | `coretemp` | CPU 核心温度 |
| hwmon6 | `iwlwifi_1` | 无线网卡温度 |

- **无 GPU 温度节点**（Intel 核显）。
- **无 DIMM/内存温度节点**。
- 硬盘温度直接读 `nvme` hwmon，无需 `drivetemp`（那是 SATA 用）。

## 9. 存储 / 显示

### 9.1 存储

| 设备 | 型号 | 容量 | 接口 |
|---|---|---|---|
| `nvme0n1` | WD Blue SN570 1TB | 931.5 GB | NVMe |
| `nvme1n1` | SAMSUNG MZVL2512HCJQ-00BL2 | 476.9 GB | NVMe |
| `sda` | USB `SSD8` | 931.5 GB | USB（便携系统盘） |

### 9.2 屏幕

```text
/sys/class/drm/card1-eDP-1/status = connected
modes[0] = 2560x1600
EDID: Manufacturer=CSO, Model=5644, Maximum image size=34 cm x 22 cm
```

⇒ 分辨率、尺寸可由 EDID 得到；型号是面板代码 `CSO 5644`，不是市场名称。对角约 16.0 英寸。

## 10. 固件更新

```text
fwupdmgr get-updates
→ 无更新可用
→ 固件更新不可用的设备包括：BIOS1、System Firmware、UEFI Device Firmware ...
```

⇒ 21CY 无 LVFS 刷写路径；只能做“版本检查 + 提示”，不能做 Linux 侧 BIOS 更新。

## 11. 权限矩阵

| 操作 | 普通用户 | root |
|---|---|---|
| DMI 型号/BIOS/主板/EC 版本 | ✅ | ✅ |
| DMI `product_serial` / `product_uuid` | ❌ | ✅ |
| `/proc`、多数 `/sys/class/power_supply` | ✅ | ✅ |
| `platform_profile` 读 | ✅ | ✅ |
| `platform_profile` 写 | ❌ | ✅ |
| `thinklmi current_value` 读/写 | ❌ | ✅ |
| `thinklmi possible_values` / `type` | ✅ | ✅ |
| `charge_types` 读 | ✅ | ✅ |
| `charge_types` / `usb_charging` / `fan_mode` 写 | ❌ | ✅ |
| `intel-rapl energy_uj` | ❌ | ✅ |
| RAPL `constraint_*_power_limit_uw` 写 | ❌ | ✅ |
| EDID / DRM modes | ✅ | ✅ |

⇒ 报告“读免 root”需要改成“部分读免 root”；SN、BIOS 当前值、RAPL 能耗都是特权读。

## 12. 工具链与网络

| 项 | 状态 |
|---|---|
| Rust | `rustc` / `cargo` / `rustup` 均无；`~/.cargo`、`~/.rustup` 无 |
| Python | 3.14.4；`gi`、`PyQt6`、`rich` 可用；`PyQt5`、`textual` 缺失 |
| GTK4 | 运行时 typelib 缺失（`Gtk 4.0` / `Adw 1` 不可导入）；`pkg-config gtk4`、`libadwaita-1` 缺失 |
| C 工具链 | `cc`、`gcc`、`make`、`pkg-config` 可用；`clang` 无 |
| 网络 | `curl` 访问 github / kernel.org / crates.io / rustup.rs 均 200 |
| `web_fetch` | 仍因 DNS fake-ip（`fdfe:dcba:9876::*`）被判“非公网 IP” |
| `sudo` | 本次会话不可用（no_new_privs） |
| 发行版工具 | `dmidecode`、`lspci`、`lsblk`、`lshw`、`fwupdmgr`、`edid-decode`、`iw`、`ethtool`、`rfkill` 可用 |

`dmidecode` 非 root 直接失败（`/dev/mem`、DMI tables 0400）；即便 root，在 lockdown/STRICT_DEVMEM 下也可能失败。内存频率/型号/形态可能拿不到。

## 13. 待补测（需 root / 真实终端）

1. `thinklmi` 各 `current_value` 的实际值（USBPort、MemoryCardSlot 等）。
2. `thinklmi` 写入后的 `pending_reboot` / 回读行为（建议先在一两项低风险属性上测试）。
3. `charge_types` 写 `Fast` / `Standard` / `Long_Life` 的效果与回读。
4. `fan_mode` 写 0/1/2/4 的实际效果；当前读回 16 的含义。
5. `platform_profile` 写 low-power/balanced/performance 与 PPD、EC 的联动。
6. BIOS “性能增强模式”开关开启/关闭时，`platform_profile=performance` 的实际差异（需重启进 BIOS）。
7. `dmidecode -t 17` 在 root 下是否可用（内存频率/型号/形态）。
8. `intel-rapl` 功耗读数口径（当前 `constraint_0=200 W`、`constraint_1=80 W`，但 `max_power=45 W`，自相矛盾）。

## 14. 复现命令

```bash
# 环境
uname -a; cat /etc/os-release; findmnt -T /; lsblk -d -o NAME,SIZE,TRAN,MODEL

# DMI
for f in product_name product_version product_family product_sku board_name \
         board_version bios_version bios_date ec_firmware_release; do
  printf '%s = ' "$f"; cat /sys/devices/virtual/dmi/id/$f
done
stat -c '%A %n' /sys/devices/virtual/dmi/id/{product_serial,product_uuid}

# 性能模式
cat /sys/firmware/acpi/platform_profile /sys/firmware/acpi/platform_profile_choices
cat /sys/module/ideapad_laptop/parameters/allow_v4_dytc
readlink -f /sys/class/platform-profile/platform-profile-0
cat /sys/class/platform-profile/platform-profile-0/name

# BIOS 设置
for d in /sys/class/firmware-attributes/thinklmi/attributes/*/; do
  printf '%s: ' "$(basename "$d")"
  cat "$d/display_name" 2>/dev/null
done
stat -c '%A %n' /sys/class/firmware-attributes/thinklmi/attributes/USBPort/current_value

# EC
for f in /sys/bus/platform/devices/VPC2004:00/{conservation_mode,usb_charging,camera_power,fn_lock,fan_mode}; do
  printf '%s = ' "$(basename "$f")"; cat "$f"
done

# 电池
for f in manufacturer model_name technology energy_full energy_full_design \
         cycle_count capacity status charge_types; do
  printf '%s = ' "$f"; cat /sys/class/power_supply/BAT0/$f
done

# 传感器
for h in /sys/class/hwmon/hwmon*/; do printf '%s: ' "$(basename "$h")"; cat "$h/name"; done

# 屏幕
cat /sys/class/drm/card1-eDP-1/status
head -1 /sys/class/drm/card1-eDP-1/modes
edid-decode /sys/class/drm/card1-eDP-1/edid | grep -iE 'Manufacturer|Model|Image size'

# 固件 / PPD
fwupdmgr get-updates
powerprofilesctl get
```
