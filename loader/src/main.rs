#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use core::mem;

use alloc::format;
use log::info;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileAttribute, FileMode},
    table::{boot::MemoryDescriptor, runtime::ResetType},
};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    st.stdout().reset(false).unwrap();

    dump_mem_map(handle, &st);

    st.boot_services().stall(3_000_000);
    st.stdout().reset(false).unwrap();
    st.runtime_services()
        .reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
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
