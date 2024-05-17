#[derive(Debug)]
pub struct Member {
    id_1: u16,
    id_2: u16,

    cm: u16,
    fl: Flags,
    mtime: u128,
    xfl: ExtraFlags,
    os: Os,

    extra: Option<(u32, String)>,
    name: Option<String>,
    comment: Option<String>,

    crc: u128,
    isize: u128,
}

#[derive(Debug)]
pub enum Flags {
    FTEXT = 0,
    FHCRC = 1,
    FEXTRA = 2,
    FNAME = 3,
    FCOMMENT = 4,
}

#[derive(Debug)]
pub enum ExtraFlags {
    MaxCompression = 2,
    Fastest = 4,
}

#[derive(Debug)]
pub enum Os {
    FAT = 0,
    Amiga = 1,
    VMS = 2,
    Unix = 3,
    VMCMS = 4,
    Atari = 5,
    HPFS = 6,
    Mac = 7,
    Z = 8,
    CP = 9,
    TOPS = 10,
    NTFS = 11,
    QDOS = 12,
    Acorn = 13,
}
