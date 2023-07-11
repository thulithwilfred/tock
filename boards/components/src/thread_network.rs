// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

//! Component to initialize the userland UDP driver.
//!
//! This provides one Component, UDPDriverComponent. This component initializes
//! a userspace UDP driver that allows apps to use the UDP stack.
//!
//! Usage
//! -----
//! ```rust
//!    let udp_driver = UDPDriverComponent::new(
//!        board_kernel,
//!        udp_send_mux,
//!        udp_recv_mux,
//!        udp_port_table,
//!        local_ip_ifaces,
//!        PAYLOAD_LEN,
//!     )
//!     .finalize(components::udp_driver_component_static!());
//! ```

// This buffer is used as an intermediate buffer for AES CCM encryption. An
// upper bound on the required size is `3 * BLOCK_SIZE + radio::MAX_BUF_SIZE`.

use capsules_core;
use capsules_core::virtualizers::virtual_aes_ccm::MuxAES128CCM;
use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;
use capsules_extra::net::ipv6::ip_utils::IPAddr;
use capsules_extra::net::ipv6::ipv6_send::IP6SendStruct;
use capsules_extra::net::network_capabilities::{
    AddrRange, NetworkCapability, PortRange, UdpVisibilityCapability,
};
use kernel::hil::symmetric_encryption::{self, AES128Ctr, AES128, AES128CBC, AES128CCM, AES128ECB};

use capsules_extra::net::thread::driver::ThreadNetworkDriver;
use capsules_extra::net::udp::udp_port_table::UdpPortManager;
use capsules_extra::net::udp::udp_recv::MuxUdpReceiver;
use capsules_extra::net::udp::udp_recv::UDPReceiver;
use capsules_extra::net::udp::udp_send::{MuxUdpSender, UDPSendStruct, UDPSender};
use core::mem::MaybeUninit;
use kernel;
use kernel::capabilities;
use kernel::capabilities::NetworkCapabilityCreationCapability;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil::radio;
use kernel::hil::time::Alarm;

const MAX_PAYLOAD_LEN: usize = super::udp_mux::MAX_PAYLOAD_LEN;
pub const CRYPT_SIZE: usize = 3 * symmetric_encryption::AES128_BLOCK_SIZE + radio::MAX_BUF_SIZE;

// Setup static space for the objects.
#[macro_export]
macro_rules! thread_network_driver_component_static {
    ($A:ty, $B:ty $(,)?) => {{
        use components::udp_mux::MAX_PAYLOAD_LEN;

        let udp_send = kernel::static_buf!(
            capsules_extra::net::udp::udp_send::UDPSendStruct<
                'static,
                capsules_extra::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm<'static, $A>,
                >,
            >
        );
        let udp_vis_cap =
            kernel::static_buf!(capsules_extra::net::network_capabilities::UdpVisibilityCapability);
        let net_cap =
            kernel::static_buf!(capsules_extra::net::network_capabilities::NetworkCapability);
        let udp_driver =
            kernel::static_buf!(capsules_extra::net::thread::driver::ThreadNetworkDriver<'static>);
        let buffer = kernel::static_buf!([u8; MAX_PAYLOAD_LEN]);
        let udp_recv =
            kernel::static_buf!(capsules_extra::net::udp::udp_recv::UDPReceiver<'static>);
        let crypt_buf = kernel::static_buf!([u8; components::ieee802154::CRYPT_SIZE]);
        let crypt = kernel::static_buf!(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, $B>,
        );

        (
            udp_send,
            udp_vis_cap,
            net_cap,
            udp_driver,
            buffer,
            udp_recv,
            crypt_buf,
            crypt,
        )
    };};
}
pub struct UDPDriverComponent<
    A: Alarm<'static> + 'static,
    B: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB,
> {
    board_kernel: &'static kernel::Kernel,
    driver_num: usize,
    udp_send_mux:
        &'static MuxUdpSender<'static, IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>>,
    udp_recv_mux: &'static MuxUdpReceiver<'static>,
    port_table: &'static UdpPortManager,
    interface_list: &'static [IPAddr],
    aes_mux: &'static MuxAES128CCM<'static, B>,
}

impl<A: Alarm<'static>, B: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB>
    UDPDriverComponent<A, B>
{
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        driver_num: usize,
        udp_send_mux: &'static MuxUdpSender<
            'static,
            IP6SendStruct<'static, VirtualMuxAlarm<'static, A>>,
        >,
        udp_recv_mux: &'static MuxUdpReceiver<'static>,
        port_table: &'static UdpPortManager,
        interface_list: &'static [IPAddr],
        aes_mux: &'static MuxAES128CCM<'static, B>,
    ) -> Self {
        Self {
            board_kernel,
            driver_num,
            udp_send_mux,
            udp_recv_mux,
            port_table,
            interface_list,
            aes_mux,
        }
    }
}

impl<A: Alarm<'static>, B: 'static + AES128<'static> + AES128Ctr + AES128CBC + AES128ECB> Component
    for UDPDriverComponent<A, B>
{
    type StaticInput = (
        &'static mut MaybeUninit<
            UDPSendStruct<
                'static,
                capsules_extra::net::ipv6::ipv6_send::IP6SendStruct<
                    'static,
                    VirtualMuxAlarm<'static, A>,
                >,
            >,
        >,
        &'static mut MaybeUninit<
            capsules_extra::net::network_capabilities::UdpVisibilityCapability,
        >,
        &'static mut MaybeUninit<capsules_extra::net::network_capabilities::NetworkCapability>,
        &'static mut MaybeUninit<capsules_extra::net::thread::driver::ThreadNetworkDriver<'static>>,
        &'static mut MaybeUninit<[u8; MAX_PAYLOAD_LEN]>,
        &'static mut MaybeUninit<UDPReceiver<'static>>,
        &'static mut MaybeUninit<[u8; CRYPT_SIZE]>,
        &'static mut MaybeUninit<
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM<'static, B>,
        >,
    );
    type Output = &'static capsules_extra::net::thread::driver::ThreadNetworkDriver<'static>;

    fn finalize(self, s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        //crypt
        let crypt_buf = s.6.write([0; CRYPT_SIZE]);
        let aes_ccm = s.7.write(
            capsules_core::virtualizers::virtual_aes_ccm::VirtualAES128CCM::new(
                self.aes_mux,
                crypt_buf,
            ),
        );
        aes_ccm.setup(); //

        // TODO: change initialization below
        let create_cap = create_capability!(NetworkCapabilityCreationCapability);
        let udp_vis = s.1.write(UdpVisibilityCapability::new(&create_cap));
        let udp_send = s.0.write(UDPSendStruct::new(self.udp_send_mux, udp_vis));

        // Can't use create_capability bc need capability to have a static lifetime
        // so that UDP driver can use it as needed
        struct DriverCap;
        unsafe impl capabilities::UdpDriverCapability for DriverCap {}
        static DRIVER_CAP: DriverCap = DriverCap;

        let net_cap = s.2.write(NetworkCapability::new(
            AddrRange::Any,
            PortRange::Any,
            PortRange::Any,
            &create_cap,
        ));

        let buffer = s.4.write([0; MAX_PAYLOAD_LEN]);

        let thread_network_driver = s.3.write(
            capsules_extra::net::thread::driver::ThreadNetworkDriver::new(
                udp_send,
                aes_ccm,
                self.board_kernel.create_grant(self.driver_num, &grant_cap),
                self.interface_list,
                MAX_PAYLOAD_LEN,
                self.port_table,
                kernel::utilities::leasable_buffer::LeasableMutableBuffer::new(buffer),
                &DRIVER_CAP,
                net_cap,
            ),
        );
        udp_send.set_client(thread_network_driver);
        AES128CCM::set_client(aes_ccm, thread_network_driver);

        self.port_table
            .set_user_ports(thread_network_driver, &DRIVER_CAP);

        let udp_driver_rcvr = s.5.write(UDPReceiver::new());
        udp_driver_rcvr.set_client(thread_network_driver);
        let (rx_bind, tx_bind) = thread_network_driver.init_binding();
        udp_driver_rcvr.set_binding(rx_bind);
        kernel::debug!("Initial set {:?}", udp_send.set_binding(tx_bind));
        //kernel::debug!("CURR VAL {:?}", udp_send.get_binding());

        self.udp_recv_mux.add_client(udp_driver_rcvr);
        // self.udp_recv_mux.print_recv_list();
        thread_network_driver
    }
}
