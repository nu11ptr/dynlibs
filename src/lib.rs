use std::{error::Error, fs, path::Path};

use goblin::{
    Object,
    mach::{Mach, MultiArch, SingleArch},
};

/// An ELF binary
pub const ELF: &str = "ELF";
/// A PE binary
pub const PE: &str = "PE";
/// A Mach-O binary
pub const MACH_O: &str = "Mach-O";
/// A Mach-O (Fat) binary
pub const MACH_O_FAT: &str = "Mach-O (Fat)";

// TODO: Add a name for the architecture
/// The dynamic libraries for a single architecture in multi-architecture binaries
pub struct DynLibEntry {
    /// The index of the architecture in the multi-architecture binary
    pub index: usize,
    /// The dynamic libraries of this architecture
    pub dyn_libs: Vec<String>,
}

/// One or more collections of dynamic libraries in a binary
pub enum DynLibEntries {
    /// The binary is a single architecture, so it has only a single collection of dynamic libraries
    SingleArch(Vec<String>),
    /// The binary is a multi-architecture binary, so it has multiple collections of dynamic libraries
    MultiArch(Vec<DynLibEntry>),
}

/// A collection of dynamic libraries in a binary
pub struct DynLibs {
    /// The type of binary
    pub binary_type: &'static str,
    /// The dynamic libraries in the binary
    pub dyn_libs: DynLibEntries,
}

impl DynLibs {
    fn parse_libs(libs: &[&str]) -> Vec<String> {
        libs.iter()
            .filter(|&&lib| lib != "self")
            .map(|lib| lib.to_string())
            .collect()
    }

    fn parse_single_entry(libs: &[&str]) -> DynLibEntries {
        DynLibEntries::SingleArch(Self::parse_libs(libs))
    }

    fn parse_mach_o_fat(fat: &MultiArch) -> Result<(&'static str, DynLibEntries), Box<dyn Error>> {
        let mut dyn_lib_entries = Vec::with_capacity(fat.narches);

        for idx in 0..fat.narches {
            match fat.get(idx)? {
                SingleArch::MachO(macho) => {
                    dyn_lib_entries.push(DynLibEntry {
                        index: idx,
                        dyn_libs: Self::parse_libs(&macho.libs),
                    });
                }
                SingleArch::Archive(_archive) => {
                    // We don't handle archives, so we skip them
                    continue;
                }
            }
        }
        Ok((MACH_O_FAT, DynLibEntries::MultiArch(dyn_lib_entries)))
    }

    /// Parse the dynamic libraries from a binary path. Returns an error if the binary path is not valid or the binary is not supported
    pub fn from_path(binary_path: &Path) -> Result<Self, Box<dyn Error>> {
        let buffer = fs::read(binary_path)?;

        let (binary_type, dyn_libs) = match Object::parse(&buffer)? {
            Object::Elf(elf) => (ELF, Self::parse_single_entry(&elf.libraries)),
            Object::PE(pe) => (PE, Self::parse_single_entry(&pe.libraries)),
            Object::Mach(mach) => match mach {
                Mach::Binary(macho) => (MACH_O, Self::parse_single_entry(&macho.libs)),
                Mach::Fat(fat) => Self::parse_mach_o_fat(&fat)?,
            },
            _ => {
                return Err(format!("Unsupported binary format: {}", binary_path.display()).into());
            }
        };

        Ok(Self {
            binary_type,
            dyn_libs,
        })
    }
}
