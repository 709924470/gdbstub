use super::prelude::*;
use crate::arch::Arch;
use crate::protocol::commands::ext::XUpcasePacket;
use crate::target::ext::base::BaseOps;
use num_traits::Bounded;

impl<T: Target, C: Connection> GdbStubImpl<T, C> {
    pub(crate) fn handle_x_upcase_packet(
        &mut self,
        _res: &mut ResponseWriter<'_, C>,
        target: &mut T,
        command: XUpcasePacket<'_>,
    ) -> Result<HandlerStatus, Error<T::Error, C::Error>> {
        if !target.use_x_upcase_packet() {
            return Ok(HandlerStatus::Handled);
        }

        crate::__dead_code_marker!("x_upcase_packet", "impl");

        let handler_status = match command {
            XUpcasePacket::X(cmd) => {
                // Only 64 bit mismatched address is supported at the moment, change if needed uwu
                // This part finds out whats the maximum value of current target's address space.
                // And thanks for the `to_be_bytes` function returning the max # of bytes needed
                let mut buf: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
                let size = <T::Arch as Arch>::Usize::max_value()
                    .to_be_bytes(&mut buf)
                    .ok_or(Error::TargetMismatch)?;

                let addr = if size >= cmd.addr.len() {
                    // If the bytes needed for storing the address is sufficient for incoming X packet
                    <T::Arch as Arch>::Usize::from_be_bytes(cmd.addr).ok_or(Error::TargetMismatch)
                } else {
                    // Or its a 64 bit to 32 bit conversion (Normally occurs on mips-mti-gdb)
                    match size {
                        4 => {
                            // the address space is 32 bit, then copy the last 4 bytes into buffer
                            let mut _buf: [u8; 4] = [0, 0, 0, 0];
                            _buf.copy_from_slice(&cmd.addr[cmd.addr.len() - 4..]);

                            // Then parse the truncated buffer
                            <T::Arch as Arch>::Usize::from_be_bytes(&_buf)
                                .ok_or(Error::TargetMismatch)
                        }
                        _ => Err(Error::TargetMismatch), // Other conventions are not supported, add if you need one
                    }
                }?;

                match target.base_ops() {
                    BaseOps::SingleThread(ops) => ops.write_addrs(addr, cmd.val),
                    BaseOps::MultiThread(ops) => {
                        ops.write_addrs(addr, cmd.val, self.current_mem_tid)
                    }
                }
                .handle_error()?;

                HandlerStatus::NeedsOk
            }
        };
        Ok(handler_status)
    }
}
