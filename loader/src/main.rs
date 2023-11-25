#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;

use alloc::format;
use core::{arch::asm, mem};
use goblin::elf;
use log::trace;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileAttribute, FileInfo, FileMode},
    table::boot::{AllocateType, MemoryDescriptor, MemoryType},
};

const EFI_PAGE_SIZE: u64 = 0x1000;

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    st.stdout().reset(false).unwrap();

    dump_mem_map(handle, &st);
    let entry_point_addr = load_kernel(handle, &st);

    trace!("entry_point_addr: {:p}", entry_point_addr as *const ());
    let entry_point: extern "sysv64" fn() = unsafe { mem::transmute(entry_point_addr) };
    entry_point();

    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

fn load_kernel(image: Handle, st: &SystemTable<Boot>) -> u64 {
    let bt = st.boot_services();

    let mut root_dir = bt
        .get_image_file_system(image)
        .unwrap()
        .open_volume()
        .unwrap();
    let mut elf_file = root_dir
        .open(
            cstr16!("kernel.elf"),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .unwrap()
        .into_regular_file()
        .unwrap();
    let mut elf_buf =
        vec![0_u8; elf_file.get_boxed_info::<FileInfo>().unwrap().file_size() as usize];
    elf_file.read(&mut elf_buf).unwrap();
    let elf = elf::Elf::parse(&elf_buf).unwrap();

    let mut dest_first = u64::MAX;
    let mut dest_last = 0_u64;
    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        dest_first = dest_first.min(ph.p_vaddr);
        dest_last = dest_last.max(ph.p_vaddr + ph.p_memsz);
    }

    st.boot_services()
        .allocate_pages(
            AllocateType::Address(dest_first),
            MemoryType::LOADER_DATA,
            ((dest_last - dest_first + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE) as usize,
        )
        .unwrap();

    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let ofs = ph.p_offset as usize;
        let fsize = ph.p_filesz as usize;
        let msize = ph.p_memsz as usize;
        let dest = unsafe { core::slice::from_raw_parts_mut(ph.p_vaddr as *mut u8, msize) };
        dest[..fsize].copy_from_slice(&elf_buf[ofs..ofs + fsize]);
        dest[fsize..].fill(0);
    }

    elf.entry
}

fn dump_mem_map(handle: Handle, st: &SystemTable<Boot>) {
    trace!("get_memory_map");

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

    trace!("done");
}
