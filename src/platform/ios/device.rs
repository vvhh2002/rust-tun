//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.
#![allow(unused_variables)]

use std::io::{self, Read, Write};
use std::net::Ipv4Addr;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::sync::Arc;

use crate::configuration::Configuration;
use crate::device::Device as D;
use crate::error::*;
use crate::platform::posix::{self, Fd};

/// A TUN device for iOS.
pub struct Device {
    queue: Queue,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        if let Some(read_fd)=config.read_fd {
            //这里忽略了原来的raw_fd
            let write_fd= match config.write_fd {
                Some(write_raw_fd) => write_raw_fd,
                _ => return Err(Error::InvalidConfig),
            };
            let mut device = unsafe {
                let tun = Fd::new(read_fd).map_err(|_| io::Error::last_os_error())?;
                let read_tun = Fd::new(read_fd).map_err(|_| io::Error::last_os_error())?;
                let write_tun = Fd::new(write_fd).map_err(|_| io::Error::last_os_error())?;
                Device {
                    queue: Queue { tun, read_tun, write_tun},
                }
            };
            Ok(device)
        } else {
            let fd= match config.raw_fd {
                Some(raw_fd) => raw_fd,
                _ => return Err(Error::InvalidConfig),
            };
            let mut device = unsafe {
                let tun = Fd::new(fd).map_err(|_| io::Error::last_os_error())?;
                let read_tun = Fd::new(fd).map_err(|_| io::Error::last_os_error())?;
                let write_tun = Fd::new(fd).map_err(|_| io::Error::last_os_error())?;
                Device {
                    queue: Queue { tun, read_tun, write_tun},
                }
            };
            Ok(device)
        }
    }

    /// Split the interface into a `Reader` and `Writer`.
    // pub fn split(self) -> (posix::Reader, posix::Writer) {
    //     let fd = Arc::new(self.queue.tun);
    //     (posix::Reader(fd.clone()), posix::Writer(fd.clone()))
    // }

    /// Split the interface into a `Reader` and `Writer`.
    ///
    pub fn split(self) -> (posix::Reader, posix::Writer) {
        if self.queue.write_tun.as_raw_fd() == self.queue.read_tun.as_raw_fd() {
            let fd = Arc::new(self.queue.tun);
            (posix::Reader(fd.clone()), posix::Writer(fd.clone()))
        } else {
            let read_fd = Arc::new(self.queue.read_tun);
            let write_fd = Arc::new(self.queue.write_tun);
            (posix::Reader(read_fd), posix::Writer(write_fd))
        }
    }

    /// Return whether the device has packet information
    pub fn has_packet_information(&self) -> bool {
        self.queue.has_packet_information()
    }

    #[cfg(feature = "async")]
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.queue.set_nonblock()
    }
}

// impl Read for Device {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         self.queue.tun.read(buf)
//     }
// }
//
// impl Write for Device {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         self.queue.tun.write(buf)
//     }
//
//     fn flush(&mut self) -> io::Result<()> {
//         self.queue.tun.flush()
//     }
// }


impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queue.read(buf)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.queue.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.queue.flush()
    }
}



impl D for Device {
    type Queue = Queue;

    fn name(&self) -> &str {
        return "";
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn broadcast(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn netmask(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        Ok(())
    }

    fn mtu(&self) -> Result<i32> {
        Err(Error::NotImplemented)
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        Ok(())
    }

    fn queue(&mut self, index: usize) -> Option<&mut Self::Queue> {
        if index > 0 {
            return None;
        }

        Some(&mut self.queue)
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.queue.tun.as_raw_fd()
    }
}

impl IntoRawFd for Device {
    fn into_raw_fd(self) -> RawFd {
        self.queue.tun.into_raw_fd()
    }
}

pub struct Queue {
    tun: Fd,
    read_tun:Fd,
    write_tun:Fd,
}

impl Queue {
    pub fn has_packet_information(&self) -> bool {
        // on ios this is always the case
        true
    }

    #[cfg(feature = "async")]
    pub fn set_nonblock(&self) -> io::Result<()> {
        if self.write_tun.as_raw_fd() == self.read_tun.as_raw_fd() {
            self.tun.set_nonblock()
        }else{
            if let Err(e)=self.write_tun.set_nonblock(){
                return Err(e);
            }
            self.read_tun.set_nonblock()
        }

    }
}

// impl AsRawFd for Queue {
//     fn as_raw_fd(&self) -> RawFd {
//         self.tun.as_raw_fd()
//     }
// }
//
// impl IntoRawFd for Queue {
//     fn into_raw_fd(self) -> RawFd {
//         self.tun.into_raw_fd()
//     }
// }

impl Read for Queue {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.write_tun.as_raw_fd() == self.read_tun.as_raw_fd() {
            self.tun.read(buf)
        } else {
            self.read_tun.read(buf)
        }
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.write_tun.as_raw_fd() == self.read_tun.as_raw_fd() {
            self.tun.write(buf)
        } else {
            self.write_tun.write(buf)
        }

    }

    fn flush(&mut self) -> io::Result<()> {
        if self.write_tun.as_raw_fd() == self.read_tun.as_raw_fd() {
            self.tun.flush()
        } else {
            self.write_tun.flush()
            // self.read_tun.flush(); //read 不用 flush
        }

    }
}
