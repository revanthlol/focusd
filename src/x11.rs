use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, GetPropertyReply};

pub struct X11Backend {
    conn: x11rb::rust_connection::RustConnection,
    atom_net_active_window: u32,
    atom_wm_class: u32,
    atom_utf8_string: u32,
    atom_net_wm_name: u32,
}

impl X11Backend {
    pub fn new() -> anyhow::Result<Self> {
        let (conn, _screen_num) = x11rb::connect(None)?;
        
        // Intern necessary atoms (constants used by X11 to identify properties)
        let atom_net_active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?.reply()?.atom;
        let atom_wm_class = conn.intern_atom(false, b"WM_CLASS")?.reply()?.atom;
        let atom_utf8_string = conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom;
        let atom_net_wm_name = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;

        Ok(Self {
            conn,
            atom_net_active_window,
            atom_wm_class,
            atom_utf8_string,
            atom_net_wm_name,
        })
    }

    pub fn get_focused_window(&self) -> Option<(String, String)> {
        let root = self.conn.setup().roots[0].root;

        // 1. Ask Root window for the Active Window ID
        let reply = self.conn.get_property(
            false, root, self.atom_net_active_window, 
            AtomEnum::WINDOW, 0, 1
        ).ok()?.reply().ok()?;

        if reply.value_len == 0 { return None; }
        
        // X11 returns data as raw bytes
        let window_id = u32::from_ne_bytes(reply.value[0..4].try_into().ok()?);

        // 2. Get WM_CLASS (The stable App ID)
        let class_reply = self.conn.get_property(
            false, window_id, self.atom_wm_class, 
            AtomEnum::STRING, 0, 1024
        ).ok()?.reply().ok()?;
        
        let app_id = self.parse_string_property(&class_reply);

        // 3. Get _NET_WM_NAME (The window title)
        let title_reply = self.conn.get_property(
            false, window_id, self.atom_net_wm_name, 
            self.atom_utf8_string, 0, 1024
        ).ok()?.reply().ok()?;

        let title = self.parse_string_property(&title_reply);

        if app_id.is_empty() { return None; }
        
        // Normalize App ID (WM_CLASS often comes as "gnome-terminal\0Gnome-terminal")
        // We usually want the capitalized or second part
        let stable_id = app_id.split('\0').last().unwrap_or(&app_id).to_string();

        Some((stable_id, title))
    }

    fn parse_string_property(&self, reply: &GetPropertyReply) -> String {
        // Convert raw bytes to UTF-8 String
        String::from_utf8_lossy(&reply.value).to_string()
    }
}