pub trait BlockDevice {
    fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]);
    fn write_sector(&mut self, lba: u32, buf: &[u8; 512]);
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone, Copy)]
pub struct BootSector {
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub reserved_sectors: u16,
    pub fats: u8,
    pub sectors_per_fat: u32,
    pub root_cluster: u32,
}

#[cfg_attr(feature = "alloc", derive(Debug, Clone))]
pub struct DirectoryEntry {
    pub name: [u8; 11],
    pub attr: u8,
    pub first_cluster: u32,
    pub size: u32,
}

#[cfg(feature = "alloc")]
impl DirectoryEntry {
    pub fn filename(&self) -> String {
        let name = core::str::from_utf8(&self.name[..8]).unwrap().trim_end();
        let ext = core::str::from_utf8(&self.name[8..]).unwrap().trim_end();
        if ext.is_empty() {
            String::from(name)
        } else {
            format!("{}.{}", name, ext)
        }
    }
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

    fn cluster_size(&self) -> usize {
        self.boot_sector.bytes_per_sector as usize
            * self.boot_sector.sectors_per_cluster as usize
    }

    fn first_data_sector(&self) -> u32 {
        self.boot_sector.reserved_sectors as u32
            + self.boot_sector.fats as u32 * self.boot_sector.sectors_per_fat
    }

    fn cluster_to_lba(&self, cluster: u32) -> u32 {
        self.first_data_sector() + (cluster - 2) * self.boot_sector.sectors_per_cluster as u32
    }

    fn read_fat_entry(&mut self, cluster: u32) -> u32 {
        let fat_start = self.boot_sector.reserved_sectors as u32;
        let offset = cluster * 4;
        let sector = fat_start + (offset / 512);
        let idx = (offset % 512) as usize;
        let mut buf = [0u8; 512];
        self.device.read_sector(sector, &mut buf);
        let entry = u32::from_le_bytes([
            buf[idx],
            buf[idx + 1],
            buf[idx + 2],
            buf[idx + 3],
        ]);
        entry & 0x0FFF_FFFF
    }

    fn read_cluster(&mut self, cluster: u32, buf: &mut [u8]) {
        let lba = self.cluster_to_lba(cluster);
        let mut tmp = [0u8; 512];
        self.device.read_sector(lba, &mut tmp);
        buf[..512].copy_from_slice(&tmp);
    }

    fn read_cluster_chain(&mut self, start: u32) -> Result<Vec<u8>, ()> {
        if start < 2 {
            panic!("invalid cluster {}", start);
        }
        let cluster_size = self.cluster_size();
        let mut current = start;
        let mut data = Vec::new();
        loop {
            let mut buf = vec![0u8; cluster_size];
            self.read_cluster(current, &mut buf);
            data.extend_from_slice(&buf);
            let next = self.read_fat_entry(current);
            if next >= 0x0FFF_FFF8 {
                break;
            }
            if next < 2 {
                panic!("invalid FAT entry {}", next);
            }
            current = next;
        }
        Ok(data)
    }

    pub fn read_root_directory(&mut self) -> Result<Vec<DirectoryEntry>, ()> {
        let data = self.read_cluster_chain(self.boot_sector.root_cluster)?;
        let mut entries = Vec::new();
        for chunk in data.chunks(32) {
            if chunk[0] == 0x00 { break; }
            if chunk[0] == 0xE5 { continue; }
            let mut name = [0u8; 11];
            name.copy_from_slice(&chunk[0..11]);
            let attr = chunk[11];
            let first_cluster =
                ((u16::from_le_bytes([chunk[20], chunk[21]]) as u32) << 16)
                | u16::from_le_bytes([chunk[26], chunk[27]]) as u32;
            let size = u32::from_le_bytes([
                chunk[28], chunk[29], chunk[30], chunk[31]
            ]);
            entries.push(DirectoryEntry { name, attr, first_cluster, size });
        }
        Ok(entries)
    }

    pub fn open_file(&mut self, entry: &DirectoryEntry) -> Result<Vec<u8>, ()> {
        let mut data = self.read_cluster_chain(entry.first_cluster)?;
        data.truncate(entry.size as usize);
        Ok(data)
    }
}

pub struct MemoryDisk {
    data: [u8; 4096],
}

impl MemoryDisk {
    pub fn new() -> Self {
        let mut data = [0u8; 4096];
        // boot sector
        data[11..13].copy_from_slice(&512u16.to_le_bytes());
        data[13] = 1; // sectors per cluster
        data[14..16].copy_from_slice(&1u16.to_le_bytes()); // reserved sectors
        data[16] = 2; // number of FATs
        data[36..40].copy_from_slice(&1u32.to_le_bytes()); // sectors per FAT
        data[44..48].copy_from_slice(&2u32.to_le_bytes()); // root cluster = 2

        // FAT table (sector 1 and 2)
        let mut fat_sector = [0u8; 512];
        fat_sector[0..4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes()); // entry 0
        fat_sector[4..8].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // entry 1
        fat_sector[8..12].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes()); // cluster2 end
        fat_sector[12..16].copy_from_slice(&0x0FFFFFFFu32.to_le_bytes()); // cluster3 end
        data[512..1024].copy_from_slice(&fat_sector); // FAT1
        data[1024..1536].copy_from_slice(&fat_sector); // FAT2

        // root directory (cluster2 -> sector 3)
        let start = 1536; // 3*512
        let name: [u8; 11] = *b"HELLO   TXT";
        let mut dir = [0u8; 32];
        dir[0..11].copy_from_slice(&name);
        dir[11] = 0x20; // file attr
        dir[26..28].copy_from_slice(&3u16.to_le_bytes()); // first cluster low
        dir[28..32].copy_from_slice(&5u32.to_le_bytes()); // file size
        data[start..start + 32].copy_from_slice(&dir);

        // file data cluster3 (sector 4)
        let data_start = 2048; // 4*512
        data[data_start..data_start + 5].copy_from_slice(b"Hello");

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
