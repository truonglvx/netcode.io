use wrapper::netcode::*;
use wrapper::util;
use wrapper::SendError;

use std::ffi::CString;

#[derive(Debug)]
pub enum ServerError {
    Create,
    MaxClients
}

pub struct Server {
    handle: *mut netcode_server_t
}

impl Server {
    pub fn new<S>(public_addr: S, bind_addr: S, private_key: &[u8], protocol_id: u64, max_clients: usize) -> Result<Server, ServerError>
            where S: Into<String> {
        if max_clients > NETCODE_MAX_CLIENTS as usize {
            return Err(ServerError::MaxClients)
        }

        let public_addr_cstr = CString::new(public_addr.into()).unwrap();
        let bind_addr_cstr = CString::new(bind_addr.into()).unwrap();

        let handle = unsafe {
            netcode_server_create(
                ::std::mem::transmute(bind_addr_cstr.as_ptr()),
                ::std::mem::transmute(public_addr_cstr.as_ptr()),
                protocol_id,
                ::std::mem::transmute(private_key.as_ptr()),
                0.0)
        };

        if handle == ::std::ptr::null_mut() {
            return Err(ServerError::Create)
        }

        unsafe {
            netcode_server_start(handle, max_clients as i32);
        }

        Ok(Server{ handle: handle })
    }

    pub fn update(&mut self, time: f64) {
        unsafe {
            netcode_server_update(self.handle, time);
        }
    }

    pub fn send(&mut self, client_id: i32, data: &[u8]) -> Result<(), SendError> {
        if data.len() > NETCODE_MAX_PACKET_SIZE as usize {
            return Err(SendError::LengthExceeded)
        }

        unsafe {
            netcode_server_send_packet(self.handle, client_id, ::std::mem::transmute(data.as_ptr()), data.len() as i32);
        }

        Ok(())
    }

    pub fn client_connected(&mut self, client_id: i32) -> bool {
        unsafe {
            netcode_server_client_connected(self.handle, client_id) != 0
        }
    }

    pub fn receive_packet(&mut self, client_id: i32) -> Option<ServerPacket> {
        let mut bytes: ::std::os::raw::c_int = 0;

        unsafe {
            let ptr = netcode_server_receive_packet(self.handle, client_id, &mut bytes);

            if ptr == ::std::ptr::null_mut() {
                return None
            }

            Some(ServerPacket::new(self, ptr as *mut u8, bytes as usize))
        }
    }

    pub fn client_id(&mut self, client_index: usize) -> u64 {
        unsafe {
            netcode_server_client_id(self.handle, client_index as i32)
        }
    }

    pub fn num_clients_connected(&mut self) -> usize {
        unsafe {
            netcode_server_num_clients_connected(self.handle) as usize
        }
    }

    pub fn disconnect_client(&mut self, client_id: i32) {
        unsafe {
            netcode_server_disconnect_client(self.handle, client_id);
        }
    }

    pub fn disconnect_all_clients(&mut self) {
        unsafe {
            netcode_server_disconnect_all_clients(self.handle);
        }
    }
}


impl Drop for Server {
    fn drop(&mut self) {
        unsafe {
            netcode_server_destroy(self.handle);
            util::global_term();
        }
    }
}

pub struct ServerPacket<'a> {
    client: &'a Server,
    data: *mut u8,
    size: usize
}

impl<'a> ServerPacket<'a> {
    fn new(client: &'a Server, data: *mut u8, size: usize) -> ServerPacket<'a> {
        ServerPacket {
            client: client,
            data: data,
            size: size
        }
    }

    pub fn data(&self) -> &'a [u8] {
        unsafe {
            ::std::slice::from_raw_parts(self.data, self.size)
        }
    }
}

impl<'a> Drop for ServerPacket<'a> {
    fn drop(&mut self) {
        unsafe {
            netcode_server_free_packet(self.client.handle, self.data as *mut ::std::os::raw::c_void);
        }
    }
}