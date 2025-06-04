pub trait BlockDevice {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]);
    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]);
}

#[derive(Debug, Clone, Copy)]
pub struct BootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fats: u8,
    pub sectors_per_fat: u32,
    pub root_cluster: u32,
}

impl BootSector {
    pub fn parse(buf: &[u8; 512]) -> Self {
        Self {
            bytes_per_sector: u16::from_le_bytes([buf[11], buf[12]]),
            sectors_per_cluster: buf[13],
            reserved_sectors: u16::from_le_bytes([buf[14], buf[15]]),
            fats: buf[16],
            sectors_per_fat: u32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]),
            root_cluster: u32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]),
        }
    }
}

pub struct Fat32<D: BlockDevice> {
    device: D,
    boot_sector: BootSector,
}

impl<D: BlockDevice> Fat32<D> {
    pub fn new(mut device: D) -> Result<Self, ()> {
        let mut buf = [0u8; 512];
        device.read_sector(0, &mut buf);
        let boot_sector = BootSector::parse(&buf);
        Ok(Self { device, boot_sector })
    }

    pub fn boot_sector(&self) -> &BootSector {
        &self.boot_sector
    }
}

pub struct MemoryDisk {
    data: [u8; 4096],
}

impl MemoryDisk {
    pub fn new() -> Self {
        let mut data = [0u8; 4096];
        // minimal boot sector setup
        data[11..13].copy_from_slice(&512u16.to_le_bytes());
        data[13] = 1; // sectors per cluster
        data[14..16].copy_from_slice(&1u16.to_le_bytes()); // reserved
        data[16] = 2; // fats
        data[36..40].copy_from_slice(&1u32.to_le_bytes()); // sectors per fat
        data[44..48].copy_from_slice(&2u32.to_le_bytes()); // root cluster
        Self { data }
    }
}

impl BlockDevice for MemoryDisk {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) {
        let start = lba as usize * 512;
        buf.copy_from_slice(&self.data[start..start + 512]);
    }

    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) {
        let start = lba as usize * 512;
        self.data[start..start + 512].copy_from_slice(buf);
    }
}
