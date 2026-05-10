use std::collections::HashMap;

use wayland_client::protocol::wl_seat;

#[derive(Default)]
pub struct Seats {
    pub wl_seats: HashMap<u32, wl_seat::WlSeat>
}
