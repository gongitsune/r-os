#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use core::mem;

use alloc::format;
use byteorder::{ByteOrder, LittleEndian};
use log::info;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileAttribute, FileInfo, FileMode},
    table::boot::{AllocateType, MemoryDescriptor, MemoryType},
};

const KERNEL_BASE_ADDR: u64 = 0x100000;

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    st.stdout().reset(false).unwrap();

    dump_mem_map(handle, &st);
    load_kernel(handle, &st);

    // stop boot service
    start_kernel(st);

    Status::SUCCESS
}

fn start_kernel(st: SystemTable<Boot>) {
    info!("start kernel");

    let kernel_entry_addr = LittleEndian::read_u64(unsafe {
        core::slice::from_raw_parts((KERNEL_BASE_ADDR + 24) as *const u8, mem::size_of::<u64>())
    });
    info!("kernel_entry_addr: {:p}", kernel_entry_addr as *const ());

    let _ = st.exit_boot_services(MemoryType::LOADER_DATA);

    let kernel_entry: extern "sysv64" fn() = unsafe { mem::transmute(kernel_entry_addr) };
    kernel_entry();

    panic!("kernel returned");
}

fn load_kernel(handle: Handle, st: &SystemTable<Boot>) {
    info!("load kernel");

    let bt = st.boot_services();
    let mut fs = bt.get_image_file_system(handle).unwrap();

    let mut root_dir = fs.open_volume().unwrap();
    let mut elf_file = root_dir
        .open(
            cstr16!("kernel.elf"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .unwrap()
        .into_regular_file()
        .unwrap();

    let file_info = elf_file.get_boxed_info::<FileInfo>().unwrap();
    let size = file_info.file_size() as usize;

    bt.allocate_pages(
        AllocateType::Address(KERNEL_BASE_ADDR),
        MemoryType::LOADER_DATA,
        (size as usize + 0xfff) / 0x1000,
    )
    .unwrap();

    elf_file
        .read(unsafe { core::slice::from_raw_parts_mut(KERNEL_BASE_ADDR as *mut u8, size) })
        .unwrap();

    info!("done");
}

fn dump_mem_map(handle: Handle, st: &SystemTable<Boot>) {
    info!("get_memory_map");

    let bt = st.boot_services();
    let mm_size = bt.memory_map_size().map_size + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mm_buf = vec![0_u8; mm_size];
    let mm = bt.memory_map(&mut mm_buf).unwrap();

    let mut fs = bt.get_image_file_system(handle).unwrap();
    let mut root_dir = fs.open_volume().unwrap();
    let mut mm_file = root_dir
        .open(
            cstr16!("memmap"),
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap()
        .into_regular_file()
        .unwrap();

    // Write header
    mm_file
        .write("Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap();

    for (i, ele) in mm.entries().enumerate() {
        mm_file
            .write(
                format!(
                    "{} {:x} {:?} {:08x} {:x} {:x}\n",
                    i, ele.ty.0, ele.ty, ele.phys_start, ele.page_count, ele.att
                )
                .as_bytes(),
            )
            .unwrap();
    }
    drop(mm_file);

    info!("done");
}
