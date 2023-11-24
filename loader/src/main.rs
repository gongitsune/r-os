#![no_std]
#![no_main]

use uefi::{prelude::*, table::runtime::ResetType};

#[entry]
fn efi_main(handle: Handle, mut st: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut st).unwrap();
    st.stdout().reset(false).unwrap();

    st.stdout()
        .output_string(cstr16!("Hello, world!\n"))
        .unwrap();

    st.boot_services().stall(3_000_000);

    st.stdout().reset(false).unwrap();
    st.runtime_services()
        .reset(ResetType::SHUTDOWN, Status::SUCCESS, None);
}
