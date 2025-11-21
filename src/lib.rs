use std::{error::Error, fs, path::Path};

use goblin::{
    Object,
    mach::{Mach, MultiArch, SingleArch},
};

const ELF: &str = "ELF";
const PE: &str = "PE";
const MACH_O: &str = "Mach-O";
const MACH_O_FAT: &str = "Mach-O (Fat)";

pub struct DynLibEntry {
    pub index: usize,
    pub dyn_libs: Vec<String>,
}

pub enum DynLibEntries {
    SingleArch(Vec<String>),
    MultiArch(Vec<DynLibEntry>),
}

pub struct DynLibs {
    pub binary_type: &'static str,
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
