use std::{error::Error, fs, path::Path};

use goblin::{Object, mach::Mach};

const ELF: &str = "ELF";
const PE: &str = "PE";
const MACH_O: &str = "Mach-O";

pub struct DynLibs {
    pub binary_type: &'static str,
    pub dyn_libs: Vec<String>,
}

impl DynLibs {
    fn parse_libs(libs: &[&str]) -> Vec<String> {
        libs.iter()
            .filter(|&&lib| lib != "self")
            .map(|lib| lib.to_string())
            .collect()
    }

    pub fn from_path(binary_path: &Path) -> Result<Self, Box<dyn Error>> {
        let buffer = fs::read(binary_path)?;

        let (binary_type, dyn_libs) = match Object::parse(&buffer)? {
            Object::Elf(elf) => (ELF, Self::parse_libs(&elf.libraries)),
            Object::PE(pe) => (PE, Self::parse_libs(&pe.libraries)),
            Object::Mach(mach) => match mach {
                Mach::Binary(macho) => (MACH_O, Self::parse_libs(&macho.libs)),
                Mach::Fat(_fat) => {
                    return Err(
                        format!("Unsupported Fat variant: {}", binary_path.display()).into(),
                    );
                }
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
