#![allow(dead_code)]

// https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface

// Implementation idea
// https://github.com/Knight-Ops/raspi-os/blob/master/src/bsp/driver/bcm/bcm2xxx_mailbox/bcm2837_mail.rs

#[repr(u32)]
#[derive(Debug)]
enum RpiMailboxTag {
    /* Videocore */
    GetFirmwareVersion = 0x1,

    /* Hardware */
    GetBoardModel = 0x10001,
    GetBoardRevision,
    GetBoardMacAddress,
    GetBoardSerial,
    GetArmMemory,
    GetVcMemory,
    GetClocks,

    /* Config */
    GetCommandLine = 0x50001,

    /* Shared resource management */
    GetDmaChannels = 0x60001,

    /* Power */
    GetPowerState(TagDeviceId) = 0x20001,
    GetTiming(TagDeviceId),
    SetPowerState {
        device_id: TagDeviceId,
        on: bool,
        wait: bool,
    } = 0x28001,

    /* Clocks */
    GetClockState(TagClockId) = 0x30001,
    SetClockState(bool) = 0x38001,
    GetClockRate(TagClockId) = 0x30002,

    /*    Onboard led status     */
    GetOnboardLedStatus = 0x34041,
    SetOnboardLedStatus {
        pin_number: u32,
        status: bool,
    },

    /*    Back to clocks     */
    GetClockRateMeasured(TagClockId) = 0x30047, // Real measured freq, not theoretical one
    SetClockRate {
        clock_id: TagClockId,
        rate: u32,
        skip_set_turbo: bool,
    } = 0x38002,
    GetMaxClockRate(TagClockId) = 0x30004,
    GetMinClockRate(TagClockId) = 0x30007,
    GetTurbo = 0x30009,
    SetTurbo(bool) = 0x38009,

    /* Voltage */
    GetVoltage(TagVoltageId) = 0x30003,
    SetVoltage {
        voltage_id: TagVoltageId,
        value: u32,
    } = 0x38003,
    GetMaxVoltage(TagVoltageId) = 0x30005,
    GetMinVoltage(TagVoltageId) = 0x30008,
    GetTemperature = 0x30006,
    GetMaxTemperature = 0x3000A,

    /*    GPU    */
    GpuAllocateMemory {
        size: u32,
        alignment: u32,
        flags: GpuAllocationFlags,
    } = 0x3000C,
    GpuLockMemory(u32) = 0x3000D,
    GpuUnlockMemory(u32) = 0x3000E,
    GpuReleaseMemory(u32) = 0x3000F,
    GpuExecuteCode {
        fp: u32,
        r0: u32,
        r1: u32,
        r2: u32,
        r3: u32,
        r4: u32,
        r5: u32,
    } = 0x30010,
    GetDispmanxMemHandle(u32) = 0x30014,
    GetEdidBlock(u32) = 0x30020,

    /* Framebuffer */
    AllocateBuffer(u32) = 0x40001,
    ReleaseBuffer = 0x48001,
    BlankScreen(bool) = 0x40002,

    GetPhysicalSize = 0x40003,
    TestPhysicalSize {
        width: u32,
        height: u32,
    } = 0x44003,
    SetPhysicalSize {
        width: u32,
        height: u32,
    } = 0x48003,

    GetVirtualSize = 0x40004,
    TestVirtualSize {
        width: u32,
        height: u32,
    } = 0x44004,
    SetVirtualSize {
        width: u32,
        height: u32,
    } = 0x48004,

    GetDepth = 0x40005,
    TestDepth(u32) = 0x44005,
    SetDepth(u32) = 0x48005,

    GetPixelOrder = 0x40006,
    TestPixelOrder(PixelOrder) = 0x44006,
    SetPixelOrder(PixelOrder) = 0x48006,

    GetAlphaMode = 0x40007,
    TestAlphaMode(AlphaMode) = 0x44007,
    SetAlphaMode(AlphaMode) = 0x48007,

    GetPitch = 0x40008,

    GetVirtualOffset = 0x40009,
    TestVirtualOffset {
        x: u32,
        y: u32,
    } = 0x44009,
    SetVirtualOffset {
        x: u32,
        y: u32,
    } = 0x48009,

    GetOverscan = 0x4000A,
    TestOverscan {
        top: u32,
        bottom: u32,
        right: u32,
        left: u32,
    } = 0x4400A,
    SetOverscan {
        top: u32,
        bottom: u32,
        right: u32,
        left: u32,
    } = 0x4800A,

    GetPalette = 0x4000B,
    TestPalette {
        offset: usize,
        length: usize,
        // palettes: Vec<u32>,
    } = 0x4400B,
    SetPaletsize {
        offset: usize,
        length: usize,
        // palettes: Vec<u32>,
    } = 0x4800B,

    SetCursorInfo {
        width: u32,
        height: u32,
        ptr: u32,
        hotspot_x: u32,
        hotspot_y: u32,
    } = 0x8011,
    SetCursorState {
        enable: bool,
        x: u32,
        y: u32,
        coords_from_framebuffer: bool, // If false, coords from display instead
    } = 0x8010,

    SetScreenGamma {
        display_number: u32,
        gamma_table_addr: u32,
    } = 0x8012,
}

#[repr(u32)]
#[derive(Debug)]
enum RpiMailboxReply {
    OperationSuccess(bool),
    FirmwareVersion(u32),
    BoardModel(u32),
    MacAddress(u8, u8, u8),
    BoardSerial(u64),
    ArmMemory {
        base: u32,
        size: u32,
    },
    VcMemory {
        base: u32,
        size: u32,
    },
    // Clocks(Vec<TagClockId>),
    // CommandLine(String),
    DmaChannels(u32),
    PowerState {
        device_id: TagDeviceId,
        on: bool,
        not_exists: bool,
    },
    PowerTiming {
        devide_id: TagDeviceId,
        wait_micro_sec: u32,
    }, // Wait N us before stable
    TagClockState {
        clock_id: TagClockId,
        on: bool,
        not_exists: bool,
    },
    TagClockRate {
        clock_id: TagClockId,
        freq: u32,
    }, // In Hertz
    OnboardLedStatus {
        pin_number: u32,
        status: u32,
    },
    TurboState(bool),
    VoltageState {
        voltage_id: TagVoltageId,
        value: u32,
    },
    TemperatureState(u32),
    GpuAllocation(Option<u32>),
    LockMemory(Option<u32>),
    GpuExecutionReturn(u32),
    DispmanxRessMemHandle {
        success: bool,
        handle: u32,
    },
    EdidBlock {
        block_number: u32,
        success: bool,
        edid_block: u128,
    },

    FramebufferAllocation {
        base_addr: u32,
        buff_size: u32,
    },
    BlankScreenState(bool),
    ScreenSize {
        width: u32,
        height: u32,
    },
    ScreenDepth(u32),
    PixelOrder(PixelOrder),
    AlphaMode(AlphaMode),
    ScreenPitch(u32),
    VirtualOffset {
        x: u32,
        y: u32,
    },
    ScreenOverscan {
        top: u32,
        bottom: u32,
        left: u32,
        right: u32,
    },
    // Palettes(Vec<u32>),
}

impl RpiMailboxTag {
    // fn send(&self) -> Vec<u32> {
    // }
}

#[repr(u32)]
#[derive(Debug)]
enum TagState {
    Request = 0,
    Response,
}

#[repr(u32)]
#[derive(Debug)]
enum TagBufferOffset {
    Size = 0,
    RequestOrResponse,
}

#[repr(u32)]
#[derive(Debug)]
enum TagOffset {
    Ident = 0,
    ValueSize,
    Response,
    Value,
}

#[repr(u32)]
#[derive(Debug)]
enum TagClockId {
    Reserved = 0,
    Emmc,
    Uart,
    Arm,
    Core,
    V3d,
    H264,
    Isp,
    Sdram,
    Pixel,
    Pwm,
    Hevc,
    Emmc2,
    M2Mc,
    PixelBvb,
}

#[repr(u32)]
#[derive(Debug)]
enum TagDeviceId {
    SdCard = 0,
    Uart0,
    Uart1,
    UsbHcd,
    I2c0,
    I2c1,
    I2c2,
    Spi,
    Ccp2tx,
}

#[repr(u32)]
#[derive(Debug)]
enum TagVoltageId {
    Reserved = 0,
    Core,
    SdRamC,
    SdRamP,
    SdRamI,
}

#[repr(u32)]
#[derive(Debug)]
enum GpuAllocationFlags {
    Normal = 0,      /* normal allocating alias. Don't use from ARM */
    Discardable = 1, /* can be resized to 0 at any time. Use for cached data */
    Coherent = 4,    /* Non-allocating in L2 but coherent */
    Direct = 8,      /* Uncached */
    L1Nonallocating = 12,
    Zero = 16,          /* initialise buffer to all zeros */
    NoInit = 32,        /* don't initialise (default is initialise to all ones */
    HintPermalock = 64, /* Likely to be locked for long periods of time. */
}

#[repr(u32)]
#[derive(Debug)]
enum PixelOrder {
    Bgr,
    Rgb,
}

#[repr(u32)]
#[derive(Debug)]
enum AlphaMode {
    Enabled,
    Reversed,
    Ignored,
}
