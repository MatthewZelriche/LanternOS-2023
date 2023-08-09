use core::{fmt::Display, ops::Sub};

#[derive(Default, Clone, Copy)]
pub struct MemSize {
    pub bytes: u64,
}

impl MemSize {
    const KIB_DIV: f64 = 1024.0;
    const MIB_DIV: f64 = 1024.0 * 1024.0;
    const GIB_DIV: f64 = 1024.0 * 1024.0 * 1024.0;

    pub fn to_bytes(&self) -> u64 {
        self.bytes
    }

    pub fn to_kib(&self) -> f64 {
        self.bytes as f64 / MemSize::KIB_DIV
    }

    pub fn to_mib(&self) -> f64 {
        self.bytes as f64 / MemSize::MIB_DIV
    }

    pub fn to_gib(&self) -> f64 {
        self.bytes as f64 / MemSize::GIB_DIV
    }
}

impl Sub for MemSize {
    type Output = u64;

    fn sub(self, rhs: Self) -> Self::Output {
        self.bytes - rhs.bytes
    }
}

impl Display for MemSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            x if x.to_bytes() < 1024 => write!(f, "{} B", self.to_bytes()),
            x if x.to_kib() < 1024.0 => write!(f, "{:.3} KiB", self.to_kib()),
            x if x.to_mib() < 1024.0 => write!(f, "{:.3} MiB", self.to_mib()),
            x if x.to_gib() < 1024.0 => write!(f, "{:.3} GiB", self.to_gib()),
            _ => write!(f, "{} B", self.to_bytes()),
        }
    }
}
