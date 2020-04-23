#![allow(non_camel_case_types)]

pub mod database;

use std::collections::BTreeSet;

use database::Database;

use serde::{Deserialize, Serialize};

/// Different types of processors. This should be unique enough to allow for
/// socketing decisions for motherboards. Thus it has to be compatible with
/// BGA layout as well as motherboard firmware.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProcessorType {
    XeonScalable_FCLGA3647,
    XeonScalableV2_FCLGA3647,
    XeonW_FCLGA3647,
    XeonW_FCLGA2066,
    XeonD_FCBGA2518,
}

/// A processor
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Processor {
    /// Manufacturer of the hardware
    manufacturer: &'static str,

    /// Name of the hardware/model number
    name: &'static str,

    /// Price of the processor in USD
    price: f64,

    /// Clock rate
    clock_rate: f64,

    /// Maximum all-core turbo rate
    turbo_rate: Option<f64>,

    /// Minimum AVX-512 clock rate
    avx512_rate: Option<f64>,

    /// Maximum AVX-512 turbo rate on all cores
    avx512_turbo_rate: Option<f64>,

    /// Number of cores
    cores: u32,

    /// Number of threads
    threads: u32,

    /// Number of AVX-512 FMA units
    avx512_fma_units: Option<u8>,

    /// The generation of the processor. For example `XeonScalableV2`. This
    /// determines the socketability into a motherboard, in terms of
    /// compatibility. This effectively combines the BGA layout as well as
    /// the "design" which would be compatible with a motherboard.
    typ: ProcessorType,

    /// Number of processors that can be socketed in a motherboard, given the
    /// motherboard supports it.
    scalability: u8,
    
    /// Type of memory supported by this motherboard
    mem_support: MemoryType,

    /// Number of memory channels
    mem_channels: u8,
}

/// Different types of memory
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[derive(PartialOrd, Ord)]
pub enum MemoryType {
    DDR4_2133 = 2133,
    DDR4_2400 = 2400,
    DDR4_2667 = 2667,
    DDR4_2933 = 2933,
}

/// Different classes of RAM
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemoryClass {
    DDR4,
}

impl MemoryType {
    /// Gets the class for this memory type
    pub fn class(&self) -> MemoryClass {
        match *self {
            MemoryType::DDR4_2133 => MemoryClass::DDR4,
            MemoryType::DDR4_2400 => MemoryClass::DDR4,
            MemoryType::DDR4_2667 => MemoryClass::DDR4,
            MemoryType::DDR4_2933 => MemoryClass::DDR4,
        }
    }
}

/// A stick of RAM
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Memory {
    /// Manufacturer of the hardware
    manufacturer: &'static str,

    /// Name of the hardware/model number
    name: &'static str,

    /// Price of the DIMM in USD
    price: f64,

    /// Type of the memory
    typ: MemoryType,

    /// Size of the individual DIMM in bytes
    size: u64,
}

/// Types of motherboard form factors
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MotherboardFormFactor {
    X11OPi,
    B11SRE,
    B11SPE,
    B11DPE,
    X11QPHp,
}

/// A motherboard
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Motherboard {
    /// Manufacturer of the hardware
    manufacturer: &'static str,

    /// Name of the hardware/model number
    name: &'static str,

    /// Price of the motherboard in USD
    price: f64,
    
    /// Type of the motherboard. This allows us to determine which `Blade`
    /// can support the form factor of this motherboard.
    form_factor: MotherboardFormFactor,

    /// Processor type supported by this motherboard
    proc_support: ProcessorType,

    /// Number of processors that socket into this motherboard
    scalability: u8,

    /// Number of memory sockets on the motherboard. It is implied that these
    /// are uniformly distributed between all of the processors on the
    /// motherboard. This means if this is `12` and `Scalability` is 2, that
    /// there are 6 DIMMs per CPU socket.
    memory_sockets: u8,

    /// Number of DIMMs per CPU channel
    dimms_per_channel: u8,

    /// Processors socketed in the system
    processors: Vec<Processor>,

    /// DIMMs socketed in the system,
    memory: Vec<Memory>,
}

/// Different types of blades, this determines the socketability of these
/// blades in to the blade server
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum BladeType {
    /// This is not a blade
    None,

    SBE614E,
    SBE610J,
}

/// A single blade unit part of a larger blade server, or a complete system
/// for example a whole 1U server.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Blade {
    /// Manufacturer of the hardware
    manufacturer: &'static str,

    /// Name of the hardware/model number
    name: &'static str,

    /// Price of the blade in USD
    price: f64,

    /// Different types of systems this can be installed into. Can be
    /// `BladeType::None` if this is not a blade.
    blade_type: BladeType,

    /// Motherboards supported by this blade
    mb_form_factor: BTreeSet<MotherboardFormFactor>,

    /// Motherboard which has been socketed
    motherboard: Option<Motherboard>,
}

/// For blade servers that have multiple `Blade`s, they will go into a
/// `System`
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct System {
    /// Manufacturer of the hardware
    manufacturer: &'static str,

    /// Name of the hardware/model number
    name: &'static str,

    /// Price of the system in USD
    price: f64,

    /// Types of blades supported by this system
    blade_type: BladeType,

    /// Number of blades supported by this system
    num_blades: u8,

    /// Blades that have been put into this system
    blades: Vec<Blade>,
}

macro_rules! get_proc_sum {
    ($field:ident, $typ:ty, $per_core:expr) => {
        pub fn $field(&self) -> $typ {
            let mut acc = 0 as $typ;

            // Go through each blade in the system
            for blade in &self.blades {
                let motherboard = blade.motherboard.as_ref().unwrap();

                acc += motherboard.processors.iter().fold(0 as $typ, |acc, x| {
                    acc + x.$field *
                        if $per_core { x.cores as $typ } else { 1 as $typ }
                });
            }

            acc
        }
    }
}

impl System {
    /// Determines the price of this system
    pub fn price(&self) -> f64 {
        let mut price = self.price;

        // Go through each blade in the system
        for blade in &self.blades {
            // Add the blade price
            price += blade.price;

            // Get the motherboard and add the price
            let motherboard = blade.motherboard.as_ref().unwrap();
            price += motherboard.price;
            
            // Add the price of all processors
            price += motherboard.processors.iter()
                .fold(0f64, |acc, x| acc + x.price);

            // Add the price of all memory
            price += motherboard.memory.iter()
                .fold(0f64, |acc, x| acc + x.price);
        }

        price
    }

    /// Number of bytes of RAM
    pub fn ram(&self) -> u64 {
        let mut acc = 0;

        // Go through each blade in the system
        for blade in &self.blades {
            let motherboard = blade.motherboard.as_ref().unwrap();
            acc += motherboard.memory.iter().fold(0, |acc, x| acc + x.size);
        }

        acc
    }

    /// Compute the max number of single-precision FLOPS using AVX-512 FMA
    pub fn sp_float_fma_gflops(&self) -> f64 {
        let mut acc = 0f64;

        // Go through each blade in the system
        for blade in &self.blades {
            let motherboard = blade.motherboard.as_ref().unwrap();

            acc += motherboard.processors.iter().fold(0f64, |acc, x| {
                acc + x.cores as f64 * x.avx512_rate.unwrap_or(0.) as f64 * 16. * 2. *
                    x.avx512_fma_units.unwrap_or(0) as f64
            });
        }

        acc
    }
    
    /// Compute the max number of single-precision FLOPS using AVX-512 all-core
    /// turbo FMA
    pub fn turbo_sp_float_fma_gflops(&self) -> f64 {
        let mut acc = 0f64;

        // Go through each blade in the system
        for blade in &self.blades {
            let motherboard = blade.motherboard.as_ref().unwrap();

            acc += motherboard.processors.iter().fold(0f64, |acc, x| {
                acc + x.cores as f64 * x.avx512_turbo_rate.unwrap_or(0.) as f64 * 16. * 2. *
                    x.avx512_fma_units.unwrap_or(0) as f64
            });
        }

        acc
    }

    get_proc_sum!(cores,      u32, false);
    get_proc_sum!(threads,    u32, false);
    get_proc_sum!(clock_rate, f64, true);
}

fn main() -> serde_json::Result<()> {
    let database = Database::new();

    let mut systems = Vec::new();

    loop {
        for _ in 0..100000 {
            if let Some(system) = database.random_system() {
                if system.ram() / system.threads() as u64
                        >= (4 * 1024 * 1024 * 1024) {

                    if system.price() > 35000. {
                        continue;
                    }

                    if !systems.contains(&system) {
                        systems.push(system);
                    }
                }
            }
        }

        systems.sort_by(|x, y| {
            (x.turbo_sp_float_fma_gflops() / x.price())
                .partial_cmp(&(y.turbo_sp_float_fma_gflops() / y.price()))
                .unwrap()
        });

        systems.drain(..systems.len()-50);

        eprint!("---\n");
        for (ii, system) in systems.iter().enumerate() {
            let gib = system.ram() as f64 / 1024. / 1024. / 1024.;
            eprint!("{:3} | {:4}C / {:4}T | {:9.2} base GFLOPS | {:9.2} turbo GFLOPS | {:8.2} GiB | ${:10.2} | {:10.6} base | {:10.6} turbo\n",
                ii, system.cores(),
                system.threads(), system.sp_float_fma_gflops(),
                system.turbo_sp_float_fma_gflops(),
                gib,
                system.price(),
                system.sp_float_fma_gflops() / system.price(),
                system.turbo_sp_float_fma_gflops() / system.price());

            std::fs::write(format!("best{}.txt", ii), format!("{:#?}\n", system)).unwrap();
        }
    }
}

